# 🧠 CYNAPSE — Offline-First Agent System & Synaptic Knowledge Memory

**Cynapse** is a high-performance, offline-first AI agent platform engineered entirely in 100% pure Rust. Designed for total privacy, zero external runtime dependencies, and instant local execution, it bridges host hardware telemetry with an intelligent multi-engine router and a dynamic, synaptic knowledge graph.

At the heart of Cynapse lies a unified architecture where three powerful engines work in concert: **Dendrite**, a 4-tier graph memory system powered by SQLite FTS5 full-text indexing and BM25 relevance ranking that organically grows and adapts like biological synapses; **Leafcutter**, a custom pure-Rust GGUF and Safetensors execution core providing local tensor streaming; and **llama.cpp / Ollama integration**, acting as a high-speed Tier-1 inference runner. Together, these subsystems provide a complete local agent harness capable of autonomous tool execution, strict GBNF grammar constrained outputs, real-time memory synthesis, 3D orbital galaxy memory visualization, and self-healing diagnostics—all running locally on your hardware without a single byte leaving your machine.

---

## 📥 Installation & Setup

### Single-Line Automated Install (Linux & macOS)
```bash
curl -fsSL https://raw.githubusercontent.com/Alartist40/cynapse/main/install.sh | bash
```

### Build from Source
```bash
git clone https://github.com/Alartist40/cynapse.git
cd cynapse
cargo build --release
cp target/release/cynapse ~/.local/bin/cynapse
```

### System Health Verification
After installation, run the self-healing diagnostic to verify hardware, SIMD acceleration, storage, and database integrity:
```bash
cynapse doctor --fix
```

---

## 🕹️ Menu & Usage Guide

### Launch Modes
- **Visual TUI Dashboard (Default)**: Launch the full visual Ratatui interactive interface:
  ```bash
  cynapse
  ```
- **CLI REPL Mode**: Run in line-by-line terminal mode:
  ```bash
  cynapse --cli
  ```
- **Resume Past Session**: Reopen any saved session transcript by ID:
  ```bash
  cynapse --resume <session_id>
  ```
- **Run Model Directly**:
  ```bash
  cynapse run <model_name_or_number>
  ```

### Interactive Slash Commands (`/`)
Type `/` in the prompt bar to trigger the floating command menu:
- `/help` — Display interactive keyboard shortcuts & help menu
- `/model` — Open interactive model selector & scanner
- `/pull` — Download GGUF models from HuggingFace (Curated catalog & custom URLs)
- `/persona` — Manage agent personality markdown files (`IDENTITY`, `SOUL`, `USER`, custom `.md`)
- `/doctor` — Launch Cynapse Doctor self-healing diagnostic dashboard
- `/memory` — Launch 3D Orbital Galaxy Memory Atlas visualizer
- `/drawer` — Open interactive Dendrite Memory drawer inspector
- `/thinking` — Toggle collapsible model reasoning/thinking stream blocks
- `/theme` — Cycle color themes (Dark Slate, Neon Cyber, Amber CRT, Emerald Matrix)
- `/session` — Manage and resume saved conversation sessions
- `/clear` — Reset conversation view and restore brand banner
- `/exit` — Quit Cynapse TUI

### Terminal Shortcuts
- `Tab`: Open Dendrite Memory Drawer inspector from anywhere
- `Up` / `Down`: Scroll conversation viewport line-by-line
- `Ctrl + Backspace` / `Ctrl + W`: Delete word backward in prompt input
- `Ctrl + T`: Toggle model thinking/reasoning blocks
- `Ctrl + A` / `Ctrl + E`: Move input cursor to start / end of line
- `Ctrl + U`: Clear input line
- `Spacebar` / `s` (in `/memory`): Toggle 3D Galaxy auto-rotation
- `r` / `F5` (in `/doctor`): Re-run self-healing diagnostics with auto-repair

---

## 🏗️ Architecture Structure

Cynapse is structured as a decoupled multi-crate Rust workspace:

```
cynapse-mini/
├── harness/
│   ├── src/main.rs             # CLI & TUI entrypoint dispatcher
│   ├── cynapse-tui/            # Ratatui visual interface, modals, 3D visualizer
│   └── cynapse-core/           # Tool execution, GBNF parser, session manager, doctor
├── memory/
│   └── cynapse-memory/         # Dendrite graph engine, SQLite FTS5 store, BM25 ranker
├── engine/
│   ├── cynapse-engine/         # Semantic hardware router & Tier-1 LLM client
│   └── leafcutter_core/        # Pure Rust GGUF & Safetensors tensor kernels
└── install.sh                  # Dual remote/local automated installer & setup script
```

