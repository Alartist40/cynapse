# 📜 Cynapse Changelog

All notable changes to the Cynapse AI Agent System are documented in this file.

## [1.9.0] - 2026-09-06

### 🚀 Tier-1 Parallel Speed Acceleration & Markdown Persona Engine (`/persona`)
- **Parallel GEMV Kernel Acceleration**: Fixed single-threaded matrix multiplication bottleneck in quantization kernels (`q4_k_gemm`, `q6_k_gemm`, `q5_k_gemm`, `iq4_nl_gemm`) by lowering the Rayon multi-core execution threshold from `n >= 4096` to `n >= 256`. Single-row GEMV operations for key, value, query, and FFN projections across all sub-4096 layer dimensions now run multi-threaded across all CPU cores.
- **CPU SMT Core Optimization**: Optimized `default_thread_count()` in `init.rs` to fully utilize physical CPU core capacity on multi-core Ryzen/Intel hardware.
- **Markdown Persona Management System (`cynapse-core::persona`)**: Implemented `PersonaManager` handling markdown system persona files (`IDENTITY.md`, `SOUL.md`, `USER.md`, `SYSTEM.md`) stored in `~/.cynapse/persona/`. Seeded default personas for direct, concise, non-generic responses.
- **Interactive TUI `/persona` Slash Command**: Integrated `/persona` slash command into `cynapse-tui` supporting `/persona`, `/persona list`, `/persona load <name>`, `/persona set <name>`, `/persona show`, and `/persona default`.
- **Cynapse Doctor Audit Expansion (Check 9)**: Added Check 9 (`check_persona_subsystem`) to `CynapseDoctor` auditing markdown persona directory health, file integrity, and dynamic system prompt compilation.

## [1.8.0] - 2026-09-06

### 🌐 Universal HuggingFace Model Auto-Detection & Execution Engine
- **Universal Architecture Support**: Extended `ModelArchitecture` enum and `detect()` in `leafcutter_core` to auto-detect and run HuggingFace GGUF models across 20+ model families (`SmolLM`, `Starcoder2`, `CommandR`, `Granite`, `InternLM`, `Baichuan`, `ChatGLM`, `MiniMax`, `MPT`, `DeepSeek`, `Llama`, `Qwen`, `Mistral`, `Gemma`, `Phi`, `Yi`, `Nemotron`, `Falcon`).
- **Dynamic Key Fallback & Auto-Binding**: Added fuzzy case-insensitive architecture parser and robust fallback layer mappings so unknown or custom HuggingFace GGUF architectures map cleanly to transformer layer execution graphs.
- **Universal Model Resolver Keywords**: Expanded `resolve_model_tag()` in `cynapse-engine` with keywords (`smollm`, `starcoder`, `command`, `granite`, `internlm`, `baichuan`, `chatglm`, `minimax`, `falcon`, `yi`, `nemotron`, `cohere`) for instant resolution of custom HuggingFace filenames.
- **GGUF Header Inspector**: Added `inspect_gguf_header()` in `cynapse-core` to inspect GGUF magic bytes, version header, tensor counts, and metadata KV entries.

## [1.7.0] - 2026-09-06

### 🌿 Pure Native Leafcutter Engine Transition (Zero External Dependencies)
- **Ollama Dependency Removal**: Disconnected Cynapse's inference runner from external Ollama daemons (`http://127.0.0.1:11434`). Cynapse now operates completely self-contained using its in-house **Leafcutter Rust Engine** (`engine/leafcutter_core/rust`).
- **Direct GGUF Storage Placement**: Stream downloader places downloaded HuggingFace `.gguf` files directly into `./models/` and `~/.cynapse/models/` without executing `ollama create` CLI commands.
- **Native GGUF Directory Scanning**: `fetch_native_models()` scans local `./models/` directories and native endpoints, dynamically resolving GGUF files on disk.
- **Cynapse Doctor Audit Upgrade**: Updated `cynapse doctor` (`check_llm_endpoint_and_models`) to audit the Cynapse Native Leafcutter Engine and verify local GGUF catalog readiness.
- **In-House Memory Unload**: `/unload`, `/stop`, and `/free` slash commands invoke `cynapse_engine::unload_model()` to clear model weight allocations directly without external process calls.

## [1.6.0] - 2026-09-06

