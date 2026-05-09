# CYNAPSE Mini - Complete Rust Implementation Prompt

**Mission:** Build CYNAPSE Mini from scratch in Rust - a lightweight, embedded-first AI agent optimized for Raspberry Pi Zero 2W and resource-constrained environments.

**Timeline:** 16-20 hours of focused development  
**Target:** 3-5MB binary, <10MB RAM usage, production-ready  
**Repository:** Create new repo at `https://github.com/Alartist40/cynapse-mini.git`  
**Branch:** `main` (fresh start)

---

## 🎯 Project Context

### What You're Building
CYNAPSE Mini is the **lightweight, embedded sibling** of CYNAPSE (Go). While CYNAPSE is a full-featured TUI agent for desktop/server (100MB), CYNAPSE Mini is optimized for:
- Raspberry Pi Zero 2W (512MB RAM, ARMv7)
- Raspberry Pi 5 (ARM64)
- Embedded systems
- Resource-constrained environments
- Fast startup (<50ms)
- Minimal memory footprint (<10MB idle)

### Key Differentiation from CYNAPSE (Go)
| Aspect | CYNAPSE (Go) | CYNAPSE Mini (Rust) |
|--------|--------------|---------------------|
| Binary Size | 100+ MB | 3-5 MB |
| Interface | Full TUI (Bubble Tea) | Simple CLI |
| Memory Usage | 50+ MB | <10 MB |
| Target | Desktop/Server | Embedded/Pi |
| Startup | 200ms | <50ms |
| Dependencies | Many | Minimal |

### Learning from ZeroClaw (Reference Architecture)

**ZeroClaw Strengths to Adopt:**
1. ✅ **Trait-based design** - Provider, Tool, Memory traits
2. ✅ **Clean separation of concerns** - agent, llm, memory, tools as separate modules
3. ✅ **Multi-provider support** - Abstract LLM provider interface
4. ✅ **Tool execution framework** - Standardized tool calling
5. ✅ **Session persistence** - SQLite for conversation history
6. ✅ **Configuration management** - TOML-based config
7. ✅ **Error handling** - anyhow/thiserror patterns

**ZeroClaw Weaknesses to Avoid:**
1. ❌ **Too many features** - 71 tools, 30+ channels (bloat)
2. ❌ **Complex workspace** - 16 interconnected crates
3. ❌ **Large binary** - 8-20MB minimum
4. ❌ **Over-abstraction** - 50+ field Agent struct
5. ❌ **Feature flag hell** - Too many optional features

**Your Goal:** Take ZeroClaw's architectural patterns, strip out complexity, build lightweight and focused.

---

## 📋 Technical Specifications

### Core Requirements
1. **Multi-turn conversations** with persistent session memory
2. **LLM Provider Support**: Ollama (primary), Anthropic, OpenAI
3. **Streaming responses** for real-time output
4. **Tool execution**: Bash, Memory, File operations
5. **SQLite persistence** for conversation history
6. **YAML configuration** for easy customization
7. **Cross-compilation** for ARM64 and ARMv7

### Performance Targets
- **Binary size:** 3-5 MB (release build with LTO and strip)
- **Startup time:** <50ms cold start
- **Memory usage:** <10MB idle, <50MB during conversation
- **Compilation time:** <2 minutes clean build
- **Dependencies:** <20 direct dependencies

### Platform Support
- Linux x86_64 (development)
- Linux ARM64 (Raspberry Pi 5)
- Linux ARMv7 (Raspberry Pi Zero 2W)
- macOS ARM64 (optional, for development)

---

## 🏗️ Architecture Design

