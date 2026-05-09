use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_stream::Stream;
use crate::config::LLMConfig;
use crate::error::{CynapseError, Result};
use super::{LLMProvider, Message, Role};

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    temperature: f64,
    max_tokens: usize,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: usize,
    temperature: f64,
    stream: bool,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

impl AnthropicProvider {
    pub fn new(config: &LLMConfig) -> Result<Self> {
        let api_key = config.anthropic.api_key.clone();
        if api_key.is_empty() {
            return Err(CynapseError::ConfigError(
                "Anthropic API key not configured".to_string()
            ));
        }
        
        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: config.anthropic.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
    }
    
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<AnthropicMessage> {
        messages
            .into_iter()
            .filter(|m| m.role != Role::System) // System prompt handled separately
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant | Role::Tool => "assistant".to_string(),
                    Role::System => "user".to_string(), // Shouldn't reach here
                },
                content: m.content,
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: false,
        };
        
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;
        
        let anthropic_response: AnthropicResponse = response.json().await?;
        Ok(anthropic_response.content.first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }
    
    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: true,
        };
        
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;
        
        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk.map_err(|e| CynapseError::HttpError(e))?;
            let text = String::from_utf8_lossy(&bytes);
            
            // Parse SSE format
            // This is simplified - actual implementation needs proper SSE parsing
            Ok(text.to_string())
        });
        
        Ok(Box::new(Box::pin(stream)))
    }
}
