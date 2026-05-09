use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{CynapseError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub llm: LLMConfig,
    pub memory: MemoryConfig,
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub device_id: String,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,  // "ollama", "anthropic", "openai"
    pub model: String,
    pub temperature: f64,
    pub max_tokens: usize,
    
    #[serde(default)]
    pub ollama: OllamaConfig,
    
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    
    #[serde(default)]
    pub openai: OpenAIConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub db_path: String,
    pub max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub enabled: Vec<String>,
    pub bash: BashToolConfig,
    pub file: FileToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashToolConfig {
    pub working_dir: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToolConfig {
    pub working_dir: String,
    pub max_file_size_mb: usize,
}

impl Config {
    /// Load configuration from YAML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CynapseError::ConfigError(format!("Failed to read config: {}", e)))?;
        
        // Expand environment variables
        let expanded = shellexpand::env(&content)
            .map_err(|e| CynapseError::ConfigError(format!("Failed to expand env vars: {}", e)))?;
        
        serde_yaml::from_str(&expanded)
            .map_err(|e| CynapseError::ConfigError(format!("Failed to parse YAML: {}", e)))
    }
    
    /// Create default configuration
    pub fn default() -> Self {
        Self {
            agent: AgentConfig {
                device_id: "cynapse_mini_01".to_string(),
                system_prompt: "You are CYNAPSE Mini, a helpful AI assistant.".to_string(),
            },
            llm: LLMConfig {
                provider: "ollama".to_string(),
                model: "qwen2:0.5b".to_string(),
                temperature: 0.7,
                max_tokens: 2048,
                ollama: OllamaConfig {
                    base_url: "http://localhost:11434".to_string(),
                },
                anthropic: AnthropicConfig::default(),
                openai: OpenAIConfig::default(),
            },
            memory: MemoryConfig {
                db_path: "data/sessions.db".to_string(),
                max_history: 20,
            },
            tools: ToolsConfig {
                enabled: vec!["bash".to_string(), "memory".to_string(), "file".to_string()],
                bash: BashToolConfig {
                    working_dir: ".".to_string(),
                    timeout_seconds: 30,
                },
                file: FileToolConfig {
                    working_dir: ".".to_string(),
                    max_file_size_mb: 10,
                },
            },
        }
    }
    
    /// Save configuration to YAML file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| CynapseError::ConfigError(format!("Failed to serialize: {}", e)))?;
        
        // Ensure directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(path, yaml)?;
        Ok(())
    }
}