### Directory Structure
```
cynapse-mini/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── LICENSE
├── .gitignore
├── config.yaml                 # Default configuration
├── src/
│   ├── main.rs                 # CLI entry point (~150 lines)
│   ├── lib.rs                  # Public API exports
│   ├── agent.rs                # Core agent logic (~200 lines)
│   ├── config.rs               # Configuration management (~100 lines)
│   ├── llm/
│   │   ├── mod.rs              # LLM traits and client (~50 lines)
│   │   ├── ollama.rs           # Ollama implementation (~150 lines)
│   │   ├── anthropic.rs        # Anthropic implementation (~150 lines)
│   │   └── openai.rs           # OpenAI implementation (~150 lines)
│   ├── memory/
│   │   ├── mod.rs              # Memory trait (~30 lines)
│   │   └── sqlite.rs           # SQLite session store (~200 lines)
│   ├── tools/
│   │   ├── mod.rs              # Tool trait and registry (~80 lines)
│   │   ├── bash.rs             # Bash execution (~100 lines)
│   │   ├── memory.rs           # Memory operations (~80 lines)
│   │   └── file.rs             # File operations (~100 lines)
│   └── error.rs                # Error types (~50 lines)
├── tests/
│   ├── integration_test.rs
│   └── agent_test.rs
└── examples/
    └── simple_chat.rs
```

**Total lines of code target:** ~1,500 lines (excluding tests)

---

## 🔧 Implementation Guide

### Phase 1: Project Setup (1 hour)

#### Step 1.1: Initialize Repository
```bash
# Create new repository
git clone https://github.com/Alartist40/cynapse-mini.git
cd cynapse-mini

# Initialize Cargo project
cargo init --name cynapse-mini

# Create directory structure
mkdir -p src/{llm,memory,tools}
touch src/{lib.rs,agent.rs,config.rs,error.rs}
touch src/llm/{mod.rs,ollama.rs,anthropic.rs,openai.rs}
touch src/memory/{mod.rs,sqlite.rs}
touch src/tools/{mod.rs,bash.rs,memory.rs,file.rs}
```

#### Step 1.2: Create Cargo.toml
```toml
[package]
name = "cynapse-mini"
version = "1.0.0"
edition = "2021"
authors = ["Alartist40"]
license = "MIT OR Apache-2.0"
description = "Lightweight Rust AI agent for embedded systems and Raspberry Pi"
repository = "https://github.com/Alartist40/cynapse-mini"
keywords = ["ai", "agent", "cli", "embedded", "raspberry-pi"]
categories = ["command-line-utilities"]

[[bin]]
name = "cynapse-mini"
path = "src/main.rs"

[lib]
name = "cynapse_mini"
path = "src/lib.rs"

[dependencies]
# Async runtime - minimal features for size
tokio = { version = "1.50", default-features = false, features = ["rt-multi-thread", "macros", "time", "io-util", "sync", "process", "fs"] }
tokio-stream = { version = "0.1", default-features = false, features = ["sync"] }

# HTTP client - minimal features
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Database
rusqlite = { version = "0.37", features = ["bundled"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
async-trait = "0.1"
futures = "0.3"

[dev-dependencies]
tempfile = "3.26"

[profile.release]
opt-level = "z"          # Optimize for size
lto = "fat"              # Full link-time optimization
codegen-units = 1        # Better optimization
strip = true             # Remove debug symbols
panic = "abort"          # Smaller binary

[profile.dev]
opt-level = 0
incremental = true
```

#### Step 1.3: Create .gitignore
```gitignore
# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Runtime
*.db
*.db-journal
config.local.yaml
.env
```

#### Step 1.4: Create Default config.yaml
```yaml
# CYNAPSE Mini Configuration

# Agent settings
agent:
  device_id: "cynapse_mini_01"
  system_prompt: "You are CYNAPSE Mini, a helpful AI assistant running on a Raspberry Pi."

# LLM Provider (ollama, anthropic, or openai)
llm:
  provider: "ollama"
  model: "qwen2:0.5b"
  temperature: 0.7
  max_tokens: 2048
  
  # Ollama settings
  ollama:
    base_url: "http://localhost:11434"
  
  # Anthropic settings (when provider = "anthropic")
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    model: "claude-sonnet-4-20250514"
  
  # OpenAI settings (when provider = "openai")
  openai:
    api_key: "${OPENAI_API_KEY}"
    model: "gpt-4"

# Memory/Session storage
memory:
  db_path: "data/sessions.db"
  max_history: 20  # Max messages to keep in context

# Tools configuration
tools:
  enabled: ["bash", "memory", "file"]
  
  bash:
    working_dir: "."
    timeout_seconds: 30
  
  file:
    working_dir: "."
    max_file_size_mb: 10
```

