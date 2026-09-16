# CYNAPSE — Offline-First Agent System & Synaptic Knowledge Memory

**Cynapse** is a high-performance, offline-first AI agent platform engineered entirely in 100% pure Rust. Designed for total privacy, zero external runtime dependencies, and instant local execution, it bridges host hardware telemetry with an intelligent multi-engine router, a dynamic synaptic knowledge graph, and an Atomic-Agent-inspired architecture elevation.

At the heart of Cynapse lies a unified architecture where three powerful engines work in concert: **Dendrite**, a force-directed 3D graph memory system with Coulomb/Hooke physics, SQLite FTS5 full-text indexing, and BM25 relevance ranking; **Leafcutter**, a custom pure-Rust GGUF and Safetensors execution core providing local tensor streaming with KV cache trunk reservation; and **llama.cpp / Ollama integration**, acting as a high-speed Tier-1 HTTP inference runner with NDJSON streaming and multi-provider fallback. Together, these subsystems provide a complete local agent harness capable of autonomous tool execution, strict GBNF grammar constrained outputs, real-time memory synthesis, 3D orbital galaxy memory visualization, and self-healing diagnostics—all running locally on your hardware without a single byte leaving your machine.

---

## Installation & Setup

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

## Launch Modes

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

---

## Interactive Slash Commands

Type `/` in the prompt bar to trigger the floating command menu:

| Command | Description |
|---------|-------------|
| `/help` | Display interactive keyboard shortcuts & help menu |
| `/model` | Open interactive model selector & scanner |
| `/pull` | Download GGUF models from HuggingFace (curated catalog & custom URLs) |
| `/persona` | Manage agent personality markdown files (`IDENTITY`, `SOUL`, `USER`, custom `.md`) |
| `/doctor` | Launch Cynapse Doctor self-healing diagnostic dashboard |
| `/memory` | Launch 3D Orbital Galaxy Memory Atlas visualizer |
| `/drawer` | Open interactive Dendrite Memory drawer inspector |
| `/thinking` | Toggle collapsible model reasoning/thinking stream blocks |
| `/theme` | Cycle color themes (Dark Slate, Neon Cyber, Amber CRT, Emerald Matrix) |
| `/session` | Manage and resume saved conversation sessions |
| `/clear` | Reset conversation view and restore brand banner |
| `/exit` | Quit Cynapse TUI |

### Terminal Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Open Dendrite Memory Drawer inspector from anywhere |
| `Up` / `Down` | Scroll conversation viewport line-by-line |
| `Ctrl+Backspace` / `Ctrl+W` | Delete word backward in prompt input |
| `Ctrl+T` | Toggle model thinking/reasoning blocks |
| `Ctrl+A` / `Ctrl+E` | Move input cursor to start / end of line |
| `Ctrl+U` | Clear input line |
| `Spacebar` / `s` (in `/memory`) | Toggle 3D Galaxy auto-rotation |
| `r` / `F5` (in `/doctor`) | Re-run self-healing diagnostics with auto-repair |

---

## Architecture

Cynapse is structured as a decoupled multi-crate Rust workspace. See [ARCHITECTURE.md](ARCHITECTURE.md) for the full technical breakdown.

```
cynapse-mini/
├── harness/
│   ├── src/main.rs             # CLI & TUI entrypoint dispatcher
│   ├── cynapse-tui/            # Ratatui visual interface, modals, 3D visualizer
│   └── cynapse-core/           # Tool execution, GBNF parser, session manager, doctor
├── memory/
│   └── cynapse-memory/         # Dendrite graph engine, SQLite FTS5 store, BM25 ranker
├── engine/
│   ├── cynapse-engine/         # Semantic hardware router, Tier-1 LLM client, provider fallback
│   └── leafcutter_core/        # Pure Rust GGUF & Safetensors tensor kernels
└── install.sh                  # Dual remote/local automated installer & setup script
```

---

## Capabilities

