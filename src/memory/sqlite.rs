use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::config::MemoryConfig;
use crate::error::{Result};
use crate::llm::{Message, Role};
use super::Memory;

pub struct SqliteMemory {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteMemory {
    pub fn new(config: &MemoryConfig) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = Path::new(&config.db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&config.db_path)?;
        
        // Create table if not exists
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
        
        // Create index for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_device_timestamp 
             ON messages(device_id, timestamp DESC)",
            [],
        )?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl Memory for SqliteMemory {
    async fn save_message(&mut self, device_id: &str, message: Message) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
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
        let mut stmt = conn.prepare(
            "SELECT role, content FROM messages 
             WHERE device_id = ?1 
             ORDER BY timestamp DESC 
             LIMIT ?2"
        )?;
        
        let rows = stmt.query_map(params![device_id, limit], |row| {
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
        for row in rows {
            messages.push(row?);
        }
        messages.reverse(); // Most recent last
        
        Ok(messages)
    }
    
    async fn clear_history(&mut self, device_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM messages WHERE device_id = ?1",
            params![device_id],
        )?;
        
        Ok(())
    }
}