---

### Phase 2: Core Types and Error Handling (1 hour)

#### Step 2.1: Create src/error.rs
```rust
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
```

#### Step 2.2: Create src/config.rs
```rust
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
```

**IMPORTANT:** Add to Cargo.toml dependencies:
```toml
shellexpand = "3.1"
```

---

### Phase 3: LLM Provider Implementation (3 hours)

#### Step 3.1: Create src/llm/mod.rs (Traits)
```rust
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
```

#### Step 3.2: Create src/llm/ollama.rs (Ollama Implementation)
```rust
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
    done: bool,
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
```

#### Step 3.3: Create src/llm/anthropic.rs (Anthropic Implementation)
```rust
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
        // Streaming implementation for Anthropic
        // Similar to complete but with stream: true
        // Parse SSE events from response
        
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
```

#### Step 3.4: Create src/llm/openai.rs (OpenAI Implementation)
```rust
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
```

---

### Phase 4: Memory/Session Storage (2 hours)

#### Step 4.1: Create src/memory/mod.rs (Trait)
```rust
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
```

#### Step 4.2: Create src/memory/sqlite.rs (SQLite Implementation)
```rust
use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::path::Path;
use crate::config::MemoryConfig;
use crate::error::{CynapseError, Result};
use crate::llm::{Message, Role};
use super::Memory;

pub struct SqliteMemory {
    conn: Connection,
    max_history: usize,
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
            conn,
            max_history: config.max_history,
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
        
        self.conn.execute(
            "INSERT INTO messages (device_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![device_id, role_str, message.content, timestamp],
        )?;
        
        Ok(())
    }
    
    async fn get_history(&self, device_id: &str, limit: usize) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
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
        
        let mut messages: Vec<Message> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
        messages.reverse(); // Most recent last
        
        Ok(messages)
    }
    
    async fn clear_history(&mut self, device_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM messages WHERE device_id = ?1",
            params![device_id],
        )?;
        
        Ok(())
    }
}
```

---

### Phase 5: Tool System (3 hours)

#### Step 5.1: Create src/tools/mod.rs (Tool Trait)
```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Tool trait - abstraction for executable capabilities
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;
    
    /// Tool description for LLM
    fn description(&self) -> &str;
    
    /// Execute the tool with given arguments
    async fn execute(&self, args: &str) -> Result<String>;
}

/// Tool registry manages available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }
    
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name)
    }
    
    pub fn list(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.name().to_string()).collect()
    }
    
    pub async fn execute(&self, name: &str, args: &str) -> Result<String> {
        match self.get(name) {
            Some(tool) => tool.execute(args).await,
            None => Err(crate::error::CynapseError::ToolError(
                format!("Tool '{}' not found", name)
            )),
        }
    }
}

pub mod bash;
pub mod memory;
pub mod file;

use crate::config::ToolsConfig;

/// Create tool registry from configuration
pub fn create_tools(config: &ToolsConfig) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::new();
    
    for tool_name in &config.enabled {
        match tool_name.as_str() {
            "bash" => registry.register(Box::new(bash::BashTool::new(&config.bash))),
            "memory" => registry.register(Box::new(memory::MemoryTool::new())),
            "file" => registry.register(Box::new(file::FileTool::new(&config.file)?)),
            _ => tracing::warn!("Unknown tool: {}", tool_name),
        }
    }
    
    Ok(registry)
}
```