### Data & Memory Flow
```
[ User Input ]
      │
      ▼
[ Dendrite Memory Core ] ──( SQLite FTS5 + BM25 )──► [ Relevant Context Facts ]
      │                                                        │
      ▼                                                        ▼
[ Hardware Tier Router ] ◄───────────────────────── [ Injected System Prompt ]
      │
      ├─────► Tier 1 (Fast): llama.cpp / Ollama HTTP Stream
      ├─────► Tier 2 (GGUF): Leafcutter Rust Tensor Core
      └─────► Tier 3 (Safetensor): Leafcutter Rust Core
      │
      ▼
[ GBNF Tool Grammar Parser ] ──( Tool Executed )──► [ Tool Output Result ]
      │                                                     │
      └─────────────────────◄ ( Reprompt Loop ) ────────────┘
      │
      ▼
[ Turn Log & Fact Extraction ] ──► [ Dendrite Knowledge Graph ]
```

---

## 🌟 Capabilities, Specs & Features

### 🧠 Synaptic Memory & 3D Galaxy Memory Atlas
Memory in Cynapse is modeled after biological neural networks. Ideas, facts, and conversation turn logs form nodes connected by weighted synaptic links that strengthen with use and decay with disuse over time.
- **3D Orbital Galaxy Visualizer (`/memory`)**: Visualizes memory nodes as stars organized into galactic clusters revolving around a supermassive central core of knowledge. Categories (Personal, Engineering, Preferences, Meta, Episodic) orbit in colorful stellar belts. Node size dynamically scales with entropy-based specialization metrics.
- **Two-Tier Hybrid Memory Recall**: Merges SQLite FTS5 keyword matching with BM25 scoring, specialization index weighting, and exponential temporal decay ($\gamma^{\Delta t}$).
- **Interactive Memory Drawer (`Tab`)**: Inspect, search, and purge individual memory nodes directly from a pop-up overlay.

### 📥 Offline Model Downloader & Model Management (`/pull`)
- **Curated Recommendations**: Probes system RAM and GPU VRAM to tag optimal model sizes (`[★ Recommended]`).
- **Custom HuggingFace Downloads**: Download any model from HuggingFace by entering a repository ID (e.g., `Qwen/Qwen2.5-7B-Instruct-GGUF` or `TheBloke/Llama-2-7B-GGUF`) or pasting a direct GGUF file URL. Select quantization levels ranging from `Q2_K` to `F16`.
- **Background Async Progress**: Non-blocking Tokio downloader streams models directly to storage with real-time speed (`MB/s`), progress bars, and automatic activation upon completion.

### 🛡️ Offline Agent Capabilities & GBNF Grammar Engine
- **GBNF Grammar Constraints**: Enforces valid JSON tool-call schema syntax, preventing output formatting panics offline.
- **KV-Cache Slot Preservation**: Reuses prompt prefix KV-caches across conversation turns for near-zero prefix evaluation latency.
- **Loop Guard Protection**: Active circular buffer detects and halts non-progressing repeated tool execution loops.
- **Max Step Safeguards**: Configurable step limits

### 🎭 System Persona Manager (`/persona`)
- **Interactive TUI Modal**: Inspect available `.md` persona files in `~/.cynapse/persona/`, preview file contents and system prompt outputs in real-time, switch active personas (`Enter`), or reset to default identity (`r`).
- **Dynamic Prompt Compiler**: Injects direct, high-character system instructions into every model prompt without generic LLM headers or preambles.

### 🩺 Cynapse Doctor Self-Healing Engine (`/doctor`)
- **9-Subsystem Auditing**: Audits host RAM headroom, SIMD instruction availability (AVX2 + FMA), GGUF magic headers (`0x46554747`), SQLite database health (`PRAGMA quick_check;`), GBNF grammar parser, local host tools (`bash`, `git`), async Tokio runtimes, and the Markdown Persona Subsystem.
- **Auto-Fix Mode (`--fix`)**: Automatically recreates missing directories (`~/.cynapse/persona`), repairs database indexes, clears stale scratch files, and updates configuration paths.

### 🎨 Visual Themes & UI Experience
- **4 Visual Color Presets**: Switch instantly between Dark Slate, Neon Cyber, Amber CRT, and Emerald Matrix themes.
- **Dynamic Input Box**: Expands downward from 3 to 8 lines as text grows, keeping long prompts completely visible.
- **Collapsible Reasoning Blocks (`Ctrl + T`)**: Expand or collapse internal model thinking streams.
- **Rich Markdown Formatting**: In-terminal syntax-highlighted code blocks, blockquotes, headers, and bullet lists.

---

## 📄 License
MIT License. Free and open-source.
