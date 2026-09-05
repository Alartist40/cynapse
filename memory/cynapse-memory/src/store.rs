//! DENDRITE SQLite persistence.
//!
//! Faithful port of Go `internal/memory/dendrite_store.go`. The schema
//! (table, index, FTS5 virtual table, triggers) and every query string are
//! kept identical so the Rust build reads and writes the same
//! `dendrite.db` as the Go original with zero migration.
//!
//! rusqlite's `Connection` is `!Sync`, so it is wrapped in a `Mutex`;
//! callers still share the store through `Arc` (single-writer semantics,
//! mirroring the Go `SetMaxOpenConns(1)`).

use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};

use crate::graph::{Dendrite, Node, NodeType};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS dendrite_nodes (
    id         TEXT PRIMARY KEY,
    title      TEXT NOT NULL,
    content    TEXT NOT NULL DEFAULT '',
    type       TEXT NOT NULL DEFAULT 'custom',
    tags       TEXT NOT NULL DEFAULT '[]',
    links      TEXT NOT NULL DEFAULT '[]',
    backlinks  TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dendrite_updated ON dendrite_nodes(updated_at DESC);
"#;

const FTS5_SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS dendrite_fts USING fts5(
    id         UNINDEXED,
    title,
    content,
    tags,
    tokenize = 'porter unicode61'
);
"#;

const FTS_FALLBACK_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS dendrite_fts_fallback (
    id    TEXT PRIMARY KEY,
    title TEXT,
    content TEXT,
    tags  TEXT
);
"#;

const TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS dendrite_nodes_ai
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
END;
"#;

/// Persists DENDRITE nodes to SQLite.
pub struct DendriteStore {
    conn: Mutex<Connection>,
    has_fts: bool,
}

fn lock_conn(conn: &Mutex<Connection>) -> MutexGuard<'_, Connection> {
    conn.lock().unwrap_or_else(|e| e.into_inner())
}

impl DendriteStore {
    pub fn open<P: AsRef<Path>>(db_path: P) -> Result<DendriteStore> {
        let conn = Connection::open_with_flags(
            db_path.as_ref(),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .context("open graph db")?;
        conn.busy_timeout(Duration::from_millis(5000))
            .context("set busy_timeout")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("enable WAL")?;

        let mut store = DendriteStore {
            conn: Mutex::new(conn),
            has_fts: true,
        };
        store.migrate().context("graph db migrate")?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let conn = lock_conn(&self.conn);
        conn.execute_batch(SCHEMA).context("create core table")?;

        // Try FTS5; if unavailable, gracefully fall back.
        match conn.execute_batch(FTS5_SCHEMA) {
            Ok(()) => {
                self.has_fts = true;
                conn.execute_batch(TRIGGERS).context("create fts triggers")?;
            }
            Err(_) => {
                self.has_fts = false;
                let _ = conn.execute_batch(FTS_FALLBACK_SCHEMA);
            }
        }
        Ok(())
    }

    /// Upsert a node into SQLite. created_at is preserved on conflict.
    pub fn save(&self, node: &Node) -> Result<()> {
        let tags = serde_json::to_string(&node.tags)?;
        let links = serde_json::to_string(&node.links)?;
        let backlinks = serde_json::to_string(&node.backlinks)?;

        let mut conn = lock_conn(&self.conn);
        let tx = conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO dendrite_nodes
                (id, title, content, type, tags, links, backlinks, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title      = excluded.title,
                content    = excluded.content,
                type       = excluded.type,
                tags       = excluded.tags,
                links      = excluded.links,
                backlinks  = excluded.backlinks,
                updated_at = excluded.updated_at
            "#,
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

        if !self.has_fts {
            tx.execute(
                r#"
                INSERT INTO dendrite_fts_fallback (id, title, content, tags)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    content = excluded.content,
                    tags = excluded.tags
                "#,
                params![node.id, node.title, node.content, tags],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let mut conn = lock_conn(&self.conn);
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM dendrite_nodes WHERE id = ?", [id])?;
        if !self.has_fts {
            tx.execute("DELETE FROM dendrite_fts_fallback WHERE id = ?", [id])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Hydrate all stored nodes directly into the graph's node map.
    pub fn load_all(&self, graph: &Dendrite) -> Result<()> {
        let conn = lock_conn(&self.conn);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, content, type, tags, links, backlinks, created_at, updated_at
            FROM dendrite_nodes
            ORDER BY updated_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let node_type: String = row.get(3)?;
            let tags: String = row.get(4)?;
            let links: String = row.get(5)?;
            let backlinks: String = row.get(6)?;
            Ok(Node {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                node_type: NodeType::from_str(&node_type),
                tags: serde_json::from_str(&tags).unwrap_or_default(),
                links: serde_json::from_str(&links).unwrap_or_default(),
                backlinks: serde_json::from_str(&backlinks).unwrap_or_default(),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        for node in rows {
            let node = node.context("reading node row")?;
            graph.insert_hydrated(node);
        }
        Ok(())
    }

    /// Full-text search, returning matching node IDs ordered by rank.
    pub fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let limit = if limit <= 0 { 10 } else { limit };
        let conn = lock_conn(&self.conn);

        if self.has_fts {
            let mut stmt = conn.prepare(
                r#"
                SELECT id FROM dendrite_fts
                WHERE dendrite_fts MATCH ?
                ORDER BY rank
                LIMIT ?
                "#,
            )?;
            let rows = stmt.query_map(params![query, limit as i64], |row| row.get(0))?;
            let mut ids = Vec::new();
            for row in rows {
                if let Ok(id) = row {
                    ids.push(id);
                }
            }
            return Ok(ids);
        }

        // Fallback: LIKE-based search across title/content/tags.
        // Escape SQL LIKE special characters so user input doesn't break
        // the pattern or allow injection.
        let pattern = format!(
            "%{}%",
            query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
        );
        let mut stmt = conn.prepare(
            r#"
            SELECT id FROM dendrite_fts_fallback
            WHERE title LIKE ? OR content LIKE ? OR tags LIKE ?
            LIMIT ?
            "#,
        )?;
        let rows =
            stmt.query_map(params![pattern, pattern, pattern, limit as i64], |row| {
                row.get(0)
            })?;
        let mut ids = Vec::new();
        for row in rows {
            if let Ok(id) = row {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    pub fn close(self) -> Result<()> {
        let conn = self.conn.into_inner().unwrap_or_else(|e| e.into_inner());
        conn.close().map_err(|(_, e)| e.into())
    }

    /// Number of rows in the core table.
    pub fn node_count(&self) -> Result<usize> {
        let conn = lock_conn(&self.conn);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM dendrite_nodes", [], |r| {
            r.get(0)
        })?;
        Ok(count as usize)
    }

    /// Perform SQLite database integrity check via PRAGMA quick_check.
    pub fn quick_check(&self) -> Result<String> {
        let conn = lock_conn(&self.conn);
        let status: String = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0))?;
        Ok(status)
    }
}