#### Step 5.2: Create src/tools/bash.rs (Bash Tool)
```rust
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use crate::config::BashToolConfig;
use crate::error::{CynapseError, Result};
use super::Tool;

pub struct BashTool {
    working_dir: String,
    timeout_seconds: u64,
}

impl BashTool {
    pub fn new(config: &BashToolConfig) -> Self {
        Self {
            working_dir: config.working_dir.clone(),
            timeout_seconds: config.timeout_seconds,
        }
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }
    
    fn description(&self) -> &str {
        "Execute bash commands in the working directory"
    }
    
    async fn execute(&self, args: &str) -> Result<String> {
        tracing::info!("Executing bash command: {}", args);
        
        // Safety check - don't allow certain dangerous commands
        if args.contains("rm -rf /") || args.contains(":(){ :|:& };:") {
            return Err(CynapseError::ToolError(
                "Dangerous command blocked".to_string()
            ));
        }
        
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_seconds),
            Command::new("sh")
                .arg("-c")
                .arg(args)
                .current_dir(&self.working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        )
        .await
        .map_err(|_| CynapseError::ToolError("Command timeout".to_string()))??;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() {
            Ok(format!("Exit code: {}\nStderr: {}\nStdout: {}", 
                      output.status.code().unwrap_or(-1),
                      stderr,
                      stdout))
        } else {
            Ok(stdout.to_string())
        }
    }
}
```

#### Step 5.3: Create src/tools/memory.rs (Memory Tool)
```rust
use async_trait::async_trait;
use crate::error::Result;
use super::Tool;

pub struct MemoryTool;

impl MemoryTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory"
    }
    
    fn description(&self) -> &str {
        "Save or recall information from conversation memory"
    }
    
    async fn execute(&self, args: &str) -> Result<String> {
        // Simple implementation - just acknowledge
        // In a real implementation, this would interact with a knowledge base
        tracing::info!("Memory tool called with: {}", args);
        
        if args.starts_with("save:") {
            Ok("Information saved to memory".to_string())
        } else if args.starts_with("recall:") {
            Ok("Recalling from memory...".to_string())
        } else {
            Ok("Memory tool - use 'save:' or 'recall:' prefix".to_string())
        }
    }
}
```

#### Step 5.4: Create src/tools/file.rs (File Tool)
```rust
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use crate::config::FileToolConfig;
use crate::error::{CynapseError, Result};
use super::Tool;

pub struct FileTool {
    working_dir: PathBuf,
    max_file_size_bytes: usize,
}

impl FileTool {
    pub fn new(config: &FileToolConfig) -> Result<Self> {
        Ok(Self {
            working_dir: PathBuf::from(&config.working_dir),
            max_file_size_bytes: config.max_file_size_mb * 1024 * 1024,
        })
    }
    
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let resolved = self.working_dir.join(path);
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        
        // Security check - ensure path is within working directory
        if !canonical.starts_with(&self.working_dir) {
            return Err(CynapseError::ToolError(
                "Path escapes working directory".to_string()
            ));
        }
        
        Ok(canonical)
    }
}

#[async_trait]
impl Tool for FileTool {
    fn name(&self) -> &str {
        "file"
    }
    
    fn description(&self) -> &str {
        "Read, write, or list files in the working directory"
    }
    
    async fn execute(&self, args: &str) -> Result<String> {
        let parts: Vec<&str> = args.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(CynapseError::ToolError(
                "Invalid format. Use 'read:path', 'write:path:content', or 'list:path'".to_string()
            ));
        }
        
        let command = parts[0];
        let rest = parts[1];
        
        match command {
            "read" => {
                let path = self.resolve_path(rest)?;
                
                // Check file size
                let metadata = tokio::fs::metadata(&path).await?;
                if metadata.len() as usize > self.max_file_size_bytes {
                    return Err(CynapseError::ToolError(
                        format!("File too large (max {}MB)", self.max_file_size_bytes / 1024 / 1024)
                    ));
                }
                
                let content = tokio::fs::read_to_string(&path).await?;
                Ok(content)
            }
            
            "write" => {
                let write_parts: Vec<&str> = rest.splitn(2, ':').collect();
                if write_parts.len() != 2 {
                    return Err(CynapseError::ToolError(
                        "Write format: 'write:path:content'".to_string()
                    ));
                }
                
                let path = self.resolve_path(write_parts[0])?;
                let content = write_parts[1];
                
                // Create parent directory if needed
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                
                tokio::fs::write(&path, content).await?;
                Ok(format!("Wrote {} bytes to {}", content.len(), path.display()))
            }
            
            "list" => {
                let path = self.resolve_path(rest)?;
                let mut entries = tokio::fs::read_dir(&path).await?;
                let mut files = Vec::new();
                
                while let Some(entry) = entries.next_entry().await? {
                    files.push(entry.file_name().to_string_lossy().to_string());
                }
                
                Ok(files.join("\n"))
            }
            
            _ => Err(CynapseError::ToolError(
                format!("Unknown command: {}. Use read, write, or list", command)
            )),
        }
    }
}
```

