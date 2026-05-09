# README.md
# CYNAPSE Mini

🦀 **Lightweight Rust AI Agent for Embedded Systems**

CYNAPSE Mini is a minimal, fast, and resource-efficient AI agent optimized for Raspberry Pi and embedded systems.

## Features

- ✅ **Tiny Binary** - 4.3 MB (vs 100+ MB for full CYNAPSE)
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

### Usage

```bash
# Initialize configuration
./target/release/cynapse-mini init

# Start interactive chat
./target/release/cynapse-mini chat

# Single query
./target/release/cynapse-mini query "What is Rust?"

# Clear history
./target/release/cynapse-mini clear

# List available tools
./target/release/cynapse-mini tools
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

## License

MIT OR Apache-2.0
