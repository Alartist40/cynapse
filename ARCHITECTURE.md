# 🏗️ Cynapse Architecture & Developer Replication Blueprint

This document provides a comprehensive technical blueprint of the **Cynapse AI Agent System**. It details the zero-dependency pure Rust crate layout, the 3-tier hardware router, the Dendrite 4-tier memory graph engine, the Colibri/Jcode-inspired TUI, and step-by-step instructions for replicating or extending the architecture.

---

## 1. High-Level Workspace Architecture

Cynapse is structured as a modular **Cargo Workspace** consisting of five decoupled crates:

```
cynapse-mini/
├── Cargo.toml                  # Workspace manifest defining crate members
├── cynapse.toml                # Runtime configuration (endpoints, default model)
├── ascii-art.txt               # Synapse brand ASCII logo
├── harness/
│   ├── Cargo.toml (binary)     # Main CLI/TUI entrypoint binary (`cynapse`)
│   ├── src/main.rs             # Dynamic path resolution & Clap argument parser
│   ├── cynapse-core/           # Local tool registry & persistent session manager
│   └── cynapse-tui/            # Ratatui TUI event loop, theme engine, & 3D visualizer
├── engine/
│   ├── cynapse-engine/         # 3-tier hardware router & Tokio MPSC async stream runner
│   └── leafcutter_core/rust/   # Pure Rust GGUF & Safetensors layer-streaming engine
└── memory/
    └── cynapse-memory/         # Dendrite 4-tier knowledge graph (SQLite FTS5 + BM25)
```

---

## 2. Component Blueprint

### A. `harness/` (CLI & Terminal Interface)
- **`src/main.rs`**: Resolves local `./models` directories dynamically across the filesystem, parses command-line arguments using `clap` (`--cli`, `--tui`, `--resume`, `list`, `run`, `route`, `pull`, `memory`), and launches `TuiSession`.
- **`cynapse-core`**:
  - `session.rs`: Handles disk serialization (`SessionData`) to `~/.cynapse/sessions/<id>.json`. Provides atomic session listing, saving, loading, and transcript recovery.
  - `lib.rs`: Implements atomic agent tools (e.g. HuggingFace model stream downloader).
- **`cynapse-tui`**:
  - `terminal.rs`: Implements `TuiRuntimeGuard`, an RAII guard utilizing `std::panic::set_hook()` to ensure stdout/stderr raw terminal state is cleanly restored upon exits or panics.
  - `theme.rs`: Defines `AppTheme` palettes (`DarkSlate`, `Cyberpunk`, `AmberCRT`, `EmeraldMatrix`) with styling methods for headers, borders, prompts, role text, and highlights.
  - `app.rs`: Manages the non-blocking Tokio MPSC event loop, slash command autocomplete menu (`/`), Left Sidebar layout, background ASCII art rendering, and modal popups.

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

### C. `memory/` (Dendrite 4-Tier Knowledge Graph)

#### 1. Node Topology & Tiers (`cynapse-memory`)
Memory nodes are categorized into 4 hierarchy tiers:
- **Tier 3 (Consolidated Core)**: `#summary` / identity nodes (Magenta/Gold).
- **Tier 2 (Procedures)**: `#procedure` / how-to nodes (Cyan).
- **Tier 1 (Atomic Facts)**: `#fact` / user preference nodes (Green).
- **Tier 0 (Turn Logs)**: Ephemeral chat transcripts (Yellow).

#### 2. Search & Indexing Architecture
- **Full-Text Search**: SQLite FTS5 index for keyword lookups.
- **Relevance Ranking**: BM25 ranking algorithm scoring keyword match density and node recency.
- **Deterministic ID Generation**: Node IDs are generated deterministically based on title/tags to prevent duplicate graph fragmentation.

---

## 3. UI/UX Layout Architecture (Colibri & Jcode Inspired)

### A. Layout Grid
The TUI is split using Ratatui `Layout`:
```
+--------------------------------------------------------------------+
| Top Header Bar ("CYNAPSE TUI")                                     |
+------------------------------+-------------------------------------+
| LEFT SIDEBAR (26% width)     | MAIN CONVERSATION VIEWPORT (74%)    |
| - System Telemetry (RAM/CPU) | - Background ASCII Art Banner       |
| - Model Details & Quant      | - Multi-line Paragraph Text Wrap    |
| - Engine Tier & speed tok/s  | - Smooth PgUp/PgDn Scrolling        |
| - Visual Theme & Mem Stats   |                                     |
+------------------------------+-------------------------------------+
| Prompt Input Bar ('・> ' prefix with rounded border)               |
+--------------------------------------------------------------------+
```

### B. Rounded Border Styling (`BorderType::Rounded`)
All Ratatui `Block` widgets enforce smooth rounded corners (`╭ ╮ ╰ ╯`) via `.border_type(BorderType::Rounded)`.

### C. 3D Dendrite Memory Galaxy Visualizer
Projects Dendrite 3D spherical galaxy coordinates $(x,y,z)$ onto 2D terminal screen $(px, py)$ using rotational perspective matrices:
$$x' = x \cos \theta - z \sin \theta$$
$$z' = x \sin \theta + z \cos \theta$$
$$y' = y \cos \phi - z' \sin \phi$$
$$px = \text{center}_x + \lfloor x' \times \text{scale}_x \rfloor, \quad py = \text{center}_y + \lfloor y' \times \text{scale}_y \rfloor$$

---

## 4. Step-by-Step Blueprint for Replicating Cynapse

To replicate or recreate Cynapse in a new environment:

1. **Initialize Workspace**: Create `Cargo.toml` with workspace members `harness/cynapse-core`, `harness/cynapse-tui`, `engine/cynapse-engine`, `engine/leafcutter_core/rust`, and `memory/cynapse-memory`.
2. **Implement RAII Terminal Guard**: Wrap stdout with Crossterm `EnterAlternateScreen` and set a panic hook to restore standard terminal mode.
3. **Build Dendrite Memory Graph**: Implement SQLite FTS5 table schema with `title`, `content`, `node_type`, `tags`, and `updated_at`.
4. **Implement 3-Tier Router**: Add `/proc/meminfo` parser to measure available RAM.
5. **Implement Tokio MPSC Token Streaming**: Spawn an async Tokio task for LLM requests that emits stream events (`StreamEvent::Token`) to an unbounded MPSC channel polled by the main TUI loop.
6. **Construct Ratatui Layout**: Divide viewport into a 26/74 horizontal split, applying `BorderType::Rounded` to all blocks.
7. **Add Slash Command Popup & Modals**: Overlay popup boxes for `/model`, `/memory`, `/session`, `/theme`, and `/help`.
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