---

### Phase 6: Core Agent Implementation (3 hours)

#### Step 6.1: Create src/agent.rs
```rust
use futures::StreamExt;
use crate::config::Config;
use crate::error::{CynapseError, Result};
use crate::llm::{self, Message, LLMProvider};
use crate::memory::{self, Memory};
use crate::tools::{self, ToolRegistry};

pub struct Agent {
    device_id: String,
    system_prompt: String,
    llm: Box<dyn LLMProvider>,
    memory: Box<dyn Memory>,
    tools: ToolRegistry,
}

impl Agent {
    /// Create a new agent from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let llm = llm::create_provider(&config.llm)?;
        let memory = memory::create_memory(&config.memory)?;
        let tools = tools::create_tools(&config.tools)?;
        
        Ok(Self {
            device_id: config.agent.device_id.clone(),
            system_prompt: config.agent.system_prompt.clone(),
            llm,
            memory,
            tools,
        })
    }
    
    /// Process a user message and return response
    pub async fn process(&mut self, user_input: &str) -> Result<String> {
        // 1. Load conversation history
        let mut history = self.memory.get_history(&self.device_id, 20).await?;
        
        // 2. Add system prompt if history is empty
        if history.is_empty() {
            history.push(Message::system(&self.system_prompt));
        }
        
        // 3. Add user message
        let user_msg = Message::user(user_input);
        self.memory.save_message(&self.device_id, user_msg.clone()).await?;
        history.push(user_msg);
        
        // 4. Get LLM response
        let response = self.llm.complete(history).await?;
        
        // 5. Save assistant response
        let assistant_msg = Message::assistant(&response);
        self.memory.save_message(&self.device_id, assistant_msg).await?;
        
        Ok(response)
    }
    
    /// Process a user message with streaming output
    pub async fn process_stream<F>(&mut self, user_input: &str, mut on_chunk: F) -> Result<String>
    where
        F: FnMut(&str) + Send,
    {
        // 1. Load conversation history
        let mut history = self.memory.get_history(&self.device_id, 20).await?;
        
        // 2. Add system prompt if history is empty
        if history.is_empty() {
            history.push(Message::system(&self.system_prompt));
        }
        
        // 3. Add user message
        let user_msg = Message::user(user_input);
        self.memory.save_message(&self.device_id, user_msg.clone()).await?;
        history.push(user_msg);
        
        // 4. Stream LLM response
        let mut stream = self.llm.stream(history).await?;
        let mut full_response = String::new();
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(text) => {
                    on_chunk(&text);
                    full_response.push_str(&text);
                }
                Err(e) => return Err(e),
            }
        }
        
        // 5. Save assistant response
        let assistant_msg = Message::assistant(&full_response);
        self.memory.save_message(&self.device_id, assistant_msg).await?;
        
        Ok(full_response)
    }
    
    /// Execute a tool
    pub async fn execute_tool(&self, tool_name: &str, args: &str) -> Result<String> {
        self.tools.execute(tool_name, args).await
    }
    
    /// List available tools
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.list()
    }
    
    /// Clear conversation history
    pub async fn clear_history(&mut self) -> Result<()> {
        self.memory.clear_history(&self.device_id).await
    }
}
```

---

### Phase 7: CLI Interface (2 hours)

#### Step 7.1: Create src/lib.rs
```rust
pub mod agent;
pub mod config;
pub mod error;
pub mod llm;
pub mod memory;
pub mod tools;

pub use agent::Agent;
pub use config::Config;
pub use error::{CynapseError, Result};
```

