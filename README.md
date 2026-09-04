# 🧠 CYNAPSE — Pure Rust AI Agent System & Dendrite Graph Memory

**Cynapse** is a production-grade, local-first AI agent system built in **100% Pure Rust**. It features zero Python and zero Node.js dependencies, a **3-Tier Hardware & Engine Router**, **Dendrite 4-Tier Knowledge Graph Memory** (with SQLite FTS5 + BM25 search), and a **Colibri/Jcode-inspired visual Terminal UI (TUI)**.

---

## 🌟 Key Features

### 🚀 Zero External Runtime Dependencies
- Built completely in pure Rust (`Ratatui 0.29` + `Crossterm 0.28` + `Tokio` + `SQLite`).
- Ultra-lightweight memory footprint and instant startup times.

### 🖥️ Colibri-Style Left Sidebar & Hardware Telemetry
- **Left Sidebar Panel (26% width)**: Real-time status panel.
  - **SYSTEM HARDWARE**: CPU model & core count, RAM usage (`Used / Total GB`) with a visual progress bar `[████░░░░] 42%`, GPU & hardware acceleration status.
  - **MODEL DETAILS**: Active filename (`qwen2.5-0.5b-instruct-q4_k_m.gguf`), Quantization (`Q4_K_M`), Size (`398 MB`), Source (`Local File` / `Leafcutter`).
  - **ENGINE TIER & STATS**: Active tier, `tok/s` speed, latency.
  - **VISUAL THEME**: Active color theme name (`Amber CRT`, etc.).
  - **DENDRITE MEMORY**: Live node & edge count, FTS5 + BM25 index status.

### 🎨 Jcode-Inspired Aesthetics & Smooth Rounded Borders
- **Background ASCII Art Banner**: Renders a centered ASCII art graphic (`ascii-art.txt`) in the Conversation Viewport when chat history is empty/welcome state.
- **Smooth Rounded Borders (`BorderType::Rounded`)**: All boxes, headers, sidebar, viewport, prompt box, slash command dropdown, and modals use smooth rounded corners (`╭ ╮ ╰ ╯`).
- **Clean Typography (Zero Emojis/Icons)**: Minimalist, professional typography for prompt lines (`・> `), roles (`User:`, `Cynapse:`), headers, and sidebar blocks.

### 🌌 3D Dendrite Memory Galaxy Visualizer (`/memory`)
- Projects Dendrite 4-tier graph memory nodes into a 3D spherical galaxy coordinate space projected onto the terminal grid.
- **Node Tier Cluster Legend**:
  - `*` **Tier 3 Summaries**: Core Galaxy Center (Magenta / Gold)
  - `+` **Tier 2 Procedures**: Inner Orbital Disk (Cyan)
  - `o` **Tier 1 Facts**: Outer Spiral Arms (Green)
  - `.` **Tier 0 Turn Logs**: Distant Stars (Yellow)
- **Interactive Controls**:
  - `Left` / `Right` / `Up` / `Down` Arrow Keys: Rotate 3D Yaw & Pitch in real time.
  - `Spacebar` / `s`: Toggle 3D Auto-Spin animation.
  - `Esc` / `q`: Exit Dendrite Galaxy.

### ⚡ 3-Tier Hardware & Engine Router
- **Tier 1 (Fast Engine)**: HTTP streaming runner over local llama.cpp / Ollama endpoints (`reqwest` streaming) with automatic model tag resolution and 404 retries.
- **Tier 2 (Large GGUF)**: Leafcutter Pure Rust GGUF Layer Streaming Core.
- **Tier 3 (Large Safetensor)**: Leafcutter Pure Rust Safetensor Core.

### 🧠 Dendrite 4-Tier Graph Memory
- SQLite FTS5 full-text index with BM25 relevance ranking.
- Categorized memory tiers: Summary (#summary), Procedure (#procedure), Fact (#fact), and TurnLog (#transcript).

### 🔍 Slash Command Dropdown Autocomplete (`/` Menu)
- Type `/` in the prompt input bar to open a floating autocomplete menu.
- Slash Commands:
  - `/help` — Display keyboard shortcuts & help menu
  - `/model` — Open interactive model selector
  - `/memory` — Open 3D Dendrite Memory Galaxy Visualizer
  - `/theme` — Cycle visual color theme (Dark Slate, Neon, Amber CRT, Emerald Matrix)
  - `/session` — Open saved sessions manager (resume past runs)
  - `/clear` — Clear conversation history
  - `/exit` — Quit Cynapse TUI
- Controls: `Up`/`Down` arrows navigate, `Tab`/`Right Arrow` auto-completes, `Enter` executes.

### 💾 Persistent Session Manager (`~/.cynapse/sessions/`)
- Automatic JSON conversation transcript persistence.
- Resume past sessions via `cynapse --resume <session_id>` or through the interactive `/session` modal.

---

## 🛠️ Installation & Building

### Prerequisites
- Linux / macOS / Windows
- Rust 1.75+ toolchain (`cargo`, `rustc`)

### Quick Install
```bash
git clone https://github.com/Alartist40/cynapse.git
cd cynapse
cargo build --release
./install.sh
```

Or build manually:
```bash
cargo build --release
cp target/release/cynapse ~/.local/bin/cynapse
```

---

## 🚀 Quick Usage

### Start TUI Mode (Default)
```bash
cynapse
```

### Start Line-by-Line CLI Mode
```bash
cynapse --cli
```

### Resume Past Session
```bash
cynapse --resume session_1725500000_1a2b
```

### CLI Subcommands
```bash
cynapse list              # List downloaded models
cynapse run <N>           # Run specific model by index or name
cynapse route             # Evaluate hardware RAM headroom & select optimal tier
cynapse pull <hf-repo>    # Streaming downloader for HuggingFace models
cynapse memory            # Render visual 4-tier Dendrite memory overview
```

---

## ⚙️ Configuration (`cynapse.toml`)

Create a `cynapse.toml` file in working directory or root to configure endpoints:

```toml
[engine]
tier1_endpoint = "http://127.0.0.1:11434"
default_model = "qwen2.5-0.5b-instruct-q4_k_m.gguf"

[memory]
db_path = "data/dendrite.db"
```

---

## 📄 License
MIT License. Free for open-source and commercial use.