### 🩺 Cynapse Self-Healing Doctor System (`cynapse doctor` & `/doctor`)
- **Expanded Subsystem Diagnostics**: Added Tier-1 LLM Engine Endpoint & Model Tag Registration alignment check (`check_llm_endpoint_and_models`). Cynapse Doctor now audits local HTTP runner connectivity (`http://localhost:11434`), checks registered Ollama tags, and compares them with local `.gguf` files in `~/.cynapse/models`.
- **Interactive TUI `/doctor` Command**: Integrated `/doctor`, `/doc`, and `/heal` slash commands into the TUI to trigger immediate live diagnostics and automatic background self-healing.

### ⚡ Tier-1 Payload Standardisation & Error Body Extraction
- **HTTP 400 Payload Fix**: Standardized JSON request payload in `query_tier1_stream()` with standard fields (`num_ctx`, `temperature`), eliminating HTTP 400 Bad Request errors caused by non-standard `options` parameters on Ollama endpoints.
- **Detailed Error Body Extraction**: Extracted full HTTP error response text from LLM engine responses (`res.text().await`), displaying exact error descriptions from Ollama / llama.cpp / vLLM servers instead of generic status strings.
- **Model Tag Fallback Safety**: Fixed model tag resolution so selecting local GGUF models preserves the target filename instead of forcibly overriding to `available_tags[0]`.

### 📋 Universal Terminal Paste Engine (`Ctrl+V` & Bracketed Paste)
- **Bracketed Paste Mode**: Enabled `EnableBracketedPaste` and `DisableBracketedPaste` in `TuiRuntimeGuard` for seamless terminal mouse right-click, Shift+Insert, and paste menu operations.
- **System Clipboard Fallback**: Added `Ctrl+V` keyboard shortcut handling interfacing with `wl-paste` (Wayland) and `xclip` (X11), pasting URLs and prompts directly into the active input field.

### 🌌 3D Galaxy Atlas & Downloader Enhancements
- **Complete Orbit Pause**: Replaced frame-based ticks with `galaxy_anim_spin`, ensuring 100% of stars, cluster centers, and links freeze completely when auto-spin is paused (`Spacebar`/`s`).
- **Sticky Download Modal**: Kept download progress modal visible on HTTP errors or completion with explicit user dismissal options (`Esc`/`q`/`Enter`).

---

## [1.5.0] - 2026-09-06

### 📥 Enhanced HuggingFace Model Downloader & Custom Repo Resolution
- **Custom Model Entry in Curated Catalog**: Added an explicit `[ 🔗 Custom Hugging Face Model... ]` entry to the curated download list, selectable via arrow keys or keyboard shortcuts (`c` / `Tab`).
- **Flexible HuggingFace Link & Repo Handling**: Supports pasting full HuggingFace direct file links (converting `/blob/main/` to `/resolve/main/` automatically), repository IDs (e.g. `Qwen/Qwen2.5-7B-Instruct-GGUF`, `TheBloke/Llama-2-7B-GGUF`), or custom URLs.
- **Quantization Selector Expansion**: Expanded target quantization options to include `Q4_K_M`, `Q5_K_M`, `Q8_0`, `F16`, `Q2_K`, `Q3_K_M`, `Q6_K`, and `IQ4_NL`.
- **Automatic Model Rescanning & Auto-Activation**: rescan models automatically upon download completion and update active model quant, size, and source metadata immediately.
- **Unified Persistent Storage (`~/.cynapse/models`)**: Standardized model storage location to `~/.cynapse/models` across all execution contexts, ensuring downloaded models are instantly recognized regardless of current working directory.

### 🛡️ GBNF Batch Tools, Agent Loop Safeguards & Retry Jitter
- **GBNF Batch Tool Call Parser**: Upgraded `validate_gbnf_tool_calls()` to parse single JSON tool objects and batch arrays (`[{"tool": "...", "args": {...}}]`).
- **Agent Tool Step Limiter**: Added maximum step depth limit (`MAX_AGENT_STEPS = 5`) to prevent infinite autonomous tool loops.
- **User Prompt Context Preservation**: Retains original user query intent when re-prompting model after tool execution (`User Request: {}\n\nTool Result for {}:\n{}\n\nContinue resolution.`).
- **Exponential Backoff Jitter**: Added 0..100ms random jitter calculation to 3x exponential backoff retries in `query_tier1_stream()`.

---

## [1.4.0] - 2026-09-05

