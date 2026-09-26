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

- **Stage 13: Verified Read Receipts (`cynapse-tui`, `cynapse-core`) [DONE]**
  `OWNS: harness/cynapse-tui/src/app.rs, harness/cynapse-core/src/lib.rs, harness/cynapse-core/src/receipts.rs`
  - Created `ReadReceiptRegistry` and `receipt_checked_read` in `harness/cynapse-core/src/receipts.rs`.
  - Extracted `pub(crate) fn read_file_at` in `harness/cynapse-core/src/lib.rs`.
  - Wired into `TuiApp` at `execute_tool_and_format`: anchors on `cynapse-read #id` within the last 6 messages (`take(6)` reprompt window).
  - Skips re-reading unchanged files with `[File unchanged since read #<id>: <path>]` stub.
  - Reset on `load_session` and `/clear`.
  - Verified with 5 unit tests in `receipts.rs` and workflow unit test in `app.rs`.

- **Stage 14: Forced Finalization on Step Exhaustion (`cynapse-tui`) [DONE]**
  `OWNS: harness/cynapse-tui/src/app.rs`
  - Added `finalization_inflight: bool` flag to `TuiApp` state and constructor.
  - Implemented `finalization_notice(max_steps: usize)` notice composer.
  - Replaced abrupt stop at `agent_step_count >= MAX_AGENT_STEPS` with a single finalization turn injecting the budget exhaustion directive before user instruction.
  - Guarded `StreamEvent::Done` with `tools_allowed = !self.finalization_inflight` to strictly disallow tool execution on finalization replies and reset flags.
  - Reset `finalization_inflight = false` on every new user turn.
  - Verified with 2 unit tests (`test_finalization_notice_no_tool_directive` and `test_finalization_prompt_structure`).

- **Stage 15: Fence Neutralization on Memory Injection (`cynapse-memory`) [DONE]**
  `OWNS: memory/cynapse-memory/src/context.rs`
  - Source: gawkbot `internal/team/scoped_memory.go` (wrapUntrustedMemoryBrief).
  - Before injecting RECALLED KNOWLEDGE / MEMORY INDEX content into the prompt, neutralize code-fence and `===` header delimiters inside recalled content and wrap it as explicitly untrusted data (`neutralize_untrusted`).
  - Preserve channel header structure owned by `assemble()`; only mutate inner recalled payloads.
  - Added untrusted memory reference banner to `=== RECALLED KNOWLEDGE ===`.
  - Unit tests: `test_neutralize_untrusted_blocks_spoofing` and `test_assembled_prompt_cannot_be_spoofed_by_node_content`.

- **Stage 16: Parallel-Safe Tool Classification (`cynapse-core`, `cynapse-tui`) [DONE]**
  `OWNS: harness/cynapse-core/src/lib.rs, harness/cynapse-tui/src/app.rs`
  - Source: nanobot `agent/tools/execution.py` (concurrency_safe/exclusive partitioning).
  - Added `concurrency_safe: bool` to `ToolDefinition` and `is_tool_concurrency_safe` in `cynapse-core/src/lib.rs` (`read_file: true`, `grep: true`, `write_file: false`, `execute_command: false`).
  - Implemented `partition_tool_calls`: contiguous runs of safe tools grouped into batches, exclusive tools placed into singleton batches; batch structure created as the seam for future parallel dispatch (execution is currently sequential — `execute_tool_and_format` is `&mut self`, parallelization deferred).
  - Preserved per-call `LoopGuard` check/record ordering within each batch in `app.rs`.
  - Unit tests: `test_tool_concurrency_classification`, `test_partition_tool_calls_mixed`, `test_batch_partitioning_execution`.

- **Stage 17: Verified Reflection Admission (`cynapse-memory`, `cynapse-tui`) [DONE]**
  `OWNS: memory/cynapse-memory/src/reflection.rs, harness/cynapse-tui/src/app.rs`
  - Source: gawkbot `internal/team/task_distill.go` (verification-gated distillation).
  - Added `verified: bool` admission gate to `ReflectionWorker::spawn_reflection`.
  - Wired `turn_verified: bool` on `TuiApp` to track execution quality across turns; resets on user turn, flips false on tool error, loop guard veto, step exhaustion, or stream failure.
  - Low-confidence/failed turns skip memory distillation to prevent polluting procedural memory with failed patterns.
  - Unit tests: `test_reflection_admission_gate_unverified_skips`, `test_reflection_admission_gate_verified_runs`, `test_turn_verified_status_and_reflection_admission`.

## Acceptance Gates
Defined in `GATES.md` (G1 through G18: ALL PASSED).