### Synaptic Memory & 3D Galaxy Memory Atlas
Memory in Cynapse is modeled after biological neural networks with a force-directed 3D physics engine. Ideas, facts, and conversation turn logs form nodes connected by weighted synaptic links that strengthen with use and decay with disuse over time.
- **Force-Directed 3D Galaxy Physics**: Real Coulomb repulsion, Hooke spring attraction, central gravity toward the supermassive core, velocity damping (0.88), collision avoidance, and force clamping.
- **3D Orbital Galaxy Visualizer (`/memory`)**: Visualizes memory nodes as stars organized into galactic clusters revolving around a supermassive central core.
- **Two-Tier Hybrid Memory Recall**: Merges SQLite FTS5 keyword matching with BM25 scoring, specialization index weighting, and exponential temporal decay.
- **Structured Memory Micro-Channels**: Session facts, recalled knowledge, and memory index injected as structured prompt sections.

### Atomic-Agent Architecture Elevation
Six architecture phases inspired by the Atomic-Agent reference implementation:
- **Stable Zone A Prefix**: Byte-stable system prompt cached across multi-turn tool loops for KV-cache reuse.
- **Two-Phase LoopGuard**: Prospective loop detection with no-progress streak tracking, wandering detection, and circuit breaker.
- **Batch Tool Dispatch**: Array tool call parsing `[{"tool": ...}, ...]` with sequential per-call guard evaluation.
- **Notice Injection**: `=== NOTICE ===` prompt tail for loop detector and steering redirections.
- **Provider Fallback Chain**: Sticky multi-provider fallback (llama-server -> Ollama -> NativeLeafcutter) with escalating cooldowns (30s -> 60s -> 300s) and lazy probe recovery.

### Offline Agent Capabilities & GBNF Grammar Engine
- **NDJSON Streaming**: Polymorphic streaming parser supporting multiple API formats (Ollama, vLLM, generic).
- **GBNF Grammar Constraints**: Enforces valid JSON tool-call schema syntax, preventing output formatting panics offline.
- **KV-Cache Slot Affinity**: Slot 0 pinned for chat, slot 1 for reflection, with `cache_prompt: true` for warm cache reuse.
- **Max Step Safeguards**: Configurable step limits (`MAX_AGENT_STEPS = 5`) to prevent runaway recursive tool execution.

### Async Background Reflection
Fire-and-forget reflection worker distills conversation turns into atomic facts, procedures, and lessons without blocking the UI. Runs on a dedicated background thread after each turn completion.

### Offline Model Downloader & Model Management (`/pull`)
- **Curated Recommendations**: Probes system RAM and GPU VRAM to tag optimal model sizes.
- **Custom HuggingFace Downloads**: Download any GGUF model from HuggingFace with real-time progress.
- **Background Async Progress**: Non-blocking Tokio downloader with real-time speed and progress bars.

### System Persona Manager (`/persona`)
- **Interactive TUI Modal & Live Editor**: Inspect, edit, and switch persona `.md` files in `~/.cynapse/persona/`.
- **Dynamic Prompt Compiler**: Injects direct system instructions into every model prompt without generic LLM headers.

### Cynapse Doctor Self-Healing Engine (`/doctor`)
- **11-Subsystem Auditing**: RAM headroom, SIMD availability, GGUF magic headers, SQLite health, GBNF grammar parser, local tools, Tokio runtimes, persona subsystem, API connectivity, endpoint health, model availability.
- **Auto-Fix Mode (`--fix`)**: Automatically recreates missing directories, repairs database indexes, clears stale files.

### Visual Themes & UI Experience
- **4 Visual Color Presets**: Dark Slate, Neon Cyber, Amber CRT, Emerald Matrix.
- **Dynamic Input Box**: Expands from 3 to 8 lines as text grows.
- **Collapsible Reasoning Blocks (`Ctrl+T`)**: Expand or collapse internal model thinking streams.
- **Rich Markdown Formatting**: In-terminal syntax-highlighted code blocks, blockquotes, headers, and bullet lists.

---

## Verification

| Metric | Value |
|--------|-------|
| Tests passed | 242 |
| Tests failed | 0 |
| Build | `cargo build --release` succeeds |
| System doctor | 92% health (11 Pass, 1 Warn, 0 Failed) |
| Acceptance gates | G1-G13 all PASSED |

---

## License
MIT License. Free and open-source.
