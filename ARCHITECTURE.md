# 🏗️ Cynapse Architecture & Developer Replication Blueprint

This document provides a comprehensive technical blueprint of the **Cynapse AI Agent System**. It details the zero-dependency pure Rust crate layout, the 3-tier hardware router, the Paraclea-style Dendrite 4-tier memory graph engine, the Colibri/Jcode-inspired TUI with dynamic multi-line input layout, and step-by-step instructions for replicating or extending the architecture.

---

## 1. High-Level Workspace Architecture

Cynapse is structured as a modular **Cargo Workspace** consisting of five decoupled crates:

```
cynapse-mini/
├── Cargo.toml                  # Workspace manifest defining crate members
├── cynapse.toml                # Runtime configuration (endpoints, default model)
├── ascii-art.txt               # Synapse brand ASCII logo (32 lines)
├── harness/
│   ├── Cargo.toml (binary)     # Main CLI/TUI entrypoint binary (`cynapse`)
│   ├── src/main.rs             # Dynamic path resolution & Clap argument parser
│   ├── cynapse-core/           # Local tool registry, offline GBNF validator, loop guard, & session manager
│   └── cynapse-tui/            # Ratatui TUI event loop, theme engine, Memory Drawer, & 3D Multi-Galaxy visualizer
├── engine/
│   ├── cynapse-engine/         # 3-tier hardware router & Tokio MPSC async stream runner
│   └── leafcutter_core/rust/   # Pure Rust GGUF & Safetensors layer-streaming engine
└── memory/
    └── cynapse-memory/         # Dendrite 4-tier knowledge graph (SQLite FTS5 + BM25 + Two-Tier Hybrid Recall)
```

---

## 2. Component Blueprint

### A. `harness/` (CLI, Terminal Interface, & Offline Agent Tools)
- **`src/main.rs`**: Resolves local `./models` directories dynamically across the filesystem, parses command-line arguments using `clap` (`--cli`, `--tui`, `--resume`, `list`, `run`, `route`, `pull`, `memory`), and launches `TuiSession`.
- **`cynapse-core`**:
  - `session.rs`: Handles disk serialization (`SessionData`) to `~/.cynapse/sessions/<id>.json`. Provides atomic session listing, saving, loading, and transcript recovery.
  - `offline_agent.rs`: Offline agent utilities:
    - `validate_gbnf_tool_calls()`: Strict GBNF JSON tool call syntax validator supporting single objects and batch arrays (`[{"tool": "...", "args": {...}}]`).
    - `format_stable_kv_prompt()`: Invariant system prompt prefix formatter ensuring 100% KV-cache hit rate.
    - `LoopGuard`: Circular buffer tracking tool call hashes and triggering automated intervention upon 3+ identical consecutive invocations.
  - `persona.rs`: Manages markdown system persona files (`IDENTITY.md`, `SOUL.md`, `USER.md`, `SYSTEM.md`) in `~/.cynapse/persona/`, constructing high-character, non-generic system prompts.
  - `doctor.rs`: Cynapse Self-Healing System Doctor auditing 9 critical subsystem areas (RAM safety headroom, AVX2 SIMD, GGUF magic headers, SQLite/FTS5 integrity, GBNF schema parser, atomic tools, Tokio channels, Tier-1 LLM endpoint, and Markdown Persona System) performing automatic self-healing repairs via CLI `cynapse doctor [--fix]` or TUI `/doctor`.
  - `downloader.rs`: Model downloader & recommendation engine:
    - `recommend_model_for_hardware()`: Selects optimal GGUF model based on host RAM.
    - `resolve_hf_download_url_async()`: Uses HuggingFace API repository tree discovery (`/api/models/{repo}/tree/main`) to locate exact GGUF filenames for custom model repos, handling case/format differences automatically.
    - `stream_download_hf_model()`: Async stream downloader forwarding progress callbacks (`speed_mbps`, `downloaded_bytes`, `pct`).
  - `lib.rs`: Implements atomic agent tool execution (`read_file`, `write_file`, `grep`, `execute_command`) and HuggingFace streaming downloader.
