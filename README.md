# 🧠 CYNAPSE — Pure Rust AI Agent System & Dendrite Graph Memory

**Cynapse** is a production-grade, local-first AI agent system built in **100% Pure Rust**. It features zero Python and zero Node.js dependencies, a **3-Tier Hardware & Engine Router**, **Paraclea-Style Dendrite 4-Tier Knowledge Graph Memory** (with SQLite FTS5 + BM25 search), and a **Colibri/Jcode-inspired visual Terminal UI (TUI)**.

---

## 🌟 Key Features

### 🚀 Zero External Runtime Dependencies
- Built completely in pure Rust (`Ratatui 0.29` + `Crossterm 0.28` + `Tokio` + `SQLite`).
- Ultra-lightweight memory footprint and instant startup times.

### ⌨️ Dynamic Input Auto-Layout & Word-by-Word Backspace (`Ctrl + Backspace`)
- **Downward Box Expansion**: The prompt input box dynamically expands downward (from 3 to 8 lines) as your prompt grows, keeping all input text visible while typing.
- **Line Word Wrapping**: Uses `.wrap(Wrap { trim: false })` so multi-line text wraps cleanly onto line 2, 3, 4, etc.
- **Multi-Line Cursor Tracking**: Multi-line aware terminal blinking cursor positioning (`f.set_cursor_position`) using exact line (`row_offset`) and column (`col_offset`) calculations.
- **Word Deletion (`Ctrl + Backspace` / `Ctrl + W`)**: `delete_word_backward` deletes words backward cleanly without inserting stray characters (`h`) or breaking prompt text.

### 🖼️ Complete 32-Line ASCII Banner Welcome Screen
- **Full 32-Line Brand Banner**: Renders the complete 32-line ASCII artwork logo (`ascii-art.txt`) centered in the Conversation Viewport upon launch or when clearing chat (`/clear`).
- **Automatic Chat Transition**: The ASCII banner cleanly recedes when you send your first prompt, focusing 100% on chat history, thinking blocks, and responses.

### 🗂️ Collapsible Reasoning Stream Cards (`/thinking` & `Ctrl + T`)
- **Expandable / Collapsible Thinking Blocks**: Model chain-of-thought (`[Thinking...]`) blocks can be collapsed into a compact single-line badge (`▶ [Thinking... (Collapsed)]`) or expanded (`▼ [Thinking...]`).
- **Toggle Shortcut**: Toggle visibility anytime via `Ctrl + T` shortcut or `/thinking` slash command.

### 📊 Live Tool Execution Progress Cards
- Real-time status cards in the Left Sidebar showing execution pipeline stages:
  - `FTS5 Index`: `✓ Active`
  - `Ranker`: `✓ BM25`
  - `RAG Budget`: `✓ 4k Budget`
  - `Engine`: `• Running...` / `✓ Idle`

### 🎨 In-Terminal Rich Markdown & Syntax-Highlighted Code Blocks
- Formats Markdown headers (`#`, `##`, `###`), bullet points (`•`), blockquotes (`│`), and fenced code blocks with language headers (`┌── [ RUST ] ───`, `└───`) and green syntax styling.

### 🖥️ Colibri-Style Left Sidebar & Hardware Telemetry
- **Left Sidebar Panel (26% width)**: Real-time telemetry status panel.
  - **SYSTEM HARDWARE**: CPU model & core count, RAM usage (`Used / Total GB`) with a visual progress bar `[████░░░░] 42%`, GPU & hardware acceleration status.
  - **MODEL DETAILS**: Active filename (`qwen2.5-0.5b-instruct-q4_k_m.gguf`), Quantization (`Q4_K_M`), Size (`398 MB`), Source (`Local File` / `Leafcutter`).
  - **ENGINE TIER & STATS**: Active tier, `tok/s` speed, latency.
  - **EXECUTION PIPELINE**: Live FTS5, BM25 ranker, RAG budget, and streaming status.
  - **VISUAL THEME**: Active color theme name (`Amber CRT`, etc.).
  - **DENDRITE MEMORY**: Live node & edge count, FTS5 + BM25 index status.

### 🌌 Multi-Galaxy 3D Dendrite Memory Visualizer (`/memory`)
- **Central Core Star (`✸`)**: Supermassive mass at origin `(0, 0, 0)` drawing category sub-galaxies into orbit.
- **Sub-Galaxy Clusters**: Categorized sub-galaxies (`Personal`, `Engineering`, `Preferences`, `Meta`, `Episodic`, `Transient Oort Cloud`) orbiting the central star.
- **Category Color-Coding**: Pink/Magenta (Personal), Cyan (Engineering), Yellow (Preferences), Green (Meta), White (Episodic), Dark Gray (Transient).
- **Specialization Node Sizing**: Scaled star symbols and brightness using normalized entropy specialization metrics ($\text{spec} > 0.75$ rendered as bright `★` / `✦`).
- **Interactive Controls**: `Left`/`Right`/`Up`/`Down` Arrow Keys rotate 3D Yaw & Pitch in real time, `Spacebar`/`s` toggles auto-spin.