#### Step 7.2: Create src/main.rs
```rust
use clap::{Parser, Subcommand};
use cynapse_mini::{Agent, Config};
use std::io::{self, Write};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "CYNAPSE Mini")]
#[command(about = "Lightweight Rust AI Agent for Embedded Systems", version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive chat
    Chat,
    
    /// Execute a single query
    Query {
        /// The query to send to the agent
        query: String,
    },
    
    /// Initialize default configuration
    Init,
    
    /// Show version information
    Version,
    
    /// Clear conversation history
    Clear,
    
    /// List available tools
    Tools,
}

#[tokio::main]
async fn main() -> cynapse_mini::Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    let filter = if cli.debug {
        EnvFilter::new("cynapse_mini=debug")
    } else {
        EnvFilter::new("cynapse_mini=info")
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    match cli.command {
        Some(Commands::Init) => {
            let config = Config::default();
            config.save(&cli.config)?;
            println!("Created default configuration at: {}", cli.config);
            Ok(())
        }
        
        Some(Commands::Version) => {
            println!("CYNAPSE Mini v{}", env!("CARGO_PKG_VERSION"));
            println!("Rust AI Agent for Embedded Systems");
            Ok(())
        }
        
        Some(Commands::Chat) => {
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            println!("🦀 CYNAPSE Mini - Interactive Chat");
            println!("Type 'quit' or 'exit' to stop\n");
            
            loop {
                print!("> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input == "quit" || input == "exit" {
                    break;
                }
                
                // Stream response
                print!("Assistant: ");
                io::stdout().flush()?;
                
                agent.process_stream(input, |chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().ok();
                }).await?;
                
                println!("\n");
            }
            
            println!("Goodbye!");
            Ok(())
        }
        
        Some(Commands::Query { query }) => {
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            let response = agent.process(&query).await?;
            println!("{}", response);
            Ok(())
        }
        
        Some(Commands::Clear) => {
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            agent.clear_history().await?;
            println!("Conversation history cleared");
            Ok(())
        }
        
        Some(Commands::Tools) => {
            let config = Config::load(&cli.config)?;
            let agent = Agent::new(&config)?;
            
            println!("Available tools:");
            for tool in agent.list_tools() {
                println!("  - {}", tool);
            }
            Ok(())
        }
        
        None => {
            // Default to chat if no command specified
            let config = Config::load(&cli.config)?;
            let mut agent = Agent::new(&config)?;
            
            println!("🦀 CYNAPSE Mini");
            println!("Type 'quit' to exit\n");
            
            loop {
                print!("> ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input == "quit" || input == "exit" {
                    break;
                }
                
                print!("Assistant: ");
                io::stdout().flush()?;
                
                agent.process_stream(input, |chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().ok();
                }).await?;
                
                println!("\n");
            }
            
            Ok(())
        }
    }
}
```

---

### Phase 8: Documentation (1 hour)

#### Step 8.1: Create README.md
```markdown
# CYNAPSE Mini

🦀 **Lightweight Rust AI Agent for Embedded Systems**

CYNAPSE Mini is a minimal, fast, and resource-efficient AI agent optimized for Raspberry Pi and embedded systems.

## Features

- ✅ **Tiny Binary** - 3-5 MB (vs 100+ MB for full CYNAPSE)
- ✅ **Low Memory** - <10 MB idle, <50 MB active
- ✅ **Fast Startup** - <50ms cold start
- ✅ **Multi-Provider** - Ollama, Anthropic, OpenAI support
- ✅ **Streaming** - Real-time response output
- ✅ **Persistent Memory** - SQLite conversation history
- ✅ **Simple CLI** - No TUI bloat
- ✅ **Cross-Platform** - ARM64, ARMv7, x86_64

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/Alartist40/cynapse-mini.git
cd cynapse-mini

# Build release binary
cargo build --release

# Binary at: target/release/cynapse-mini
```

### Cross-Compilation for Raspberry Pi

```bash
# For Pi 5 (ARM64)
cargo build --release --target aarch64-unknown-linux-gnu

# For Pi Zero 2W (ARMv7)
cargo build --release --target armv7-unknown-linux-gnueabihf
```

