use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Classification of a knowledge node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Identity,
    Person,
    Concept,
    Project,
    Event,
    Memory,
    Custom,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Identity => "identity",
            NodeType::Person => "person",
            NodeType::Concept => "concept",
            NodeType::Project => "project",
            NodeType::Event => "event",
            NodeType::Memory => "memory",
            NodeType::Custom => "custom",
        }
    }
}

impl std::str::FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "identity" => Ok(NodeType::Identity),
            "person" => Ok(NodeType::Person),
            "concept" => Ok(NodeType::Concept),
            "project" => Ok(NodeType::Project),
            "event" => Ok(NodeType::Event),
            "memory" => Ok(NodeType::Memory),
            "custom" => Ok(NodeType::Custom),
            _ => Err(format!("unknown node type: {}", s)),
        }
    }
}

/// A single knowledge node in the graph.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: NodeType,
    pub tags: Vec<String>,
    pub links: Vec<String>,      // outgoing [[links]]
    pub backlinks: Vec<String>,  // auto-maintained incoming
    pub created_at: i64,
    pub updated_at: i64,
}

impl Node {
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            node_type: NodeType::Custom,
            tags: Vec::new(),
            links: Vec::new(),
            backlinks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// In-memory knowledge graph. All operations are thread-safe.
pub struct Dendrite {
    pub(crate) nodes: Arc<RwLock<HashMap<String, Node>>>,
    link_re: Regex,
    tag_re: Regex,
}

impl Default for Dendrite {
    fn default() -> Self {
        Self::new()
    }
}

impl Dendrite {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            link_re: Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap(),
            tag_re: Regex::new(r"#([A-Za-z0-9_-]+)").unwrap(),
        }
    }

    /// Create or fully replace a node and re-wire all backlinks.
    pub fn upsert(
        &self,
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        node_type: NodeType,
        tags: Option<Vec<String>>,
    ) -> Node {
        let id = id.into();
        let title = title.into();
        let content = content.into();
        let now = chrono::Utc::now().timestamp();

        let links = self.parse_links(&content);
        let tags = tags.unwrap_or_else(|| self.parse_tags(&content));

        // Collect old links before acquiring write lock for removal
        let old_links: Vec<String> = {
            let guard = self.nodes.read().unwrap();
            guard.get(&id).map(|n| n.links.clone()).unwrap_or_default()
        };

        let mut guard = self.nodes.write().unwrap();

        // Remove old backlinks
        for old_link in &old_links {
            if let Some(target) = guard.get_mut(old_link) {
                target.backlinks.retain(|b| b != &id);
            }
        }

        let node = guard.entry(id.clone()).or_insert_with(|| Node {
            id: id.clone(),
            title: title.clone(),
            content: content.clone(),
            node_type: node_type.clone(),
            tags: tags.clone(),
            links: links.clone(),
            backlinks: Vec::new(),
            created_at: now,
            updated_at: now,
        });

        node.title = title;
        node.content = content;
        node.node_type = node_type;
        node.tags = tags;
        node.links = links.clone();
        node.updated_at = now;

        // Collect links to wire (clone to avoid borrow issues)
        let links_to_wire: Vec<String> = links;
        drop(guard);

        // Wire new backlinks in a separate lock
        let mut guard = self.nodes.write().unwrap();
        for link in &links_to_wire {
            let target = guard.entry(link.clone()).or_insert_with(|| {
                Node::new(link.clone(), link.clone(), "")
            });
            if !target.backlinks.contains(&id) {
                target.backlinks.push(id.clone());
            }
        }

        guard.get(&id).cloned().unwrap()
    }

    /// Remove a node and clean up all graph references.
    pub fn delete(&self, id: &str) -> bool {
        let mut guard = self.nodes.write().unwrap();

        let node = match guard.get(id) {
            Some(n) => n.clone(),
            None => return false,
        };

        for link in &node.links {
            if let Some(target) = guard.get_mut(link) {
                target.backlinks.retain(|b| b != id);
            }
        }

        for other in guard.values_mut() {
            other.links.retain(|l| l != id);
        }

        guard.remove(id).is_some()
    }

    /// Get a node by ID.
    pub fn get(&self, id: &str) -> Option<Node> {
        self.nodes.read().unwrap().get(id).cloned()
    }

    /// Return every node sorted by updated_at descending.
    pub fn all(&self) -> Vec<Node> {
        let guard = self.nodes.read().unwrap();
        let mut nodes: Vec<Node> = guard.values().cloned().collect();
        nodes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        nodes
    }

    /// Return the total number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 1-hop neighborhood (links + backlinks, deduplicated).
    pub fn neighbors(&self, id: &str) -> Vec<Node> {
        let guard = self.nodes.read().unwrap();
        let node = match guard.get(id) {
            Some(n) => n.clone(),
            None => return Vec::new(),
        };

        let mut seen = HashSet::new();
        seen.insert(id.to_string());

        let mut out = Vec::new();
        for lid in &node.links {
            if seen.insert(lid.clone()) {
                if let Some(n) = guard.get(lid) {
                    out.push(n.clone());
                }
            }
        }
        for bid in &node.backlinks {
            if seen.insert(bid.clone()) {
                if let Some(n) = guard.get(bid) {
                    out.push(n.clone());
                }
            }
        }
        out
    }

    /// 2-hop neighborhood.
    pub fn neighbors_2hop(&self, id: &str) -> Vec<Node> {
        let guard = self.nodes.read().unwrap();
        let node = match guard.get(id) {
            Some(n) => n.clone(),
            None => return Vec::new(),
        };

        let mut seen = HashSet::new();
        seen.insert(id.to_string());
        let mut out = Vec::new();
        let mut hop1 = Vec::new();

        for lid in &node.links {
            if seen.insert(lid.clone()) {
                if let Some(n) = guard.get(lid) {
                    out.push(n.clone());
                    hop1.push(lid.clone());
                }
            }
        }
        for bid in &node.backlinks {
            if seen.insert(bid.clone()) {
                if let Some(n) = guard.get(bid) {
                    out.push(n.clone());
                    hop1.push(bid.clone());
                }
            }
        }

        for h1_id in hop1 {
            if let Some(h1_node) = guard.get(&h1_id) {
                for lid in &h1_node.links {
                    if seen.insert(lid.clone()) {
                        if let Some(n) = guard.get(lid) {
                            out.push(n.clone());
                        }
                    }
                }
                for bid in &h1_node.backlinks {
                    if seen.insert(bid.clone()) {
                        if let Some(n) = guard.get(bid) {
                            out.push(n.clone());
                        }
                    }
                }
            }
        }

        out
    }

    /// 3-hop BFS neighborhood.
    pub fn neighbors_3hop(&self, id: &str) -> Vec<Node> {
        let guard = self.nodes.read().unwrap();
        if !guard.contains_key(id) {
            return Vec::new();
        }

        let mut seen = HashSet::new();
        seen.insert(id.to_string());
        let mut out = Vec::new();

        #[derive(Clone)]
        struct QueueItem {
            node_id: String,
            depth: usize,
        }

        let mut queue = vec![QueueItem {
            node_id: id.to_string(),
            depth: 0,
        }];

        while let Some(item) = queue.pop() {
            if item.depth >= 3 {
                continue;
            }

            let n = match guard.get(&item.node_id) {
                Some(n) => n,
                None => continue,
            };

            for lid in &n.links {
                if seen.insert(lid.clone()) {
                    if let Some(target) = guard.get(lid) {
                        out.push(target.clone());
                        queue.push(QueueItem {
                            node_id: lid.clone(),
                            depth: item.depth + 1,
                        });
                    }
                }
            }
            for bid in &n.backlinks {
                if seen.insert(bid.clone()) {
                    if let Some(target) = guard.get(bid) {
                        out.push(target.clone());
                        queue.push(QueueItem {
                            node_id: bid.clone(),
                            depth: item.depth + 1,
                        });
                    }
                }
            }
        }

        out
    }

    /// Substring search across title, content, and tags.
    pub fn search(&self, query: &str) -> Vec<Node> {
        let q = query.to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let guard = self.nodes.read().unwrap();
        guard
            .values()
            .filter(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.content.to_lowercase().contains(&q)
                    || n.tags.iter().any(|t| t.eq_ignore_ascii_case(&q))
            })
            .cloned()
            .collect()
    }

    /// Find nodes by tag.
    pub fn by_tag(&self, tag: &str) -> Vec<Node> {
        let guard = self.nodes.read().unwrap();
        guard
            .values()
            .filter(|n| n.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .cloned()
            .collect()
    }

    // ── Internal parsing ──────────────────────────────────────────────────

    fn parse_links(&self, content: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut links = Vec::new();
        for cap in self.link_re.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                let id = to_node_id(m.as_str());
                if seen.insert(id.clone()) {
                    links.push(id);
                }
            }
        }
        links
    }

    fn parse_tags(&self, content: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut tags = Vec::new();
        for cap in self.tag_re.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                let tag = m.as_str().to_string();
                if seen.insert(tag.clone()) {
                    tags.push(tag);
                }
            }
        }
        tags
    }
}

/// Normalise a wiki-link target into a stable lowercase_underscore ID.
fn to_node_id(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "_")
}

pub mod context;
pub mod store;

pub use context::DendriteContext;
pub use store::DendriteStore;
