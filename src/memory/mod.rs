use async_trait::async_trait;
use crate::error::Result;
use crate::llm::Message;

/// Memory trait - abstraction for conversation persistence
#[async_trait]
pub trait Memory: Send + Sync {
    /// Save a message to the session
    async fn save_message(&mut self, device_id: &str, message: Message) -> Result<()>;
    
    /// Get conversation history for a device
    async fn get_history(&self, device_id: &str, limit: usize) -> Result<Vec<Message>>;
    
    /// Clear conversation history for a device
    async fn clear_history(&mut self, device_id: &str) -> Result<()>;
}

pub mod sqlite;

use crate::config::MemoryConfig;

/// Create memory store from configuration
pub fn create_memory(config: &MemoryConfig) -> Result<Box<dyn Memory>> {
    Ok(Box::new(sqlite::SqliteMemory::new(config)?))
}
