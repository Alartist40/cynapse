//! DENDRITE system-prompt assembly.
//!
//! Faithful port of Go `internal/memory/dendrite_context.go`: budgets,
//! relevance discovery, scoring, and the 5-minute prompt cache.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::graph::{Node, NodeType, Dendrite};
use crate::store::DendriteStore;

const DEFAULT_MAX_TOKENS: usize = 6000;
/// 40% of the token budget for core identity nodes.
const CORE_NODE_BUDGET: f64 = 0.40;
/// Maximum number of candidate nodes returned by `find_relevant` before
/// scoring.  Keeps the 2-hop expansion from blowing up the system prompt
/// on large graphs.
const MAX_CANDIDATES: usize = 50;

/// Core identity nodes always included first.
const CORE_IDS: [&str; 4] = ["identity", "soul", "agents", "tools"];

struct ContextInner {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
    cached_prompt: String,
    cached_at: Instant,
    cache_ttl: std::time::Duration,
    dirty: bool,
    next_cb_id: u64,
    callbacks: std::collections::HashMap<u64, std::sync::Arc<dyn Fn() + Send + Sync>>,
}

/// Handle returned by `register_on_change`. Dropping it unregisters the
/// callback, preventing leaked Arcs if contexts are created and dropped
/// repeatedly.
pub struct ChangeGuard {
    id: u64,
    graph: Arc<Dendrite>,
}

impl Drop for ChangeGuard {
    fn drop(&mut self) {
        self.graph.unregister_on_change(self.id);
    }
}

/// Assembles the LLM system prompt from graph nodes.
pub struct DendriteContext {
    inner: Arc<Mutex<ContextInner>>,
}

impl DendriteContext {
    pub fn new(graph: Arc<Dendrite>, store: Option<Arc<DendriteStore>>) -> Arc<DendriteContext> {
        let inner = Arc::new(Mutex::new(ContextInner {
            graph,
            store,
            cached_prompt: String::new(),
            cached_at: Instant::now(),
            cache_ttl: std::time::Duration::from_secs(300),
            dirty: true,
            next_cb_id: 0,
            callbacks: std::collections::HashMap::new(),
        }));

        // Whenever the graph mutates, mark the cache dirty.
        let weak = Arc::downgrade(&inner);
        let cb_id = {
            let mut i = inner.lock().unwrap_or_else(|e| e.into_inner());
            let id = i.next_cb_id;
            i.next_cb_id += 1;
            let cb: std::sync::Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                if let Some(inner) = weak.upgrade() {
                    if let Ok(mut i) = inner.lock() {
                        i.dirty = true;
                    }
                }
            });
            i.callbacks.insert(id, cb.clone());
            id
        };

        {
            let i = inner.lock().unwrap_or_else(|e| e.into_inner());
            i.graph.register_on_change_with_id(cb_id, i.callbacks[&cb_id].clone());
        }

        Arc::new(DendriteContext { inner })
    }

    /// Return the system prompt. If `user_message` is non-empty it biases
    /// context toward relevant nodes; otherwise a cached general prompt.
    pub fn build_prompt(&self, user_message: &str, max_tokens: usize) -> String {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let max_tokens = if max_tokens == 0 {
            DEFAULT_MAX_TOKENS
        } else {
            max_tokens
        };

        // Message-specific context always recomputes — do NOT clear dirty
        // here because we haven't refreshed the cache.
        if !user_message.trim().is_empty() {
            return assemble(&inner, user_message, max_tokens);
        }

        let now = Instant::now();
        if !inner.dirty && now.duration_since(inner.cached_at) < inner.cache_ttl && !inner.cached_prompt.is_empty() {
            return inner.cached_prompt.clone();
        }

        inner.dirty = false;
        let prompt = assemble(&inner, "", max_tokens);
        inner.cached_prompt = prompt.clone();
        inner.cached_at = now;
        prompt
    }
}