### 📥 Atomic-Agent Inspired HuggingFace Model Downloader (`/pull` / `/download`)
- **Hardware Recommendation Engine**: Autodetects host RAM/VRAM and tags optimal curated models (`[★ Recommended]`).
- **Custom HuggingFace Write-In**: Input custom HuggingFace repo identifiers or URLs (e.g. `TheBloke/Llama-2-7B-GGUF` or `unsloth/gemma-4-12B-it-qat-GGUF`).
- **Quantization Selector**: Select target quantization level (`Q4_K_M`, `Q5_K_M`, `Q8_0`, `F16`).
- **Live Streaming Progress Bar**: Non-blocking background Tokio task with real-time speed (`14.2 MB/s`), percentage, and MB counter visualizer (`[██████░░░░] 60%`).
- **Auto-Registration**: Automatically saves to local `models` directory and registers the downloaded model as active.

### 🩺 Self-Healing System Doctor (`cynapse doctor` & `/doctor`)
- **Subsystem Diagnostic Engine**: Audits 9 subsystem areas (Host RAM Safety Headroom, AVX2 SIMD, GGUF magic header validation `0x46554747`, SQLite `PRAGMA quick_check;` & FTS5 health, GBNF tool call grammar, Atomic-Agent local tools, Tokio async task scheduler).
- **Auto-Healing Recovery (`--fix` / `r`/`F5`)**: Automatically repairs DB schemas, missing storage directories, stale scratch files, and broken model references.
- **TUI & CLI Entrypoints**: Run `cynapse doctor [--fix]` in CLI or type `/doctor` inside the TUI dashboard.

### 🗄️ Interactive Dendrite Memory Drawer (`Tab` Key / `/drawer`)
- **Instant Overlay (`Tab`)**: Press `Tab` anywhere in the TUI to open the interactive Memory Drawer inspector overlay.
- **Node Management**: Inspect title, category, specialization score, and hashtags. Delete obsolete nodes directly with `d` or `Delete`.

### 🛡️ Atomic-Agent Offline Capability Suite
- **GBNF Tool Call Grammar Validation**: Ensures zero-syntax-error tool execution offline.
- **Stable Prefix KV-Cache Preservation**: 100% KV-cache hit rate on local llama.cpp/GGUF engines.
- **Two-Tier Hybrid Memory Recall**: Combines BM25 term matches, specialization index boost, and exponential recency decay ($\gamma^{\Delta t}$).
- **Loop Guard Protection**: Circular buffer detecting repeating non-progressing tool call loops offline.

### ⚡ 3-Tier Hardware & Engine Router
- **Tier 1 (Fast Engine)**: HTTP streaming runner over local llama.cpp / Ollama endpoints (`reqwest` streaming) with automatic model tag resolution and 404 retries.
- **Tier 2 (Large GGUF)**: Leafcutter Pure Rust GGUF Layer Streaming Core.
- **Tier 3 (Large Safetensor)**: Leafcutter Pure Rust Safetensor Core.

### 🔍 Slash Command Dropdown Autocomplete (`/` Menu)
- Type `/` in the prompt input bar to open a floating autocomplete menu.
- Slash Commands:
  - `/help` — Display keyboard shortcuts & help menu
  - `/model` — Open interactive model selector
  - `/pull` — Open interactive HuggingFace model downloader (hardware curated)
  - `/doctor` — Run self-healing Cynapse Doctor system diagnostic & recovery
  - `/memory` — Open 3D Dendrite Memory Galaxy Visualizer
  - `/drawer` — Open interactive Dendrite Memory drawer inspector
  - `/thinking` — Toggle collapsible model thinking/reasoning blocks
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

### Keyboard Shortcuts
- `Up` / `Down` Arrow Keys: Scroll conversation viewport up/down line-by-line
- `Ctrl + Backspace` / `Ctrl + W`: Delete word backward in prompt bar
- `Ctrl + T`: Toggle collapsible model thinking/reasoning cards
- `Ctrl + A` / `Ctrl + E`: Jump cursor to start / end of prompt line
- `Ctrl + U`: Clear prompt line

---

## 📄 License
MIT License. Free for open-source and commercial use.
