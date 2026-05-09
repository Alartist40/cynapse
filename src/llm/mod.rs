use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio_stream::Stream;
use crate::error::Result;

/// Message role in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }
    
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
    
    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
        }
    }
}

/// LLM Provider trait - abstraction over different LLM APIs
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a completion request with message history
    async fn complete(&self, messages: Vec<Message>) -> Result<String>;
    
    /// Stream a completion response (for real-time output)
    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>>;
}

pub mod ollama;
pub mod anthropic;
pub mod openai;

use crate::config::LLMConfig;

/// Create LLM provider from configuration
pub fn create_provider(config: &LLMConfig) -> Result<Box<dyn LLMProvider>> {
    match config.provider.as_str() {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config)?)),
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
        "openai" => Ok(Box::new(openai::OpenAIProvider::new(config)?)),
        _ => Err(crate::error::CynapseError::ConfigError(
            format!("Unknown provider: {}", config.provider)
        )),
    }
}
