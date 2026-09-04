# 📜 Cynapse Changelog

All notable changes to the Cynapse AI Agent System are documented in this file.

---

## [1.0.0] - 2026-09-05

### 🚀 Major Architecture Overhaul (Pure Rust Zero-Dependency Engine)
- **100% Pure Rust Architecture**: Removed all Python and Node.js wrapper scripts and replaced them with a modular Rust workspace (`cynapse-core`, `cynapse-engine`, `cynapse-memory`, `cynapse-tui`, `leafcutter`).
- **3-Tier Hardware & Model Router**:
  - **Tier 1 (Fast Engine)**: Tokio async HTTP streaming runner querying local llama.cpp / Ollama endpoints (`reqwest` streaming).
  - **Tier 2 (Large GGUF)**: Leafcutter Pure Rust GGUF Layer Streaming Core.
  - **Tier 3 (Large Safetensor)**: Leafcutter Pure Rust Safetensor Core.
- **Dendrite 4-Tier Memory Graph**:
  - Implemented SQLite FTS5 full-text indexing + BM25 relevance ranker.
  - Tiered node topology: Tier 3 Summary (#summary), Tier 2 Procedure (#procedure), Tier 1 Fact (#fact), Tier 0 TurnLog (#transcript).

### 🎨 TUI Visual & Aesthetic Overhaul (Colibri & Jcode Inspired)
- **Colibri-Style Left Sidebar Panel**:
  - Moved sidebar to the **LEFT** side of the terminal layout (26% width).
  - Real-time System Hardware Telemetry: RAM used/total + progress bar (`[████░░░░] 42%`), CPU brand & core count, GPU acceleration status.
  - Model details (`Name`, `Quant`, `Size`, `Source`).
  - Active visual theme manager (`/theme`).
  - Dendrite memory graph stats (live nodes & edges count).
- **Jcode-Inspired Background ASCII Art**:
  - Centered ASCII art banner (`ascii-art.txt`) rendered inside the Conversation Viewport when chat history is empty/welcome state.
- **Smooth Rounded Borders (`BorderType::Rounded`)**:
  - Replaced square box corners (`┌ ┐ └ ┘`) with smooth rounded corners (`╭ ╮ ╰ ╯`) across all panels, headers, input bar, dropdown, and modals.
- **Clean Professional Typography**:
  - Removed all emojis and icons for a clean, minimalist typography (`User:`, `Cynapse:`, `[Thinking...]`, `[Response]:`, `Info:`, `Error:`).
  - Simplified top header to `" CYNAPSE TUI "`.
  - Prompt input prefix: `・> `.
  - Animated synapse pulse stream loading frames (`⚙️ Generating ・>・・`, `⚙️ Generating ・・>・`, `⚙️ Generating ・・・>`).

### 🌌 3D Dendrite Memory Galaxy Visualizer (`/memory`)
- Implemented a 3D spherical galaxy projection mapping Dendrite memory nodes into a 3D coordinate space projected onto the terminal grid.
- Interactive camera controls: `Left`/`Right`/`Up`/`Down` arrow keys rotate 3D Yaw/Pitch in real-time, `Spacebar`/`s` toggles auto-spin.
- Color-coded node tier clusters (`*` Summaries, `+` Procedures, `o` Facts, `.` Halo Logs).

### 🔍 Interactive Slash Command Autocomplete Dropdown (`/` Menu)
- Added floating popup dropdown when typing `/` in prompt bar.
- Filters available commands (`/model`, `/memory`, `/theme`, `/session`, `/clear`, `/help`, `/exit`).
- Interactive keyboard navigation: `Up`/`Down` arrows navigate, `Tab`/`Right Arrow` auto-completes, `Enter` executes.

### 💾 Persistent Session Management (`~/.cynapse/sessions/`)
- Persistent JSON transcript storage saving session data across runs.
- Interactive session manager modal (`/session`) to list and resume past conversations.
- Resume from CLI: `cynapse --resume <session_id>`.

### ⚡ Engine & Model Tag Resolution Fixes
- Added workspace directory model scanning (`/home/xander/Documents/portfolio/cynapse-mini/models`), automatically detecting local `.gguf` models (e.g. `qwen2.5-0.5b-instruct-q4_k_m.gguf`).
- Implemented smart tag matching in `query_tier1_stream()` to resolve local filenames to registered server tags, eliminating 404 HTTP errors.
- RAII Terminal Protection (`TuiRuntimeGuard`) ensuring terminal state is safely restored on panic or exit.
- Verified workspace stability with 196 passing unit tests.
