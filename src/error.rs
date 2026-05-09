use thiserror::Error;

#[derive(Error, Debug)]
pub enum CynapseError {
    #[error("LLM provider error: {0}")]
    LLMError(String),
    
    #[error("Tool execution error: {0}")]
    ToolError(String),
    
    #[error("Memory storage error: {0}")]
    MemoryError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, CynapseError>;
