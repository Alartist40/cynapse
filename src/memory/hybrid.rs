use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::config::MemoryConfig;
use crate::dendrite::{Dendrite, DendriteContext, DendriteStore, NodeType};
use crate::error::Result;
use crate::llm::{Message, Role};
use super::Memory;

/// Hybrid memory combines SQLite session history with DENDRITE graph memory.
/// - Session history: raw conversation messages (compacted when too long)
/// - DENDRITE: persistent knowledge graph with wiki-links and relevance scoring
pub struct HybridMemory {
    conn: Arc<Mutex<Connection>>,
    pub graph: Dendrite,
    store: DendriteStore,
    context: DendriteContext,
    max_history: usize,
    compaction_threshold: usize,
}

#[async_trait]
impl Memory for HybridMemory {
    async fn save_message(&mut self, device_id: &str, message: Message) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();
        let role_str = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (device_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![device_id, role_str, message.content, timestamp],
        )?;

        Ok(())
    }

    async fn get_history(&self, device_id: &str, limit: usize) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();

        // First get summaries
        let mut summaries = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT content FROM summaries 
                 WHERE device_id = ?1 
                 ORDER BY timestamp DESC 
                 LIMIT 3",
            )?;
            let rows = stmt.query_map(params![device_id], |row| {
                let content: String = row.get(0)?;
                Ok(content)
            })?;
            for row in rows {
                summaries.push(row?);
            }
        }

        // Then get recent messages
        let mut stmt = conn.prepare(
            "SELECT role, content FROM messages 
             WHERE device_id = ?1 
             ORDER BY timestamp DESC 
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![device_id, limit as i64], |row| {
            let role_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let role = match role_str.as_str() {
                "system" => Role::System,
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "tool" => Role::Tool,
                _ => Role::User,
            };
            Ok(Message { role, content })
        })?;

        let mut messages: Vec<Message> = Vec::new();

        // Add summaries as system context
        for summary in summaries.into_iter().rev() {
            messages.push(Message::system(format!("[Previous context summary] {}", summary)));
        }

        for row in rows {
            messages.push(row?);
        }
        messages.reverse();

        Ok(messages)
    }

    async fn clear_history(&mut self, device_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM messages WHERE device_id = ?1",
            params![device_id],
        )?;
        conn.execute(
            "DELETE FROM summaries WHERE device_id = ?1",
            params![device_id],
        )?;
        Ok(())
    }
}

impl HybridMemory {
    pub fn new(config: &MemoryConfig) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = Path::new(&config.db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&config.db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )?;