- **`cynapse-tui`**:
  - `terminal.rs`: Implements `TuiRuntimeGuard`, an RAII guard utilizing `std::panic::set_hook()` to ensure terminal raw mode is safely restored on exit or panic.
  - `theme.rs`: Defines `AppTheme` palettes (`DarkSlate`, `Cyberpunk`, `AmberCRT`, `EmeraldMatrix`) with styling methods for headers, borders, prompts, role text, and highlights.
  - `app.rs`: Manages the non-blocking Tokio MPSC event loop, slash command menu (`/`), Left Sidebar layout, dynamic prompt input auto-layout, collapsible thinking cards (`Ctrl + T`), rich Markdown parser, viewport scroll badge (`[▲ Scroll XX% ▼]`), Memory Drawer (`Tab`), and 3D Multi-Galaxy Atlas.

---

### B. `engine/` (3-Tier Hardware Router & Inference Engine)

#### 1. Hardware Headroom Calculator (`cynapse-engine`)
Before loading models, the router evaluates host RAM and VRAM availability using `/proc/meminfo`:
$$\text{Memory Needed} = \text{Model Disk Size} + 1.5 \text{ GiB (KV Cache Headroom)}$$

#### 2. Tier Selection Routing Logic
- **Tier 1 (Fast Engine)**: Selected when model fits within host RAM or GPU offload is preferred. Uses Tokio async stream runner (`query_tier1_stream`) sending HTTP POST queries to local LLM server (`/api/generate`). Includes model tag matching (`/api/tags`) to resolve `.gguf` file names to registered tags with 404 retries.
- **Tier 2 (Large GGUF Core)**: Selected when GGUF model size exceeds host RAM. Invokes Leafcutter pure Rust layer-streaming engine to stream weights from disk on demand.
- **Tier 3 (Large Safetensor Core)**: Selected when model uses Safetensors format. Invokes Leafcutter Safetensors engine.

#### 3. System Hardware Telemetry (`probe_hardware_info()`)
Parses `/proc/cpuinfo` and `/proc/meminfo` to return `SystemHardwareInfo`:
- CPU brand model & online core count.
- Total RAM, Available RAM, Used RAM, and RAM usage percentage bar `[████░░░░] 42%`.
- GPU device & VRAM status.

---

### C. `memory/` (Paraclea-Style Dendrite 4-Tier Knowledge Graph)

#### 1. Node Topology & Sub-Galaxy Clusters (`cynapse-memory`)
Memory nodes are classified into 4 hierarchy tiers and grouped into 6 spinning sub-galaxy category clusters:
- **Tier 3 (Consolidated Core)**: `#summary` / identity nodes (`★` Magenta/Gold).
- **Tier 2 (Procedures)**: `#procedure` / how-to nodes (`✪` Cyan).
- **Tier 1 (Atomic Facts)**: `#fact` / user preference nodes (`●` Green).
- **Tier 0 (Turn Logs)**: Ephemeral chat transcripts (`.` Yellow).
- **Sub-Galaxy Clusters (`NodeCategory`)**: `Personal` (Pink), `Engineering` (Cyan), `Preferences` (Amber), `Meta & Identity` (Green), `Episodic` (White), and `Transient Oort Cloud` (Dark Gray).
- **Specialization Metric ($\text{spec}(e)$)**: Normalized entropy metric ranking nodes from generalist hubs ($\text{spec} \le 0.5$) to domain specialists ($\text{spec} > 0.75$).

#### 2. Search & Two-Tier Hybrid Recall Architecture
- **Full-Text Search**: SQLite FTS5 index for keyword lookups.
- **Two-Tier Hybrid Scoring**: Combines lexical BM25 term frequency, specialization boost $\text{spec}(e)$, and exponential recency decay ($\gamma^{\Delta t}$):
  $$\text{Score} = (\text{BM25} \cdot 0.95^{\Delta t}) + (4.0 \cdot \text{spec}(e)) + 0.3 \cdot (\text{Links} + \text{Backlinks})$$
