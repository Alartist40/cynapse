# PLAN: Cynapse System Hardening & Bloat Reduction

## Task
Harden security, fix TUI UTF-8 input crashes, integrate native Leafcutter routing, fix memory search and bloat, and eliminate repository deadweight.

## Contract & File Ownership
- **Stage 1: Repository Bloat Cleanup [DONE]**
  `OWNS: .gitignore, engine/leafcutter_core/rust/hf_step_*.json, cynapse`
- **Stage 2: Tool Sandboxing & Downloader (`cynapse-core`) [DONE]**
  `OWNS: harness/cynapse-core/src/lib.rs, harness/cynapse-core/src/downloader.rs`
- **Stage 3: Memory Store & Context Refactor (`cynapse-memory`) [DONE]**
  `OWNS: memory/cynapse-memory/src/store.rs, memory/cynapse-memory/src/graph.rs, memory/cynapse-memory/src/context.rs`
- **Stage 4: Engine Routing & Config (`cynapse-engine`, `main.rs`, `cynapse-tui/src/lib.rs`) [DONE]**
  `OWNS: engine/cynapse-engine/src/lib.rs, harness/src/main.rs, harness/cynapse-tui/src/lib.rs`
- **Stage 5: TUI UTF-8 Input Stability & Banner Fix (`cynapse-tui`) [DONE]**
  `OWNS: harness/cynapse-tui/src/app.rs, ascii-art.txt`
- **Stage 6: Engine Auto-Routing, Paraclea Memory Gating, Theme-Adaptive ASCII Logo, Jcode Spinner [DONE]**
  `OWNS: harness/cynapse-core/src/persona.rs, engine/cynapse-engine/src/lib.rs, memory/cynapse-memory/src/context.rs, harness/cynapse-tui/src/theme.rs, harness/cynapse-tui/src/app.rs`
- **Stage 7: Atomic-Agent Architecture Elevation [DONE]**
  `OWNS: harness/cynapse-core/src/compressor.rs, harness/cynapse-core/src/lib.rs, engine/cynapse-engine/src/lib.rs, engine/cynapse-engine/src/daemon.rs, engine/cynapse-engine/src/slots.rs, memory/cynapse-memory/src/graph.rs, memory/cynapse-memory/src/store.rs, harness/cynapse-tui/src/app.rs`
  - Phase 1: Two-zone byte-stable prompt prefix & disciplined sampling (`temp: 0.2`, `top_p: 0.95`, `repeat_penalty: 1.1`).
  - Phase 2: Tool output compressor (`compress_tool_result` - max 400 chars, tail extraction, error signature).
  - Phase 3: Managed local `llama-server` child daemon & pure Rust `SlotManager` with `cache_prompt: true`.
  - Phase 4: Procedural memory (`NodeType::Lesson`, `NodeType::Procedure`) & async background reflection.

- **Stage 8: Agent Loop Stability & System Hardening (`cynapse-tui`, `cynapse-core`, `cynapse-memory`, `cynapse-engine`) [DONE]**
  `OWNS: harness/cynapse-tui/src/app.rs, harness/cynapse-core/src/lib.rs, harness/cynapse-core/src/offline_agent.rs, harness/cynapse-core/src/compressor.rs, memory/cynapse-memory/src/context.rs, engine/cynapse-engine/src/lib.rs`
  - Fixed reprompt stream receiver dropping race in `poll_stream_events`.
  - Unified Zone A byte-stable prefix across all turns to eliminate KV-cache evictions.
  - Reconnected Dendrite memory context with 1500 token budget and automatic TurnLog persistence into SQLite store and graph.
  - Corrected conversation history windowing.
  - Fixed format string interpolation in `assemble()`.
  - Hardened path validation against symlink bypasses.
  - Added HTTP connection pool reuse via shared `reqwest::Client`.
  - Enhanced multi-line diagnostic error signatures in compressor.

- **Stage 9: Two-Phase ToolLoopTracker & LoopGuard Hardening (`cynapse-core`, `cynapse-tui`) [DONE]**
  `OWNS: harness/cynapse-core/src/offline_agent.rs, harness/cynapse-tui/src/app.rs`
  - Upgraded `ToolLoopTracker` to two-phase state machine (`check` -> prospective validation, `record_call` -> dispatch, `record_outcome` -> result hash tracking, `record_veto` -> breaker tracking).
  - Added consecutive no-progress streak detection with result hashing and intervening call reset.
  - Added wandering detector for exploratory command loops (`is_wandering_prone`).
  - Added circuit breaker on repeated tool vetoes (`is_breaker_tripped`).
  - Integrated `LoopGuard` into TUI reprompt event pipeline with prospective intervention checks.

- **Stage 10: Memory Channels & Async Reflection (`cynapse-memory`, `cynapse-core`, `cynapse-tui`) [DONE]**
  `OWNS: memory/cynapse-memory/src/context.rs, harness/cynapse-core/src/offline_agent.rs, harness/cynapse-tui/src/app.rs`
  - Refactored `assemble()` in `context.rs` into structured micro-channels (`=== SESSION FACTS ===`, `=== RECALLED KNOWLEDGE ===`, `=== MEMORY INDEX ===`).
  - Added robust punctuation stripping to lexical search and scoring in `context.rs`.
  - Added micro-channel header detection in `compile_zone_b_tail`.
  - Wired `ReflectionWorker::spawn_reflection` to fire-and-forget background reflection on chat turn completion in `TuiApp`.

- **Stage 11: Batch Tool Call Array Execution (`cynapse-core`, `cynapse-tui`) [DONE]**
  `OWNS: harness/cynapse-tui/src/app.rs, harness/cynapse-core/src/offline_agent.rs`
  - Integrated `validate_gbnf_tool_calls` in `app.rs` to support multi-tool arrays and single tool objects seamlessly.
  - Implemented sequential multi-tool batch execution with per-call `LoopGuard` checking and outcome recording.
  - Combined multi-tool results into a unified turn result and single reprompt.
  - Added unit test `test_batch_tool_calls_in_codeblocks`.

- **Stage 12: Structured Prompt Tail & Provider Fallback Chain (`cynapse-core`, `cynapse-engine`, `cynapse-tui`) [DONE]**
  `OWNS: harness/cynapse-core/src/offline_agent.rs, engine/cynapse-engine/src/lib.rs, harness/cynapse-tui/src/app.rs`
  - Added `=== NOTICE ===` structured prompt tail support to `compile_zone_b_tail`, placed immediately before `=== USER INSTRUCTION ===`.
  - Injected loop intervention warnings and steering notices into the reprompt prompt tail.
  - Implemented `ProviderFallbackChain` in `cynapse-engine` with sticky override, escalating cooldown (30s -> 60s -> 300s), lazy probe, and `shared_fallback_chain()`.
  - Modularized streaming into `try_llama_server_stream`, `try_ollama_stream`, `try_native_leafcutter_stream`.
  - Wired `query_tier1_stream` to consult `shared_fallback_chain().resolve_provider()` with automatic success/failure recording and candidate iteration.

## Acceptance Gates
Defined in `GATES.md` (G1 through G13: ALL PASSED).
