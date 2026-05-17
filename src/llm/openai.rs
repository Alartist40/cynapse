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

// Streaming types
#[derive(Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
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
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await?;
        
        let byte_stream = response.bytes_stream();
        let text_stream = futures::stream::unfold(byte_stream, |mut stream| async move {
            let mut buffer = String::new();
            loop {
                match stream.next().await {
                    Some(Ok(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        // Process complete lines
                        if let Some(pos) = buffer.rfind('\n') {
                            let complete = buffer[..=pos].to_string();
                            buffer = buffer[pos + 1..].to_string();
                            return Some((complete, stream));
                        }
                    }
                    Some(Err(e)) => {
                        return Some((format!("ERROR: {e}"), stream));
                    }
                    None => {
                        if !buffer.is_empty() {
                            let remaining = buffer.clone();
                            buffer.clear();
                            return Some((remaining, stream));
                        }
                        return None;
                    }
                }
            }
        });

        let parsed = text_stream.flat_map(|text| {
            let chunks: Vec<Result<String>> = text
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with(':') {
                        return None;
                    }
                    if !line.starts_with("data: ") {
                        return None;
                    }
                    let data = &line[6..];
                    if data == "[DONE]" {
                        return None;
                    }
                    match serde_json::from_str::<OpenAIStreamChunk>(data) {
                        Ok(chunk) => {
                            let text = chunk.choices.first()
                                .map(|c| c.delta.content.clone())
                                .unwrap_or_default();
                            if text.is_empty() { None } else { Some(Ok(text)) }
                        }
                        Err(_) => None,
                    }
                })
                .collect();
            futures::stream::iter(chunks)
        });
        
        Ok(Box::new(Box::pin(parsed)))
    }
}
