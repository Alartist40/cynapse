//! DENDRITE — the in-memory knowledge graph.
//!
//! Faithful port of Go `internal/memory/dendrite.go`. Nodes carry
//! wiki-links (`[[target]]`) and hashtags (`#tag`) that are parsed from
//! content; backlinks are auto-wired and kept in sync on every mutation.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Mutex, MutexGuard, OnceLock};

use regex::Regex;

fn link_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap())
}

fn tag_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"#([A-Za-z0-9_-]+)").unwrap())
}

fn fenced_code_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)```.*?```").unwrap())
}

fn inline_code_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"`[^`\n]*`").unwrap())
}

/// Strip Markdown fenced (```) and inline (`) code spans before running the
/// wiki-link / hashtag regexes over content, so illustrative syntax like
/// `` `[[id]]` `` in documentation text isn't parsed as a real link. Not a
/// full Markdown parser — just enough to avoid the common case of code
/// spans quoting the very syntax this module looks for.
fn strip_code_spans(content: &str) -> String {
    let without_fences = fenced_code_pattern().replace_all(content, " ");
    inline_code_pattern().replace_all(&without_fences, " ").into_owned()
}

/// Classifies what kind of knowledge a node holds across the 4-tier memory model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Core self / agent persona (L3)
    Identity,
    /// A real person (user, contact) (L1 / L3)
    Person,
    /// Abstract concept or topic (L2)
    Concept,
    /// A project or task (L2)
    Project,
    /// Procedural skill or workflow (L2)
    Procedure,
    /// Something that happened / event (L1)
    Event,
    /// Atomic fact or preference (L1)
    AtomicFact,
    /// Raw chat turn log (L0)
    TurnLog,
    /// Episodic memory entry (L1)
    Memory,
    /// User-defined
    Custom,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Identity => "identity",
            NodeType::Person => "person",
            NodeType::Concept => "concept",
            NodeType::Project => "project",
            NodeType::Procedure => "procedure",
            NodeType::Event => "event",
            NodeType::AtomicFact => "atomic_fact",
            NodeType::TurnLog => "turn_log",
            NodeType::Memory => "memory",
            NodeType::Custom => "custom",
        }
    }

    pub fn label(&self) -> &'static str {
        self.as_str()
    }

    pub fn tier(&self) -> u8 {
        match self {
            NodeType::TurnLog => 0,
            NodeType::AtomicFact | NodeType::Memory | NodeType::Event | NodeType::Person => 1,
            NodeType::Procedure | NodeType::Project | NodeType::Concept => 2,
            NodeType::Identity => 3,
            NodeType::Custom => 1,
        }
    }

    pub fn from_str(s: &str) -> NodeType {
        match s {
            "identity" => NodeType::Identity,
            "person" => NodeType::Person,
            "concept" => NodeType::Concept,
            "project" => NodeType::Project,
            "procedure" => NodeType::Procedure,
            "event" => NodeType::Event,
            "atomic_fact" => NodeType::AtomicFact,
            "turn_log" => NodeType::TurnLog,
            "memory" => NodeType::Memory,
            _ => NodeType::Custom,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single knowledge node in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: NodeType,
    pub tags: Vec<String>,
    /// Outgoing [[links]]
    pub links: Vec<String>,
    /// Auto-maintained incoming links
    pub backlinks: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Node {
    /// Create a minimal placeholder for a node that is referenced by a
    /// `[[link]]` but has no content yet.  Currently unused — placeholder
    /// creation was removed from `upsert` to prevent ghost nodes, but
    /// this helper is kept for potential future use (e.g. lazy resolution).
    #[allow(dead_code)]
    pub fn placeholder(id: String, now: i64) -> Node {
        Node {
            title: id.clone(),
            id,
            content: String::new(),
            node_type: NodeType::Custom,
            tags: Vec::new(),
            links: Vec::new(),
            backlinks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Default)]
struct DendriteInner {
    nodes: HashMap<String, Node>,
    on_change: HashMap<u64, std::sync::Arc<dyn Fn() + Send + Sync>>,
    next_cb_id: u64,
}

/// The in-memory graph. Thread-safe: all mutations take an internal lock,
/// and `on_change` callbacks are invoked after the lock is released.
pub struct Dendrite {
    inner: Mutex<DendriteInner>,
}

fn lock_inner(inner: &Mutex<DendriteInner>) -> MutexGuard<'_, DendriteInner> {
    inner.lock().unwrap_or_else(|e| e.into_inner())
}

impl Default for Dendrite {
    fn default() -> Self {
        Self::new()
    }
}

impl Dendrite {
    pub fn new() -> Dendrite {
        Dendrite {
            inner: Mutex::new(DendriteInner::default()),
        }
    }

    /// Register a callback invoked on every mutation. Returns a numeric ID
    /// that can be passed to `unregister_on_change` to remove it later.
    pub fn register_on_change(&self, cb: std::sync::Arc<dyn Fn() + Send + Sync>) -> u64 {
        let mut inner = lock_inner(&self.inner);
        let id = inner.next_cb_id;
        inner.next_cb_id += 1;
        inner.on_change.insert(id, cb);
        id
    }

    /// Register a callback with a specific ID (used by DendriteContext to
    /// coordinate cleanup via `ChangeGuard`).
    pub fn register_on_change_with_id(&self, id: u64, cb: std::sync::Arc<dyn Fn() + Send + Sync>) {
        let mut inner = lock_inner(&self.inner);
        inner.on_change.insert(id, cb);
    }

    /// Remove a previously registered callback by its ID.
    pub fn unregister_on_change(&self, id: u64) {
        let mut inner = lock_inner(&self.inner);
        inner.on_change.remove(&id);
    }

    fn notify(&self) {
        let callbacks: Vec<_> = {
            let inner = lock_inner(&self.inner);
            inner.on_change.values().cloned().collect()
        };
        for cb in callbacks {
            cb();
        }
    }

    /// Insert a hydrated node directly (used by the store on load). Skips
    /// backlink recalculation because backlinks are already stored.
    pub fn insert_hydrated(&self, node: Node) {
        lock_inner(&self.inner).nodes.insert(node.id.clone(), node);
    }

    /// Create or fully replace a node, re-wiring all backlinks.
    pub fn upsert(
        &self,
        id: &str,
        title: &str,
        content: &str,
        node_type: NodeType,
        tags: Option<Vec<String>>,
    ) -> Node {
        let now = timestamp();
        let links = parse_links(content);
        let tags = tags.unwrap_or_else(|| parse_tags(content));

        let mut inner = lock_inner(&self.inner);

        // Remove old backlinks from the previous version of this node.
        let old_links: Vec<String> = inner
            .nodes
            .get(id)
            .map(|old| old.links.clone())
            .unwrap_or_default();
        for old_link in &old_links {
            if let Some(target) = inner.nodes.get_mut(old_link) {
                target.backlinks = remove_str(&target.backlinks, id);
            }
        }

        let node = match inner.nodes.entry(id.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                let n = e.get_mut();
                n.title = title.to_string();
                n.content = content.to_string();
                n.node_type = node_type;
                n.tags = tags;
                n.links = links.clone();
                n.updated_at = now;
                n.clone()
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                let n = Node {
                    id: id.to_string(),
                    title: title.to_string(),
                    content: content.to_string(),
                    node_type,
                    tags,
                    links: links.clone(),
                    backlinks: Vec::new(),
                    created_at: now,
                    updated_at: now,
                };
                e.insert(n.clone());
                n
            }
        };

        // Wire new backlinks — only for targets that already exist in the
        // graph.  We no longer create placeholder nodes for unresolved
        // `[[links]]` because documentation code spans and hypothetical
        // references were populating the graph with empty ghost nodes.
        for link in &links {
            if let Some(target) = inner.nodes.get_mut(link) {
                if !target.backlinks.contains(&node.id) {
                    target.backlinks.push(node.id.clone());
                }
            }
        }

        drop(inner);
        self.notify();
        node
    }

    /// Delete a node and clean up all references.
    pub fn delete(&self, id: &str) -> bool {
        let mut inner = lock_inner(&self.inner);
        let node = match inner.nodes.get(id) {
            Some(n) => n.clone(),
            None => return false,
        };

        for link in &node.links {
            if let Some(target) = inner.nodes.get_mut(link) {
                target.backlinks = remove_str(&target.backlinks, id);
            }
        }
        for n in inner.nodes.values_mut() {
            n.links = remove_str(&n.links, id);
        }

        inner.nodes.remove(id);
        drop(inner);
        self.notify();
        true
    }

    pub fn get(&self, id: &str) -> Option<Node> {
        lock_inner(&self.inner).nodes.get(id).cloned()
    }

    /// All nodes sorted by UpdatedAt descending.
    pub fn all(&self) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        let mut nodes: Vec<Node> = inner.nodes.values().cloned().collect();
        nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        nodes
    }

    /// Return all nodes belonging to a specific memory tier (0..3).
    pub fn by_tier(&self, tier: u8) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        let mut nodes: Vec<Node> = inner
            .nodes
            .values()
            .filter(|n| n.node_type.tier() == tier)
            .cloned()
            .collect();
        nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        nodes
    }

    /// Return all nodes and edge connections (source_id, target_id) in the graph.
    pub fn topology(&self) -> (Vec<Node>, Vec<(String, String)>) {
        let inner = lock_inner(&self.inner);
        let mut nodes: Vec<Node> = inner.nodes.values().cloned().collect();
        nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        let mut edges = Vec::new();
        for node in &nodes {
            for link in &node.links {
                if inner.nodes.contains_key(link) {
                    edges.push((node.id.clone(), link.clone()));
                }
            }
        }
        (nodes, edges)
    }

    /// Fast in-memory BM25 term relevance search.
    pub fn search_bm25(&self, query: &str, limit: usize) -> Vec<(Node, f32)> {
        let inner = lock_inner(&self.inner);
        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|s| s.len() >= 2)
            .map(|s| s.to_string())
            .collect();

        if query_tokens.is_empty() {
            return Vec::new();
        }

        let total_docs = inner.nodes.len() as f32;
        if total_docs == 0.0 {
            return Vec::new();
        }

        // Compute document frequency for each query token (how many docs contain it).
        let mut doc_freq: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        for node in inner.nodes.values() {
            let doc_text = format!("{} {} {}", node.title, node.content, node.tags.join(" ")).to_lowercase();
            for token in &query_tokens {
                if doc_text.contains(token) {
                    *doc_freq.entry(token.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        let mut scored: Vec<(Node, f32)> = Vec::new();
        let k1 = 1.2f32;
        let b = 0.75f32;

        let avg_len = inner
            .nodes
            .values()
            .map(|n| {
                let doc_text = format!("{} {} {}", n.title, n.content, n.tags.join(" "));
                doc_text.len() as f32
            })
            .sum::<f32>()
            / total_docs.max(1.0);

        for node in inner.nodes.values() {
            let doc_text = format!("{} {} {}", node.title, node.content, node.tags.join(" ")).to_lowercase();
            let doc_len = doc_text.len() as f32;
            let mut score = 0.0f32;

            for token in &query_tokens {
                let count = doc_text.matches(token).count() as f32;
                if count > 0.0 {
                    let df = doc_freq.get(token).copied().unwrap_or(0.0);
                    // Standard BM25 IDF: ln((N - df + 0.5) / (df + 0.5))
                    let idf = ((total_docs - df + 0.5) / (df + 0.5)).max(0.0001).ln();
                    let tf = (count * (k1 + 1.0)) / (count + k1 * (1.0 - b + b * (doc_len / avg_len.max(1.0))));
                    score += idf * tf;
                }
            }

            if score > 0.0 {
                scored.push((node.clone(), score));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if limit > 0 && scored.len() > limit {
            scored.truncate(limit);
        }
        scored
    }

    /// 1-hop neighborhood (links + backlinks combined).
    pub fn neighbors(&self, id: &str) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        let node = match inner.nodes.get(id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        let mut seen = HashSet::new();
        seen.insert(id.to_string());
        let mut out = Vec::new();

        for lid in &node.links {
            if !seen.contains(lid) {
                if let Some(n) = inner.nodes.get(lid) {
                    out.push(n.clone());
                    seen.insert(lid.clone());
                }
            }
        }
        for bid in &node.backlinks {
            if !seen.contains(bid) {
                if let Some(n) = inner.nodes.get(bid) {
                    out.push(n.clone());
                    seen.insert(bid.clone());
                }
            }
        }
        out
    }

    /// Nodes within 2 hops (BFS).
    pub fn neighbors_2hop(&self, id: &str) -> Vec<Node> {
        self.neighbors_nhop(id, 2)
    }

    /// Nodes within N hops (BFS).
    fn neighbors_nhop(&self, id: &str, max_depth: usize) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        if !inner.nodes.contains_key(id) {
            return Vec::new();
        }

        let mut seen = HashSet::new();
        seen.insert(id.to_string());
        let mut out = Vec::new();

        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        queue.push_back((id.to_string(), 0));
        while let Some((node_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            let n = match inner.nodes.get(&node_id) {
                Some(n) => n,
                None => continue,
            };
            let next_depth = depth + 1;
            for lid in &n.links {
                if !seen.contains(lid) {
                    seen.insert(lid.clone());
                    if let Some(target) = inner.nodes.get(lid) {
                        out.push(target.clone());
                    }
                    queue.push_back((lid.clone(), next_depth));
                }
            }
            for bid in &n.backlinks {
                if !seen.contains(bid) {
                    seen.insert(bid.clone());
                    if let Some(target) = inner.nodes.get(bid) {
                        out.push(target.clone());
                    }
                    queue.push_back((bid.clone(), next_depth));
                }
            }
        }
        out
    }

    /// Nodes within 3 hops (BFS).
    pub fn neighbors_3hop(&self, id: &str) -> Vec<Node> {
        self.neighbors_nhop(id, 3)
    }

    /// Nodes whose title, content, or tags contain the query (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<Node> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let inner = lock_inner(&self.inner);
        inner
            .nodes
            .values()
            .filter(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.content.to_lowercase().contains(&q)
                    || contains_str_fold(&n.tags, &q)
            })
            .cloned()
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<Node> {
        let inner = lock_inner(&self.inner);
        inner
            .nodes
            .values()
            .filter(|n| contains_str_fold(&n.tags, tag))
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        lock_inner(&self.inner).nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Normalise a wiki-link target into a stable lowercase_underscore ID.
fn to_node_id(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "_")
}

pub(crate) fn parse_links(content: &str) -> Vec<String> {
    let content = strip_code_spans(content);
    let mut seen = HashSet::new();
    let mut links = Vec::new();
    for caps in link_pattern().captures_iter(&content) {
        if let Some(m) = caps.get(1) {
            let id = to_node_id(m.as_str());
            if seen.insert(id.clone()) {
                links.push(id);
            }
        }
    }
    links
}

pub(crate) fn parse_tags(content: &str) -> Vec<String> {
    let content = strip_code_spans(content);
    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for caps in tag_pattern().captures_iter(&content) {
        if let Some(m) = caps.get(1) {
            let t = m.as_str().to_string();
            if seen.insert(t.clone()) {
                tags.push(t);
            }
        }
    }
    tags
}

pub(crate) fn contains_str_fold(slice: &[String], item: &str) -> bool {
    slice.iter().any(|s| s.eq_ignore_ascii_case(item))
}

pub(crate) fn remove_str(slice: &[String], item: &str) -> Vec<String> {
    slice.iter().filter(|s| *s != item).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_links_ignores_inline_code_spans() {
        // Reproduces the exact TOOLS.md sentence that used to create a
        // bogus permanent placeholder node named "id".
        let content = "Wiki-links `[[id]]` and `#tags` are parsed automatically.";
        assert!(parse_links(content).is_empty());
        assert!(parse_tags(content).is_empty());
    }

    #[test]
    fn parse_links_ignores_fenced_code_blocks() {
        let content = "See below:\n```\n[[fenced_link]] #fenced_tag\n```\nReal link: [[real_target]]";
        assert_eq!(parse_links(content), vec!["real_target".to_string()]);
    }

    #[test]
    fn parse_links_still_finds_real_links_and_tags_outside_code_spans() {
        let content = "Refer to [[identity]] and [[soul]] for #personality guidance.";
        assert_eq!(
            parse_links(content),
            vec!["identity".to_string(), "soul".to_string()]
        );
        assert_eq!(parse_tags(content), vec!["personality".to_string()]);
    }

    #[test]
    fn upsert_does_not_create_placeholder_from_documentation_code_span() {
        let g = Dendrite::new();
        g.upsert(
            "tools",
            "Tools",
            "Wiki-links `[[id]]` and `#tags` are parsed automatically.",
            NodeType::Concept,
            None,
        );
        assert!(g.get("id").is_none(), "code-span example must not create a real node");
    }
}
