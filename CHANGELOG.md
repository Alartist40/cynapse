# Changelog

All notable changes to Cynapse are documented here.

## [Unreleased] — Architecture Elevation (2026-09-16)

### Added

#### Phase 1: Stable Zone A Prefix & KV-Cache Slot Affinity
- `cached_zone_a_prefix` on `TuiApp` — compiles Zone A system prompt once, reuses across all multi-turn tool loops.
- `invalidate_zone_a_prefix()` hooks on all 6 persona mutation points.
- Slot affinity: `slot_id: 0` + `cache_prompt: true` for all query paths (llama-server, Ollama primary, Ollama retry).

#### Phase 2: Two-Phase ToolLoopTracker & LoopGuard
- `ToolLoopTracker` two-phase state machine: `check()` → `record_call()` → `record_outcome()` / `record_veto()`.
- `LoopGuard` wrapper delegating to `ToolLoopTracker`.
- No-progress streak detection with result hashing.
- Wandering detector for `execute_command` tools.
- Circuit breaker on consecutive vetoes.
- Thresholds: warn=3, critical=5, breaker_veto_streak=3, wandering=6, wandering_escalation=12.
- 5 unit tests: args-only repeat, no-progress streak, breaker, wandering, different args breaks streak.

#### Phase 3: Memory Micro-Channels & Async Reflection
- Refactored `assemble()` in `context.rs` into structured micro-channels:
  - `=== SESSION FACTS ===` (500 token budget, priority-sorted TurnLog entries).
  - `=== RECALLED KNOWLEDGE ===` (60% of budget, BM25-scored nodes).
  - `=== MEMORY INDEX ===` (300 token budget, compact `#id [tags] preview` pointers).
- Lexical query tokenization with punctuation stripping in `find_relevant()` and `score()`.
- Zone B header handling in `compile_zone_b_tail` detecting micro-channel prefixes.
- `ReflectionWorker` with fire-and-forget background distillation on turn completion.
- `InFlightGuard` for overlap protection.
- 2 new tests: `test_assemble_micro_channels`, `test_assemble_skip_core_nodes`.

#### Phase 4: Batch Tool Call Array Execution
- `validate_gbnf_tool_calls` (plural) integrated in `app.rs` for array `[{"tool": ...}, ...]` parsing.
- Sequential multi-tool batch execution with per-call `LoopGuard` checking and outcome recording.
- Combined tool results into single turn message and single reprompt.
- `agent_step_count` incremented once per batch, not per tool.
- `all_vetoed` flag suppresses reprompt when every tool in batch was vetoed.
- Unit test `test_batch_tool_calls_in_codeblocks`.

#### Phase 5: Structured Prompt Tail (`=== NOTICE ===`)
- `compile_zone_b_tail` accepts `notice: Option<&str>` parameter.
- `=== NOTICE ===` section placed immediately before `=== USER INSTRUCTION ===`.
- Loop intervention warnings injected as notices from `LoopGuard` veto results.
- Test verifying notice presence and ordering.

#### Phase 6: Provider Fallback Chain
- `ProviderFallbackChain` struct with sticky override, escalating cooldown (30s → 60s → 300s), lazy probe.
- `ProviderKind` enum: `LlamaServer`, `Ollama`, `NativeLeafcutter`.
- `shared_fallback_chain()` singleton via `OnceLock<Mutex<...>>`.
- `resolve_provider()` returning `(ProviderKind, is_probe)`.
- `record_success()` / `record_failure()` for breaker management.
- Modularized streaming: `try_llama_server_stream`, `try_ollama_stream`, `try_native_leafcutter_stream`.
- `query_tier1_stream` wired to consult fallback chain with candidate iteration.
- Test: `test_provider_fallback_chain` covering resolve, failure, success, escalation.

### Fixed (earlier stages)
- Reprompt stream receiver race condition in `poll_stream_events`.
- Zone A prefix unification across conversational and tool turns.
- Dendrite memory context reconnection with 1500 token budget.
- Format string interpolation in `assemble()`.
- Symlink traversal path validation.
- Shared HTTP client via `OnceLock<reqwest::Client>`.

---

## [0.1.0] — Initial Release

- Force-directed 3D graph memory with SQLite FTS5 and BM25 ranking.
- Multi-engine routing: llama.cpp, Ollama, Leafcutter native.
- GBNF grammar-constrained tool call output.
- Interactive TUI with Ratatui.
- Model downloader with HuggingFace integration.
- System persona manager.
- Self-healing doctor diagnostics.
- 4 visual themes.