        // Session messages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_device_timestamp 
             ON messages(device_id, timestamp DESC)",
            [],
        )?;

        // Compaction summaries table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS summaries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_id TEXT NOT NULL,
                content TEXT NOT NULL,
                message_count INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        // DENDRITE graph + store
        let db_dir = Path::new(&config.db_path).parent().unwrap_or(Path::new("."));
        let dendrite_db = db_dir.join("dendrite.db");
        let mut store = DendriteStore::new(&dendrite_db)?;
        let graph = Dendrite::new();
        store.load_all(&graph).ok(); // Ignore errors on first run

        // Seed default persona nodes if graph is empty
        if graph.is_empty() {
            seed_default_persona(&graph, &mut store)?;
        }

        let context = DendriteContext::new(graph.clone(), None);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            graph,
            store,
            context,
            max_history: config.max_history,
            compaction_threshold: config.max_history + 20,
        })
    }

    /// Build a dynamic system prompt from DENDRITE graph, biased toward user query.
    pub fn build_system_prompt(&self, user_message: &str) -> String {
        let base = "You are Cynapse Mini, a lightweight AI agent with access to tools. \
                    You can use tools by outputting JSON: {\"tool\": \"name\", \"args\": \"...\"}\
                    After using a tool, wait for the result and then respond.";

        let context = self.context.build_prompt(user_message, 4000);
        if context.is_empty() {
            base.to_string()
        } else {
            format!("{}\n\n---\n\n{}", base, context)
        }
    }

    /// Save a fact to the DENDRITE graph (deduplicated).
    pub fn save_fact(&mut self, fact: &str, tags: &[String]) -> Result<()> {
        // Deduplication: check for identical existing memory
        let trimmed = fact.trim();
        for node in self.graph.all() {
            if node.node_type == NodeType::Memory && node.content.trim() == trimmed {
                // Merge tags
                let mut tag_set: std::collections::HashSet<String> =
                    node.tags.into_iter().collect();
                for t in tags {
                    tag_set.insert(t.clone());
                }
                let merged: Vec<String> = tag_set.into_iter().collect();
                let updated = self.graph.upsert(
                    node.id,
                    node.title,
                    node.content,
                    NodeType::Memory,
                    Some(merged),
                );
                self.store.save(&updated)?;
                return Ok(());
            }
        }

        let id = format!("fact_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let title = if fact.len() > 40 {
            format!("Fact: {}...", &fact[..40])
        } else {
            format!("Fact: {}", fact)
        };
        let node = self.graph.upsert(id, title, fact, NodeType::Memory, Some(tags.to_vec()));
        self.store.save(&node)?;
        Ok(())
    }

    /// Count messages for a device.
    fn message_count(&self, device_id: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE device_id = ?1",
            params![device_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Compact old messages: summarize oldest N into a summary message, extract facts.
    pub fn compact(&mut self, device_id: &str, llm_summary: Option<&str>) -> Result<()> {
        let count = self.message_count(device_id)?;
        if count < self.compaction_threshold {
            return Ok(());
        }

        let conn = self.conn.lock().unwrap();

        // Get oldest messages to compact
        let mut stmt = conn.prepare(
            "SELECT role, content FROM messages 
             WHERE device_id = ?1 
             ORDER BY timestamp ASC 
             LIMIT ?2",
        )?;
        let to_compact = self.compaction_threshold - self.max_history;
        let rows = stmt.query_map(params![device_id, to_compact as i64], |row| {
            let role: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok((role, content))
        })?;

        let mut conversation = String::new();
        let mut compacted_count = 0;
        for row in rows {
            let (role, content) = row?;
            conversation.push_str(&format!("{}: {}\n", role, content));
            compacted_count += 1;
        }
        drop(stmt);
        drop(conn);

        // Use provided summary or just note the compaction
        let summary_content = if let Some(summary) = llm_summary {
            summary.to_string()
        } else {
            format!("[{} older messages compacted]", compacted_count)
        };

        // Save summary
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO summaries (device_id, content, message_count, timestamp) 
             VALUES (?1, ?2, ?3, ?4)",
            params![device_id, summary_content, compacted_count, timestamp],
        )?;

        // Delete compacted messages
        conn.execute(
            "DELETE FROM messages WHERE device_id = ?1 AND id IN (
                SELECT id FROM messages 
                WHERE device_id = ?1 
                ORDER BY timestamp ASC 
                LIMIT ?2
            )",
            params![device_id, compacted_count as i64],
        )?;

        Ok(())
    }

    /// Get compaction summaries for context.
    pub fn get_summaries(&self, device_id: &str, limit: usize) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content FROM summaries 
             WHERE device_id = ?1 
             ORDER BY timestamp DESC 
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![device_id, limit as i64], |row| {
            let content: String = row.get(0)?;
            Ok(content)
        })?;

        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row?);
        }
        Ok(summaries)
    }
}

/// Seed default persona nodes into an empty graph.
fn seed_default_persona(graph: &Dendrite, store: &mut DendriteStore) -> Result<()> {
    let nodes = vec![
        ("identity", "Identity", "You are Cynapse Mini, a lightweight AI agent.", NodeType::Identity),
        ("user", "User Profile", "No profile information yet.", NodeType::Person),
        ("memory_notes", "Memory", "Long-term memory storage.", NodeType::Memory),
    ];

    for (id, title, content, node_type) in nodes {
        let node = graph.upsert(id, title, content, node_type, None);
        store.save(&node)?;
    }

    Ok(())
}