### 🩺 Cynapse Self-Healing Doctor System (`cynapse doctor` & TUI `/doctor`)
- **Comprehensive Diagnostic Engine (`cynapse_core::doctor`)**: Audits 9 critical subsystems including host hardware RAM safety headroom, SIMD/AVX2 processor acceleration, GGUF magic header integrity (`0x46554747`), SQLite & FTS5 database integrity (`PRAGMA quick_check;`), GBNF JSON schema tool-call engine, Atomic-Agent local tools (`bash`, `git`), and Tokio async channels.
- **Automatic Self-Healing (`--fix` & `r`/`F5`)**: Automatically repairs corrupted or missing database tables/schemas, recreates missing storage directories (`./models`, `data/`, `~/.cynapse`), clears temporary scratch files, and fixes model symlinks.
- **Interactive TUI `/doctor` Modal Dashboard**: Rich Ratatui TUI dashboard rendering real-time health scores (`[ 100% HEALTHY ]`), pass/warn/repaired/fail badges, and instant `r`/`F5` trigger key for background healing.
- **CLI Subcommand**: Standardized command line entrypoint `cynapse doctor [--fix]` displaying high-contrast terminal diagnostic dashboard.

---

## [1.3.0] - 2026-09-05

### 📥 Atomic-Agent Inspired HuggingFace Model Downloader (`/pull` & `/download`)
- **Hardware Recommendation Engine**: Autodetects host RAM/CPU via `probe_hardware_info()` and highlights optimal models (`[★ Recommended]`) matching host RAM headroom (Qwen2.5-0.5B for low RAM, Qwen2.5-3B for 8GB, Qwen2.5-7B for 12-16GB, Gemma-4-12B for 16GB+).
- **Custom HuggingFace Write-In**: Users can type or paste any HuggingFace repository identifier or URL (`TheBloke/Llama-2-7B-GGUF`, `unsloth/gemma-4-12B-it-qat-GGUF`).
- **Quantization Selector**: Select quantization level (`Q4_K_M`, `Q5_K_M`, `Q8_0`, `F16`).
- **Non-Blocking Background Downloader**: Tokio background streaming task emitting progress updates (`speed_mbps`, `downloaded_bytes`, `total_bytes`, `pct`).
- **Live Progress Bar Widget**: Displays live progress bar (`[██████░░░░] 60% (14.2 MB/s) - 245.0 / 400.0 MB`) inside TUI modal.
- **Auto-Registration**: Downloaded GGUF models are saved directly to `./models` / `~/.cynapse/models` and automatically registered in Cynapse's active model router.

---

## [1.2.0] - 2026-09-05

### 🌌 Colibri-Inspired Multi-Galaxy 3D Dendrite Memory Topology
- **Central Core Center Star**: Rendered a central supermassive core star (`✸`) at `(0,0,0)` serving as the galaxy center of mass.
- **Sub-Galaxy Category Clusters**: Grouped graph nodes into spinning sub-galaxies (`Personal`, `Engineering`, `Preferences`, `Meta`, `Episodic`, `Transient Oort Cloud`) orbiting the central star.
- **Category Color-Coding**: Assign distinct colors to clusters: Pink/Magenta (Personal), Cyan (Engineering), Yellow (Preferences), Green (Meta), White (Episodic), Dark Gray (Transient).
- **Specialization Index Node Sizing**: Scaled star symbols and brightness using normalized entropy specialization metrics ($\text{spec} > 0.75$ rendered as bright `★` / `✦`).
- **Dynamic Synapse Links & Oort Cloud**: Transient/unlinked memories render in an outer "Oort Cloud" halo before promotion. Added 3D midpoint coordinate projection drawing link markers (`·`) between connected memory nodes.

### 🗄️ Interactive Dendrite Memory Inspector Drawer (`Tab` Key / `/drawer`)
- **`Tab` Key Modal Overlay**: Pressing `Tab` opens an interactive Memory Drawer modal for inspecting, filtering, and managing active graph nodes.
- **Node Deletion (`d` / `Delete`)**: Delete selected memory nodes directly from the TUI interface with instant database sync.
- **Metadata Badging**: Displays category badges, specialization scores (`spec:0.85`), and hashtags for every graph entry.

### 📜 Scroll Position & Viewport Overflow Badge
- **Scroll Percentage Badge**: Rendered a live scroll percentage badge (`[▲ Scroll 45% ▼]`) on the right border of the Conversation Viewport header when scrolled.

### 🛡️ Atomic-Agent Offline Capabilities Suite
- **Zero-Syntax-Error GBNF Grammar Validator**: Implemented `validate_gbnf_tool_call()` to enforce strict JSON tool call schema validation offline.
- **Stream Event LoopGuard Intercept**: Wired GBNF schema parsing and circular `LoopGuard` checks into `poll_stream_events()` stream completion to prevent infinite loops.
- **Stable Prefix KV-Cache Preservation**: Formatted system prompt and tool definitions at a fixed head position (`=== CYNAPSE SYSTEM PRESET ===`), ensuring 100% KV-cache hit rate on local GGUF/llama.cpp engines.
- **Two-Tier Hybrid Memory Recall**: Upgraded context assembly scoring in `context.rs` combining BM25 term matches, specialization index boost, and exponential recency decay ($\gamma^{\Delta t}$).
- **Sidebar Pipeline Badges**: Updated Left Sidebar status cards showing `FTS5`, `BM25 + Spec Ranker`, `GBNF Schema Check`, `4k RAG Budget`.
- **Zero-Warning Clean Build**: Verified 196 workspace unit tests and resolved compiler warnings for a 100% clean release build.