- **Content Sanitization**: `clean_node_content` strips internal wiki-links (`Target: [[...]]`, `Linked: [[...]]`) before prompt injection.
- **Turn Log Exclusion**: Ephemeral `TurnLog` nodes are stored in SQLite and visual topology, but excluded from RAG system prompt context insertion to prevent prompt bloat and model hallucinations.

---

## 3. UI/UX Layout Architecture (Colibri & Jcode Inspired)

### A. Layout Grid & Dynamic Auto-Layout
The TUI is split using Ratatui `Layout`:
```
+--------------------------------------------------------------------+
| Top Header Bar ("CYNAPSE TUI")                                     |
+------------------------------+-------------------------------------+
| LEFT SIDEBAR (26% width)     | MAIN CONVERSATION VIEWPORT (74%)    |
| - System Telemetry (RAM/CPU) | - 32-Line ASCII Art Welcome Banner  |
| - Model Details & Quant      | - Multi-line Paragraph Text Wrap    |
| - Engine Tier & speed tok/s  | - Smooth Line-by-Line Scroll (Up/Down)|
| - Live Execution Pipeline    | - Collapsible Thinking Cards        |
| - Visual Theme & Mem Stats   | - Rich Markdown Syntax Highlighting |
+------------------------------+-------------------------------------+
| Prompt Input Bar ('・> ' prefix, dynamic 3..8 lines height)        |
+--------------------------------------------------------------------+
```

### B. Dynamic Input Box Height Calculation
The bottom input box dynamically measures required visual lines when wrapped at inner width:
```rust
let input_total_cols = 4 + self.input.chars().count();
let wrapped_input_lines = (input_total_cols + input_inner_width - 1) / input_inner_width;
let input_height = (wrapped_input_lines as u16 + 2).clamp(3, 8);
```

### C. Rounded Border Styling (`BorderType::Rounded`)
All Ratatui `Block` widgets enforce smooth rounded corners (`╭ ╮ ╰ ╯`) via `.border_type(BorderType::Rounded)`.

---

## 4. Step-by-Step Blueprint for Replicating Cynapse

To replicate or recreate Cynapse in a new environment:

1. **Initialize Workspace**: Create `Cargo.toml` with workspace members `harness/cynapse-core`, `harness/cynapse-tui`, `engine/cynapse-engine`, `engine/leafcutter_core/rust`, and `memory/cynapse-memory`.
2. **Implement RAII Terminal Guard**: Wrap stdout with Crossterm `EnterAlternateScreen` and set a panic hook to restore standard terminal mode.
3. **Build Dendrite Memory Graph**: Implement SQLite FTS5 table schema with `title`, `content`, `node_type`, `tags`, and `updated_at`.
4. **Implement 3-Tier Router**: Add `/proc/meminfo` parser to measure available RAM.
5. **Implement Tokio MPSC Token Streaming**: Spawn an async Tokio task for LLM requests that emits stream events (`StreamEvent::Token`) to an unbounded MPSC channel polled by the main TUI loop.
6. **Construct Ratatui Layout**: Divide viewport into a 26/74 horizontal split, applying `BorderType::Rounded` to all blocks.
7. **Add Slash Command Popup & Modals**: Overlay popup boxes for `/model`, `/memory`, `/thinking`, `/session`, `/theme`, and `/help`.
8. **Verify Unit Tests**: Enforce unit tests across all crates ensuring 100% build stability (`cargo test --workspace`).

---

## 5. Build & Deployment Verification

```bash
# Check compilation across all crates
cargo check --workspace

# Run all unit tests
cargo test --workspace

# Build release binary
cargo build --release

# Install binary to local user path
cp target/release/cynapse ~/.local/bin/cynapse
```