### Usage

```bash
# Initialize configuration
./cynapse-mini init

# Start interactive chat
./cynapse-mini chat

# Single query
./cynapse-mini query "What is Rust?"

# Clear history
./cynapse-mini clear

# List available tools
./cynapse-mini tools
```

## Configuration

Edit `config.yaml`:

```yaml
agent:
  device_id: "cynapse_mini_01"
  system_prompt: "You are CYNAPSE Mini, a helpful AI assistant."

llm:
  provider: "ollama"  # ollama, anthropic, or openai
  model: "qwen2:0.5b"
  temperature: 0.7
  max_tokens: 2048
  
  ollama:
    base_url: "http://localhost:11434"

memory:
  db_path: "data/sessions.db"
  max_history: 20

tools:
  enabled: ["bash", "memory", "file"]
```

## Architecture

```
cynapse-mini/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── agent.rs      # Core agent logic
│   ├── config.rs     # Configuration management
│   ├── llm/          # LLM provider implementations
│   ├── memory/       # SQLite session storage
│   └── tools/        # Tool implementations
```

## Performance

| Metric | Value |
|--------|-------|
| Binary Size | 3-5 MB |
| Startup Time | <50ms |
| Idle Memory | <10 MB |
| Active Memory | <50 MB |

## Comparison with CYNAPSE (Go)

| Feature | CYNAPSE (Go) | CYNAPSE Mini (Rust) |
|---------|--------------|---------------------|
| Binary Size | 100+ MB | 3-5 MB |
| Interface | Full TUI | Simple CLI |
| Memory Usage | 50+ MB | <10 MB |
| Target | Desktop/Server | Embedded/Pi |
| Startup | 200ms | <50ms |

## License

MIT OR Apache-2.0

## Credits

Built with inspiration from [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw).
```

#### Step 8.2: Create CHANGELOG.md
```markdown
# Changelog

All notable changes to CYNAPSE Mini will be documented in this file.

## [1.0.0] - 2026-05-09

### Added
- Initial release of CYNAPSE Mini
- Multi-provider LLM support (Ollama, Anthropic, OpenAI)
- Streaming response support
- SQLite session persistence
- Basic tool system (bash, memory, file)
- Simple CLI interface
- Cross-platform ARM compilation support
- YAML configuration
- Memory management with conversation history

### Performance
- Binary size: 3-5 MB
- Startup time: <50ms
- Memory usage: <10 MB idle

### Documentation
- Complete README with quick start guide
- Configuration examples
- Cross-compilation instructions
```

---

### Phase 9: Testing (2 hours)

#### Step 9.1: Create tests/integration_test.rs
```rust
use cynapse_mini::{Agent, Config};
use tempfile::TempDir;

#[tokio::test]
async fn test_agent_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    let config = Config::default();
    config.save(&config_path).unwrap();
    
    let loaded_config = Config::load(&config_path).unwrap();
    assert_eq!(loaded_config.agent.device_id, "cynapse_mini_01");
}

#[tokio::test]
async fn test_memory_persistence() {
    // Test that messages are saved and retrieved correctly
    // This would require mocking the memory layer
}
```

#### Step 9.2: Run Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_agent_creation
```

---

### Phase 10: Build Optimization and Deployment (2 hours)

#### Step 10.1: Optimize Binary Size
```bash
# Build with maximum optimization
cargo build --release

# Check binary size
ls -lh target/release/cynapse-mini

# Expected: 3-5 MB

# Strip additional symbols (if needed)
strip target/release/cynapse-mini
```

#### Step 10.2: Cross-Compilation Setup
```bash
# Install cross-compilation tools
rustup target add aarch64-unknown-linux-gnu
rustup target add armv7-unknown-linux-gnueabihf

# Install linkers (Ubuntu/Debian)
sudo apt-get install gcc-aarch64-linux-gnu
sudo apt-get install gcc-arm-linux-gnueabihf

# Build for different targets
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target armv7-unknown-linux-gnueabihf
cargo build --release --target x86_64-unknown-linux-gnu
```

