use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_stream::Stream;
use crate::config::LLMConfig;
use crate::error::{CynapseError, Result};
use super::{LLMProvider, Message, Role};

pub struct OpenAIProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    temperature: f64,
    max_tokens: usize,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
    max_tokens: usize,
    stream: bool,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

impl OpenAIProvider {
    pub fn new(config: &LLMConfig) -> Result<Self> {
        let api_key = config.openai.api_key.clone();
        if api_key.is_empty() {
            return Err(CynapseError::ConfigError(
                "OpenAI API key not configured".to_string()
            ));
        }
        
        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: config.openai.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
    }
    
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OpenAIMessage> {
        messages
            .into_iter()
            .map(|m| OpenAIMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant | Role::Tool => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            stream: false,
        };
        
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
        
        let openai_response: OpenAIResponse = response.json().await?;
        Ok(openai_response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default())
    }
    
    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            stream: true,
        };
        
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;
        
        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk.map_err(|e| CynapseError::HttpError(e))?;
            let text = String::from_utf8_lossy(&bytes);
            
            // Parse SSE format
            // Simplified - real implementation needs proper SSE parsing
            Ok(text.to_string())
        });
        
        Ok(Box::new(Box::pin(stream)))
    }
}