fn assemble(inner: &ContextInner, user_message: &str, max_tokens: usize) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut used: usize = 0;
    let core_budget = ((max_tokens as f64) * CORE_NODE_BUDGET) as usize;

    // Always include core identity nodes first.
    for id in CORE_IDS {
        let node = match inner.graph.get(id) {
            Some(n) => n,
            None => continue,
        };
        let part = format!("## {}\n\n{}", node.title, node.content);
        let cost = estimate_tokens(&part);
        if used + cost > core_budget {
            break;
        }
        parts.push(part);
        used += cost;
    }

    if !user_message.trim().is_empty() {
        // Conversation-relevant nodes.
        let candidates = find_relevant(inner, user_message);
        let scored = score(&candidates, user_message);
        for (node, _score) in scored {
            if CORE_IDS.contains(&node.id.as_str()) {
                continue; // already included
            }
            let part = format!("## {}\n\n{}", node.title, node.content);
            let cost = estimate_tokens(&part);
            if used + cost > max_tokens {
                break;
            }
            parts.push(part);
            used += cost;
        }
    } else {
        // No message context: recently updated non-core nodes.
        for node in inner.graph.all() {
            if CORE_IDS.contains(&node.id.as_str()) {
                continue;
            }
            let part = format!("## {}\n\n{}", node.title, node.content);
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
        "{prompt}\n\n## System Instructions & Protocol\n\
        1. You are CYNAPSE — a local-first, modular, precise AI companion.\n\
        2. Lead with the answer or immediate action on line 1. No greetings, preambles, or 'Great question!' openers.\n\
        3. Number multi-step tasks clearly. Cap lists at maximum 5 items.\n\
        4. End with exactly one concrete next action. No closers like 'Hope this helps!' or 'Let me know if you need anything else'.\n\
        5. State cause and fix directly for errors. Be concise and brief.\n\
        6. Never repeat system headers, section dividers, or internal tokens. Stop generation immediately when the response is complete."
    )
}

fn find_relevant(inner: &ContextInner, user_message: &str) -> Vec<Node> {
    let mut seen = HashSet::new();
    let mut out: Vec<Node> = Vec::new();

    let add_node = |n: &Node, out: &mut Vec<Node>, seen: &mut HashSet<String>| {
        if !seen.contains(&n.id) {
            seen.insert(n.id.clone());
            // Defense in depth against orphan placeholder nodes (e.g. an
            // unresolved `[[link]]` target) that carry no real content and
            // would otherwise waste context budget and clutter results.
            if !n.content.trim().is_empty() {
                out.push(n.clone());
            }
        }
    };

    let add_with_neighbors = |n: &Node, out: &mut Vec<Node>, seen: &mut HashSet<String>| {
        add_node(n, out, seen);
        for neighbor in inner.graph.neighbors_2hop(&n.id) {
            add_node(&neighbor, out, seen);
        }
    };

    // 1. Try FTS5 first (most precise, if available).
    if let Some(store) = &inner.store {
        if let Ok(ids) = store.fts_search(user_message, 10) {
            for id in ids {
                if let Some(n) = inner.graph.get(&id) {
                    add_with_neighbors(&n, &mut out, &mut seen);
                }
            }
        }
    }

    // 2. Full-query substring search (fallback / complement).
    for n in inner.graph.search(user_message) {
        add_with_neighbors(&n, &mut out, &mut seen);
    }

    // 3. Word-by-word search in titles, content, and tags.
    for word in user_message.to_lowercase().split_whitespace() {
        if word.chars().count() < 3 || is_stop_word(word) {
            continue;
        }
        for n in inner.graph.search(word) {
            add_with_neighbors(&n, &mut out, &mut seen);
        }
        for n in inner.graph.by_tag(word) {
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
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut scored: Vec<ScoredNode> = nodes
        .iter()
        .map(|n| {
            let mut s = 0.0;

            if n.title.to_lowercase().contains(&q) {
                s += 15.0;
            }
            s += count_occurrences(&n.content.to_lowercase(), &q) as f64 * 2.0;

            // Recency boost (linear decay, max 5 points over 7 days).
            let age = (now - n.updated_at) as f64 / 86400.0;
            if age < 7.0 {
                s += (7.0 - age) * (5.0 / 7.0);
            }

            // Connectivity bonus — hub nodes carry more weight.
            s += (n.links.len() + n.backlinks.len()) as f64 * 0.3;

            // Node type priority.
            match n.node_type {
                NodeType::Identity => s += 10.0,
                NodeType::Person => s += 5.0,
                NodeType::Project => s += 3.0,
                _ => {}
            }

            (n.clone(), s)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

/// Count non-overlapping occurrences of `needle` in `haystack`.
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

/// Rough token estimate (1 token ≈ 4 chars). Delegates to the same shared
/// estimator the compressor uses so tuning one tunes both instead of
/// drifting apart.
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
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