---

## [1.1.0] - 2026-09-05

### ⌨️ Dynamic Input Auto-Layout & Word-by-Word Backspace (`Ctrl + Backspace`)
- **Word Deletion (`Ctrl + Backspace` / `Ctrl + W` / `Ctrl + H`)**: Implemented `delete_word_backward` to delete whole words backward cleanly without typing stray `h` characters or breaking prompt text.
- **Downward Box Auto-Layout Expansion**: Prompt input box dynamically expands downward (from 3 to 8 lines) as text grows (`Constraint::Length(input_height)`).
- **Line Word Wrapping**: Enabled `.wrap(Wrap { trim: false })` on the input `Paragraph` so multi-line prompts wrap cleanly onto lines 2, 3, 4, etc.
- **Multi-Line Cursor Tracking**: Updated terminal blinking cursor positioning (`f.set_cursor_position`) with exact line (`row_offset`) and column (`col_offset`) calculations.

### 🖼️ Complete 32-Line ASCII Banner Welcome Screen
- **Full 32-Line Brand Logo**: Replaced truncated 8-line snippet with the complete 32-line ASCII artwork from `ascii-art.txt`.
- **Welcome Display State**: Displays the full ASCII artwork centered upon launch or when clearing chat history (`/clear`).
- **Automatic Viewport Transition**: The ASCII banner cleanly recedes when sending your first prompt, focusing 100% on chat history, thinking blocks, and responses.

### 🗂️ Collapsible Reasoning & Chain-of-Thought Stream Cards (`/thinking` & `Ctrl + T`)
- **Collapsible Thinking Cards**: Model chain-of-thought (`[Thinking...]`) blocks can be collapsed into a compact single-line badge (`▶ [Thinking... (Collapsed)]`) or expanded (`▼ [Thinking...]`).
- **Toggle Shortcuts**: Added `Ctrl + T` keyboard shortcut and `/thinking` slash command.

### 📊 Live Tool Execution Progress Cards
- **Execution Pipeline Panel**: Real-time status cards in the Left Sidebar showing FTS5 search (`✓ Active`), BM25 ranker (`✓ BM25`), RAG context budget (`✓ 4k Budget`), and streaming engine state (`• Running...` / `✓ Idle`).

### 🎨 In-Terminal Rich Markdown & Syntax Highlighting
- **Rich Markdown Parser**: Render Markdown headers (`#`, `##`, `###`), bullet points (`•`), blockquotes (`│`), and fenced code blocks with language badges (`┌── [ RUST ] ───`, `└───`) and green syntax styling.

### 🧠 Paraclea-Style Memory RAG & Keyboard Scrolling
- **Line-by-Line Viewport Scrolling**: Added keyboard scrolling with `Up` and `Down` arrow keys when autocomplete dropdown is closed.
- **Paraclea Memory Architecture**: Sanitized node content (`clean_node_content`) stripping internal tags (`Target: [[...]]`), excluding raw `TurnLog` nodes from RAG prompts, and injecting clean atomic facts (`AtomicFact`), preferences, concepts, and identity nodes into system prompts.

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
  - Animated synapse pulse stream loading frames (`Generating ・>・・`, `Generating ・・>・`, `Generating ・・・>`).

### 🌌 3D Dendrite Memory Galaxy Visualizer (`/memory`)
- Implemented a 3D spherical galaxy projection mapping Dendrite memory nodes into a 3D coordinate space projected onto the terminal grid.
- Interactive camera controls: `Left`/`Right`/`Up`/`Down` arrow keys rotate 3D Yaw/Pitch in real-time, `Spacebar`/`s` toggles auto-spin.
- Color-coded node tier clusters (`★` Summaries, `✪` Procedures, `●` Facts, `.` Halo Logs).

### 🔍 Interactive Slash Command Autocomplete Dropdown (`/` Menu)
- Added floating popup dropdown when typing `/` in prompt bar.
- Filters available commands (`/model`, `/memory`, `/thinking`, `/theme`, `/session`, `/clear`, `/help`, `/exit`).
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
