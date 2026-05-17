use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{Dendrite, Node, NodeType};
use crate::error::Result;

/// Persists Dendrite nodes to SQLite.
pub struct DendriteStore {
    conn: Arc<Mutex<Connection>>,
    has_fts5: bool,
}

impl DendriteStore {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?;

        let mut store = Self {
            conn: Arc::new(Mutex::new(conn)),
            has_fts5: false,
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Core table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dendrite_nodes (
                id         TEXT PRIMARY KEY,
                title      TEXT NOT NULL,
                content    TEXT NOT NULL DEFAULT '',
                type       TEXT NOT NULL DEFAULT 'custom',
                tags       TEXT NOT NULL DEFAULT '[]',
                links      TEXT NOT NULL DEFAULT '[]',
                backlinks  TEXT NOT NULL DEFAULT '[]',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dendrite_updated ON dendrite_nodes(updated_at DESC)",
            [],
        )?;

        // Try FTS5; fall back gracefully
        if conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS dendrite_fts USING fts5(
                    id UNINDEXED,
                    title,
                    content,
                    tags,
                    tokenize = 'porter unicode61'
                )",
                [],
            )
            .is_ok()
        {
            self.has_fts5 = true;
            // Sync triggers
            conn.execute_batch(
                "CREATE TRIGGER IF NOT EXISTS dendrite_nodes_ai
                 AFTER INSERT ON dendrite_nodes BEGIN
                     INSERT INTO dendrite_fts(id, title, content, tags)
                     VALUES (new.id, new.title, new.content, new.tags);
                 END;

                 CREATE TRIGGER IF NOT EXISTS dendrite_nodes_au
                 AFTER UPDATE ON dendrite_nodes BEGIN
                     DELETE FROM dendrite_fts WHERE id = old.id;
                     INSERT INTO dendrite_fts(id, title, content, tags)
                     VALUES (new.id, new.title, new.content, new.tags);
                 END;

                 CREATE TRIGGER IF NOT EXISTS dendrite_nodes_ad
                 AFTER DELETE ON dendrite_nodes BEGIN
                     DELETE FROM dendrite_fts WHERE id = old.id;
                 END;",
            )?;
        } else {
            // Fallback table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS dendrite_fts_fallback (
                    id    TEXT PRIMARY KEY,
                    title TEXT,
                    content TEXT,
                    tags  TEXT
                )",
                [],
            )?;
        }

        Ok(())
    }

    /// Upsert a node into SQLite.
    pub fn save(&mut self, node: &Node) -> Result<()> {
        let tags = serde_json::to_string(&node.tags)?;
        let links = serde_json::to_string(&node.links)?;
        let backlinks = serde_json::to_string(&node.backlinks)?;

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO dendrite_nodes
                (id, title, content, type, tags, links, backlinks, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                type = excluded.type,
                tags = excluded.tags,
                links = excluded.links,
                backlinks = excluded.backlinks,
                updated_at = excluded.updated_at",
            params![
                node.id,
                node.title,
                node.content,
                node.node_type.as_str(),
                tags,
                links,
                backlinks,
                node.created_at,
                node.updated_at,
            ],
        )?;

        if !self.has_fts5 {
            tx.execute(
                "INSERT INTO dendrite_fts_fallback (id, title, content, tags)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    content = excluded.content,
                    tags = excluded.tags",
                params![node.id, node.title, node.content, tags],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Delete a node.
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM dendrite_nodes WHERE id = ?1", params![id])?;
        if !self.has_fts5 {
            tx.execute("DELETE FROM dendrite_fts_fallback WHERE id = ?1", params![id])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Load all nodes into a Dendrite graph.
    pub fn load_all(&self, graph: &Dendrite) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, title, content, type, tags, links, backlinks, created_at, updated_at
                      FROM dendrite_nodes ORDER BY updated_at DESC")?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let type_str: String = row.get(3)?;
            let tags_json: String = row.get(4)?;
            let links_json: String = row.get(5)?;
            let backlinks_json: String = row.get(6)?;
            let created_at: i64 = row.get(7)?;
            let updated_at: i64 = row.get(8)?;

            let node_type = type_str.parse().unwrap_or(NodeType::Custom);
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let links: Vec<String> = serde_json::from_str(&links_json).unwrap_or_default();
            let backlinks: Vec<String> = serde_json::from_str(&backlinks_json).unwrap_or_default();

            Ok(Node {
                id,
                title,
                content,
                node_type,
                tags,
                links,
                backlinks,
                created_at,
                updated_at,
            })
        })?;

        for row in rows {
            let node = row?;
            // Insert directly to preserve stored backlinks
            let mut guard = graph.nodes.write().unwrap();
            guard.insert(node.id.clone(), node);
        }

        Ok(())
    }

    /// Full-text search returning node IDs.
    pub fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let limit = if limit == 0 { 10 } else { limit };

        if self.has_fts5 {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id FROM dendrite_fts WHERE dendrite_fts MATCH ?1 ORDER BY rank LIMIT ?2")?;
            let rows = stmt.query_map(params![query, limit as i64], |row| row.get(0))?;
            let mut ids = Vec::new();
            for id in rows {
                ids.push(id?);
            }
            Ok(ids)
        } else {
            let pattern = format!("%{query}%");
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id FROM dendrite_fts_fallback
                          WHERE title LIKE ?1 OR content LIKE ?1 OR tags LIKE ?1
                          LIMIT ?2")?;
            let rows = stmt.query_map(params![&pattern, limit as i64], |row| row.get(0))?;
            let mut ids = Vec::new();
            for id in rows {
                ids.push(id?);
            }
            Ok(ids)
        }
    }
}
