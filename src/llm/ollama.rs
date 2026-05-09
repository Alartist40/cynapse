use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_stream::Stream;
use crate::config::LLMConfig;
use crate::error::{CynapseError, Result};
use super::{LLMProvider, Message, Role};

pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
    temperature: f64,
    max_tokens: usize,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f64,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
    // done: bool,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

impl OllamaProvider {
    pub fn new(config: &LLMConfig) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            base_url: config.ollama.base_url.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
    }
    
    fn convert_messages(&self, messages: Vec<Message>) -> Vec<OllamaMessage> {
        messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "assistant".to_string(), // Ollama doesn't have tool role
                },
                content: m.content,
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        let request = OllamaRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            stream: false,
            options: OllamaOptions {
                temperature: self.temperature,
                num_predict: self.max_tokens as i32,
            },
        };
        
        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.message.content)
    }
    
    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        let request = OllamaRequest {
            model: self.model.clone(),
            messages: self.convert_messages(messages),
            stream: true,
            options: OllamaOptions {
                temperature: self.temperature,
                num_predict: self.max_tokens as i32,
            },
        };
        
        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        let stream = response.bytes_stream().map(|chunk| {
            let bytes = chunk.map_err(|e| CynapseError::HttpError(e))?;
            let text = String::from_utf8_lossy(&bytes);
            
            // Parse each line as JSON
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                
                if let Ok(response) = serde_json::from_str::<OllamaResponse>(line) {
                    return Ok(response.message.content);
                }
            }
            
            Ok(String::new())
        });
        
        Ok(Box::new(Box::pin(stream)))
    }
}
