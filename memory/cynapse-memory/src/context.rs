//! DENDRITE system-prompt assembly.
//!
//! Faithful port of Go `internal/memory/dendrite_context.go`: budgets,
//! relevance discovery, scoring, and prompt caching.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::graph::{Node, NodeType, Dendrite};
use crate::store::DendriteStore;

pub const DEFAULT_MAX_TOKENS: usize = 6000;
/// 40% of the token budget for core identity nodes.
pub const CORE_NODE_BUDGET: f64 = 0.40;
/// Maximum number of candidate nodes returned by `find_relevant` before scoring.
const MAX_CANDIDATES: usize = 50;

/// Core identity nodes always included first.
const CORE_IDS: [&str; 5] = ["identity", "cynapse_core", "soul", "agents", "tools"];

struct CacheState {
    cached_prompt: String,
    cached_at: Instant,
    cache_ttl: std::time::Duration,
    dirty: bool,
}

/// Assembles the LLM system prompt from graph nodes with zero-deadlock lock hierarchy.
pub struct DendriteContext {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
    cache: Arc<Mutex<CacheState>>,
    cb_id: u64,
    pub default_max_tokens: usize,
    pub core_node_budget: f64,
}

impl Drop for DendriteContext {
    fn drop(&mut self) {
        self.graph.unregister_on_change(self.cb_id);
    }
}

impl DendriteContext {
    pub fn new(graph: Arc<Dendrite>, store: Option<Arc<DendriteStore>>) -> Arc<DendriteContext> {
        let cache = Arc::new(Mutex::new(CacheState {
            cached_prompt: String::new(),
            cached_at: Instant::now(),
            cache_ttl: std::time::Duration::from_secs(300),
            dirty: true,
        }));

        let weak_cache = Arc::downgrade(&cache);
        let cb: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            if let Some(c) = weak_cache.upgrade() {
                if let Ok(mut state) = c.lock() {
                    state.dirty = true;
                }
            }
        });

        let cb_id = graph.register_on_change(cb);

        Arc::new(DendriteContext {
            graph,
            store,
            cache,
            cb_id,
            default_max_tokens: DEFAULT_MAX_TOKENS,
            core_node_budget: CORE_NODE_BUDGET,
        })
    }

    /// Return the system prompt. If `user_message` is non-empty it biases
    /// context toward relevant nodes; otherwise a cached general prompt.
    pub fn build_prompt(&self, user_message: &str, max_tokens: usize) -> String {
        let effective_max_tokens = if max_tokens == 0 {
            self.default_max_tokens
        } else {
            max_tokens
        };

        // Message-specific context always recomputes without holding cache lock
        if !user_message.trim().is_empty() {
            return assemble(&self.graph, self.store.as_deref(), user_message, effective_max_tokens, self.core_node_budget);
        }

        // Fast path for empty message: check cache under lock
        {
            if let Ok(state) = self.cache.lock() {
                let now = Instant::now();
                if !state.dirty && now.duration_since(state.cached_at) < state.cache_ttl && !state.cached_prompt.is_empty() {
                    return state.cached_prompt.clone();
                }
            }
        }

        // Cache miss: compute prompt without holding lock
        let prompt = assemble(&self.graph, self.store.as_deref(), "", effective_max_tokens, self.core_node_budget);
        let now = Instant::now();

        if let Ok(mut state) = self.cache.lock() {
            state.dirty = false;
            state.cached_prompt = prompt.clone();
            state.cached_at = now;
        }

        prompt
    }
}

fn clean_node_content(content: &str) -> String {
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Target:") || trimmed.starts_with("Linked:") {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n").trim().to_string()
}

fn assemble(
    graph: &Dendrite,
    store: Option<&DendriteStore>,
    user_message: &str,
    max_tokens: usize,
    core_budget_ratio: f64,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut used: usize = 0;
    let core_budget = ((max_tokens as f64) * core_budget_ratio) as usize;

    // Always include core identity nodes first.
    for id in CORE_IDS {
        let node = match graph.get(id) {
            Some(n) => n,
            None => continue,
        };
        let cleaned = clean_node_content(&node.content);
        if cleaned.is_empty() {
            continue;
        }
        let part = format!("## {}\n\n{}", node.title, cleaned);
        let cost = estimate_tokens(&part);
        if used + cost > core_budget {
            break;
        }
        parts.push(part);
        used += cost;
    }

    if !user_message.trim().is_empty() {
        // Conversation-relevant nodes (excluding TurnLogs to avoid prompt pollution).
        let candidates = find_relevant(graph, store, user_message);
        let scored = score(&candidates, user_message);
        for (node, _score) in scored {
            if CORE_IDS.contains(&node.id.as_str()) || node.node_type == NodeType::TurnLog {
                continue; // Skip core (already added) and ephemeral turn logs
            }
            let cleaned = clean_node_content(&node.content);
            if cleaned.is_empty() {
                continue;
            }
            let part = format!("## {}\n\n{}", node.title, cleaned);
            let cost = estimate_tokens(&part);
            if used + cost > max_tokens {
                break;
            }
            parts.push(part);
            used += cost;
        }
    } else {
        // No message context: recently updated non-core nodes.
        for node in graph.all() {
            if CORE_IDS.contains(&node.id.as_str()) || node.node_type == NodeType::TurnLog {
                continue;
            }
            let cleaned = clean_node_content(&node.content);
            if cleaned.is_empty() {
                continue;
            }
            let part = format!("## {}\n\n{}", node.title, cleaned);
            let cost = estimate_tokens(&part);
            if used + cost > max_tokens {
                break;
            }
            parts.push(part);
            used += cost;
        }
    }

    let prompt = parts.join("\n\n");
    format!(
        "=== CYNAPSE SYSTEM PRESET ===\n\
        1. You are CYNAPSE — a local-first, modular, precise AI companion.\n\
        2. Lead with the answer or immediate action on line 1. No greetings, preambles, or 'Great question!' openers.\n\
        3. Number multi-step tasks clearly. Cap lists at maximum 5 items.\n\
        4. End with exactly one concrete next action. No closers like 'Hope this helps!' or 'Let me know if you need anything else'.\n\
        5. State cause and fix directly for errors. Be concise and brief.\n\
        6. Never repeat system headers, section dividers, or internal tokens. Stop generation immediately when the response is complete.\n\n\
        === DENDRITE KNOWLEDGE CONTEXT ===\n\
        {prompt}"
    )
}