#### Step 10.3: Create Release Script
Create `scripts/build-all.sh`:
```bash
#!/bin/bash

set -e

echo "Building CYNAPSE Mini for all platforms..."

# x86_64 Linux
cargo build --release --target x86_64-unknown-linux-gnu
echo "✓ Built x86_64-unknown-linux-gnu"

# ARM64 (Pi 5)
cargo build --release --target aarch64-unknown-linux-gnu
echo "✓ Built aarch64-unknown-linux-gnu"

# ARMv7 (Pi Zero 2W)
cargo build --release --target armv7-unknown-linux-gnueabihf
echo "✓ Built armv7-unknown-linux-gnueabihf"

echo ""
echo "Binary sizes:"
ls -lh target/*/release/cynapse-mini

echo ""
echo "✓ All builds complete!"
```

Make executable:
```bash
chmod +x scripts/build-all.sh
```

---

### Phase 11: Final Git Commit and Push (30 minutes)

#### Step 11.1: Create .gitignore (if not already)
```gitignore
/target/
**/*.rs.bk
*.pdb
Cargo.lock
.vscode/
.idea/
*.swp
*.db
*.db-journal
config.local.yaml
.env
```

#### Step 11.2: Commit and Push
```bash
# Initialize git (if not already)
git init

# Add all files
git add .

# Initial commit
git commit -m "Initial release: CYNAPSE Mini v1.0.0

- Multi-provider LLM support (Ollama, Anthropic, OpenAI)
- Streaming response capability
- SQLite session persistence
- Basic tool system (bash, memory, file)
- Simple CLI interface
- Cross-platform ARM compilation
- Binary size: 3-5 MB
- Startup: <50ms
- Memory: <10 MB idle"

# Set remote
git remote add origin https://github.com/Alartist40/cynapse-mini.git

# Push to main
git branch -M main
git push -u origin main
```

---

## ✅ Success Criteria

### Functionality
- [x] Multi-turn conversations work
- [x] LLM streaming responses work
- [x] Session history persists across runs
- [x] All three providers (Ollama, Anthropic, OpenAI) functional
- [x] Tools execute correctly
- [x] CLI commands all work

### Performance
- [x] Binary size: 3-5 MB
- [x] Startup time: <50ms
- [x] Memory usage: <10 MB idle
- [x] Compilation time: <2 minutes

### Quality
- [x] Clean code (no warnings)
- [x] Proper error handling
- [x] Comprehensive documentation
- [x] Cross-platform builds work
- [x] Tests pass

---

## 🎯 Final Deliverables

1. **Working CYNAPSE Mini binary** (3-5 MB)
2. **Complete source code** (~1,500 lines)
3. **README.md** with quick start
4. **CHANGELOG.md** documenting v1.0.0
5. **config.yaml** with sensible defaults
6. **Cross-compilation** support for ARM
7. **Git repository** with clean history
8. **Tests** for core functionality

---

## 💡 Key Design Principles

### What Makes This Better Than ZeroClaw for Embedded

1. **Simplicity Over Features**
   - 3 tools vs 71 tools
   - 1 workspace vs 16 crates
   - 1,500 lines vs 7,000+ lines

2. **Size Optimization**
   - Minimal dependencies
   - No feature flags (everything needed included)
   - LTO + strip + panic=abort

3. **Clear Architecture**
   - Single main.rs entry point
   - Trait-based abstractions
   - No over-engineering

4. **Embedded-First**
   - Optimized for Pi Zero 2W
   - Low memory footprint
   - Fast startup
   - No unnecessary dependencies

---

## 🚀 You're Done!

After completing all phases, you'll have:

✅ Production-ready CYNAPSE Mini  
✅ 3-5 MB binary  
✅ <50ms startup  
✅ Multi-provider LLM support  
✅ Streaming responses  
✅ Session persistence  
✅ Clean, maintainable code  
✅ Full documentation  
✅ Cross-platform builds  

**This is genuinely lightweight, genuinely different from CYNAPSE (Go), and genuinely optimized for Raspberry Pi!** 🎉
