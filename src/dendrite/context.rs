use std::collections::HashSet;

use super::{Dendrite, DendriteStore, Node, NodeType};

const DEFAULT_MAX_TOKENS: usize = 6000;
const CORE_NODE_BUDGET: f64 = 0.40;

/// Assembles the LLM system prompt from graph nodes.
pub struct DendriteContext {
    graph: Dendrite,
    store: Option<DendriteStore>,
}

impl DendriteContext {
    pub fn new(graph: Dendrite, store: Option<DendriteStore>) -> Self {
        Self { graph, store }
    }

    /// Build a system prompt. If user_message is non-empty, bias toward relevant nodes.
    pub fn build_prompt(&self, user_message: &str, max_tokens: usize) -> String {
        let max_tokens = if max_tokens == 0 {
            DEFAULT_MAX_TOKENS
        } else {
            max_tokens
        };

        if !user_message.trim().is_empty() {
            self.assemble(user_message, max_tokens)
        } else {
            self.assemble("", max_tokens)
        }
    }

    fn assemble(&self, user_message: &str, max_tokens: usize) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut used = 0usize;

        let core_budget = (max_tokens as f64 * CORE_NODE_BUDGET) as usize;

        // Always include core identity nodes first
        let core_ids = ["identity", "soul", "agents", "tools"];
        for id in &core_ids {
            if let Some(node) = self.graph.get(id) {
                let part = format!("## {}\n\n{}", node.title, node.content);
                let cost = estimate_tokens(&part);
                if used + cost > core_budget {
                    break;
                }
                parts.push(part);
                used += cost;
            }
        }

        // Add conversation-relevant nodes
        if !user_message.trim().is_empty() {
            let candidates = self.find_relevant(user_message);
            let scored = self.score(candidates, user_message);

            for sn in scored {
                if core_ids.contains(&sn.node.id.as_str()) {
                    continue;
                }
                let part = format!("## {}\n\n{}", sn.node.title, sn.node.content);
                let cost = estimate_tokens(&part);
                if used + cost > max_tokens {
                    break;
                }
                parts.push(part);
                used += cost;
            }
        } else {
            // No message context: add recently updated non-core nodes
            for node in self.graph.all() {
                if core_ids.contains(&node.id.as_str()) {
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

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n\n---\n\n")
        }
    }

    fn find_relevant(&self, user_message: &str) -> Vec<Node> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut out: Vec<Node> = Vec::new();

        let mut add_node = |n: Node| {
            if seen.insert(n.id.clone()) {
                out.push(n);
            }
        };

        // 1. Try FTS5 first
        if let Some(store) = &self.store {
            if let Ok(ids) = store.fts_search(user_message, 20) {
                for id in ids {
                    if let Some(n) = self.graph.get(&id) {
                        add_node(n.clone());
                        for neighbor in self.graph.neighbors(&id) {
                            add_node(neighbor);
                        }
                    }
                }
            }
        }

        // 2. Full-query substring search
        for n in self.graph.search(user_message) {
            let nid = n.id.clone();
            add_node(n);
            for neighbor in self.graph.neighbors(&nid) {
                add_node(neighbor);
            }
        }

        // 3. Word-by-word search
        for word in user_message.to_lowercase().split_whitespace() {
            if word.len() < 3 || is_stop_word(word) {
                continue;
            }
            for n in self.graph.search(word) {
                let nid = n.id.clone();
                add_node(n);
                for neighbor in self.graph.neighbors(&nid) {
                    add_node(neighbor);
                }
            }
            for n in self.graph.by_tag(word) {
                add_node(n);
            }
        }

        out
    }

    fn score(&self, nodes: Vec<Node>, query: &str) -> Vec<ScoredNode> {
        let q = query.to_lowercase();
        let now = chrono::Utc::now().timestamp();

        let mut scored: Vec<ScoredNode> = nodes
            .into_iter()
            .map(|n| {
                let mut s = 0.0f64;

                if n.title.to_lowercase().contains(&q) {
                    s += 15.0;
                }
                s += n.content.to_lowercase().matches(&q).count() as f64 * 2.0;

                // Recency boost (linear decay over 7 days)
                let age = (now - n.updated_at) as f64 / 86400.0;
                if age < 7.0 {
                    s += (7.0 - age) * (5.0 / 7.0);
                }

                // Connectivity bonus
                s += (n.links.len() + n.backlinks.len()) as f64 * 0.3;

                // Node type priority
                match n.node_type {
                    NodeType::Identity => s += 10.0,
                    NodeType::Person => s += 5.0,
                    NodeType::Project => s += 3.0,
                    _ => {}
                }

                ScoredNode { node: n, score: s }
            })
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored
    }
}

struct ScoredNode {
    node: Node,
    score: f64,
}

fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

fn is_stop_word(w: &str) -> bool {
    matches!(w,
        "the" | "and" | "for" | "are" | "but" | "not" | "you" | "all" | "can" | "had" |
        "her" | "was" | "one" | "our" | "out" | "day" | "get" | "has" | "him" | "his" |
        "how" | "its" | "may" | "new" | "now" | "old" | "see" | "two" | "who" | "boy" |
        "did" | "she" | "use" | "way" | "many" | "oil" | "sit" | "set" | "run" | "eat" |
        "far" | "sea" | "eye" | "ago" | "off" | "too" | "any" | "say" | "man" | "try" |
        "ask" | "end" | "why" | "let" | "put" | "own" | "tell" | "when" | "come" | "here" |
        "just" | "like" | "long" | "make" | "over" | "such" | "take" | "than" | "them" |
        "well" | "were" | "what" | "will" | "with" | "have" | "from" | "they" | "know" |
        "want" | "been" | "good" | "much" | "some" | "time" | "would" | "there" | "their" |
        "could" | "other" | "after" | "first" | "never" | "these" | "think" | "where" |
        "being" | "every" | "great" | "might" | "shall" | "still" | "those" | "while" |
        "about" | "should"
    )
}