fn find_relevant(graph: &Dendrite, store: Option<&DendriteStore>, user_message: &str) -> Vec<Node> {
    let mut seen = HashSet::new();
    let mut out: Vec<Node> = Vec::new();

    let add_node = |n: &Node, out: &mut Vec<Node>, seen: &mut HashSet<String>| {
        if !seen.contains(&n.id) {
            seen.insert(n.id.clone());
            if !n.content.trim().is_empty() {
                out.push(n.clone());
            }
        }
    };

    let add_with_neighbors = |n: &Node, out: &mut Vec<Node>, seen: &mut HashSet<String>| {
        add_node(n, out, seen);
        for neighbor in graph.neighbors_2hop(&n.id) {
            add_node(&neighbor, out, seen);
        }
    };

    // 1. Try FTS5 first (most precise, if available).
    if let Some(s) = store {
        if let Ok(ids) = s.fts_search(user_message, 10) {
            for id in ids {
                if let Some(n) = graph.get(&id) {
                    add_with_neighbors(&n, &mut out, &mut seen);
                }
            }
        }
    }

    // 2. Full-query substring search (fallback / complement).
    for n in graph.search(user_message) {
        add_with_neighbors(&n, &mut out, &mut seen);
    }

    // 3. Word-by-word search in titles, content, and tags.
    for word in user_message.to_lowercase().split_whitespace() {
        if word.chars().count() < 3 || is_stop_word(word) {
            continue;
        }
        for n in graph.search(word) {
            add_with_neighbors(&n, &mut out, &mut seen);
        }
        for n in graph.by_tag(word) {
            add_node(&n, &mut out, &mut seen);
        }
        // Hard cap — stop expanding once we have enough candidates.
        if out.len() >= MAX_CANDIDATES {
            break;
        }
    }

    out.truncate(MAX_CANDIDATES);
    out
}

type ScoredNode = (Node, f64);

fn score(nodes: &[Node], query: &str) -> Vec<ScoredNode> {
    let q = query.to_lowercase();
    let query_words: Vec<&str> = q.split_whitespace().filter(|w| !is_stop_word(w)).collect();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut scored: Vec<ScoredNode> = nodes
        .iter()
        .map(|n| {
            let mut bm25_score = 0.0;

            if n.title.to_lowercase().contains(&q) {
                bm25_score += 15.0;
            }
            for qw in &query_words {
                if n.title.to_lowercase().contains(qw) {
                    bm25_score += 5.0;
                }
                bm25_score += count_occurrences(&n.content.to_lowercase(), qw) as f64 * 2.0;
            }

            // Recency decay gamma^(delta_t) (0.95 decay per day)
            let age_days = (now - n.updated_at).max(0) as f64 / 86400.0;
            let recency_decay = 0.95f64.powf(age_days);

            // Specialization index boost spec(e)
            let spec_boost = n.spec_index() as f64;

            // Final 2-tier score combining lexical BM25 + specialization + recency decay
            let mut final_score = (bm25_score * recency_decay) + (spec_boost * 4.0);

            // Connectivity bonus — hub nodes carry more weight
            final_score += (n.links.len() + n.backlinks.len()) as f64 * 0.3;

            // Node type priority
            match n.node_type {
                NodeType::Identity => final_score += 10.0,
                NodeType::Person => final_score += 5.0,
                NodeType::Project => final_score += 3.0,
                _ => {}
            }

            (n.clone(), final_score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

/// Token estimate considering character count, word count, and UTF-8 bytes.
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();
    ((char_count / 4) + (word_count / 2)).max(1)
}

fn stop_words() -> &'static HashSet<&'static str> {
    static WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    WORDS.get_or_init(|| {
        [
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was",
            "one", "our", "out", "day", "get", "has", "him", "his", "how", "its", "may", "new",
            "now", "old", "see", "two", "who", "boy", "did", "she", "use", "way", "many", "oil",
            "sit", "set", "run", "eat", "far", "sea", "eye", "ago", "off", "too", "any", "say",
            "man", "try", "ask", "end", "why", "let", "put", "own", "tell", "when", "come", "here",
            "just", "like", "long", "make", "over", "such", "take", "than", "them", "well", "were",
            "what", "will", "with", "have", "from", "they", "know", "want", "been", "good", "much",
            "some", "time", "would", "there", "their", "could", "other", "after", "first", "never",
            "these", "think", "where", "being", "every", "great", "might", "shall", "still",
            "those", "while", "about", "should",
        ]
        .into_iter()
        .collect()
    })
}

fn is_stop_word(w: &str) -> bool {
    stop_words().contains(w)
}
