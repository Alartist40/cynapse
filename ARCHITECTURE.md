# Architecture: Cynapse-Mini

## Overview

Cynapse is a multi-crate Rust workspace implementing an offline-first AI agent system with a force-directed graph memory, multi-engine inference routing, and an Atomic-Agent-inspired architecture elevation.

## Workspace Structure

```
cynapse-mini/
├── harness/
│   ├── src/main.rs                  # CLI & TUI entrypoint dispatcher
│   ├── Cargo.toml
│   ├── cynapse-tui/                 # Ratatui visual interface
│   │   └── src/
│   │       ├── app.rs               # TUI event loop, reprompt mechanism, prompt assembly
│   │       ├── lib.rs               # TUI module wiring
│   │       └── theme.rs            # Color theme definitions
│   └── cynapse-core/                # Core agent logic
│       └── src/
│           ├── lib.rs               # Tool definitions (with concurrency_safe), path validation, execute_command, partition_tool_calls
│           ├── offline_agent.rs     # GBNF tool call validation, Zone A/B prefix compilation, LoopGuard, ToolLoopTracker
│           ├── receipts.rs          # ReadReceiptRegistry, receipt_checked_read (verified read receipts)
│           ├── bots.rs              # BotProfile/BotRegistry, allow-deny-ask permissions, workspace confinement
│           ├── subagent.rs          # SubagentManager, run_subagent_loop, approval_fn channel, stall detection
│           ├── compressor.rs        # Tool output compression (max 400 chars)
│           ├── persona.rs           # Persona file management
│           ├── session.rs           # Session persistence
│           └── doctor.rs            # System health diagnostics
├── memory/
│   └── cynapse-memory/              # Dendrite graph memory
│       └── src/
│           ├── graph.rs             # In-memory knowledge graph (NodeType, Node, Dendrite)
│           ├── context.rs           # Dendrite prompt assembly (assemble, find_relevant, score)
│           ├── store.rs             # SQLite FTS5 persistence
│           └── reflection.rs        # TurnLog persistence, ReflectionWorker
├── engine/
│   ├── cynapse-engine/              # Inference routing & provider management
│   │   └── src/
│   │       ├── lib.rs               # Tier routing, query streaming, ProviderFallbackChain, shared HTTP client
│   │       ├── daemon.rs            # llama-server daemon lifecycle
│   │       └── slots.rs             # KV-cache slot management
│   └── leafcutter_core/             # Pure Rust GGUF/Safetensors execution
│       └── rust/
│           └── llama.cpp/           # llama.cpp C++ backend
├── data/                            # Runtime data (databases, models)
├── models/                          # GGUF model storage
├── install.sh                       # Automated installer
└── Cargo.toml                       # Workspace root
```

## Core Concepts

### Two-Zone Prompt Architecture

The prompt is split into two zones for KV-cache efficiency:

**Zone A (Invariant Prefix)** — compiled once per session, byte-stable across all turns:
```
=== CYNAPSE SYSTEM PRESET ===
<operating directives>
<available tools with GBNF schemas>
```

**Zone B (Variable Tail)** — changes per turn, structured as micro-channels:
```
=== SESSION FACTS ===
<key=value facts from recent TurnLog nodes>

=== RECALLED KNOWLEDGE ===
<BM25-scored knowledge nodes>

=== MEMORY INDEX ===
<compact #id [tags] preview pointers>

=== CONVERSATION LOG ===
<recent turns>

=== NOTICE ===
<loop detector warnings, steering redirections>

=== USER INSTRUCTION ===
<current query>
```

The model receives `Zone A + Zone B` as a single prompt. Zone A is cached on the TUI (`cached_zone_a_prefix`) and invalidated only on persona mutations. Zone B is rebuilt each turn from the Dendrite graph.

### Multi-Turn Tool Loop

```
User Query → Inference → Parse Tool Call(s) → Execute → Compose Results → Reprompt → ...
```

Key invariants:
- One inference per step (no internal reasoning loops)
- `agent_step_count` incremented once per batch (max 5 steps)
- On step exhaustion: one forced finalization turn (no-tools status report) — never an abrupt stop
- `LoopGuard` checks every tool call before execution
- Batch tool arrays `[{"tool": ...}, ...]` parsed and partitioned into concurrency-safe runs / exclusive singletons
- Single reprompt with all batch results combined

### LoopGuard & ToolLoopTracker

Two-phase state machine preventing runaway tool loops:

1. **`check(tool, args)`** — prospective validation before execution
2. **`record_call(tool, args)`** — commit to history
3. **`record_outcome(tool, args, result)`** — hash result for no-progress detection
4. **`record_veto(tool, args)`** — count consecutive vetoes for circuit breaker

Detection signals:
- **Args-only repeat** (warn at 3): same tool + same args
- **No-progress streak** (critical at 5): same tool + same args + same result hash
- **Wandering** (warn at 6, escalation at 12): distinct args on `execute_command`
- **Circuit breaker** (trips at 3 consecutive vetoes): forces graceful turn end

### Provider Fallback Chain

Sticky multi-provider fallback with escalating cooldown:

```
LlamaServer (primary) → Ollama → NativeLeafcutter
```

- `resolve_provider()` returns active provider + probe flag
- `record_success()` clears override, resets breaker
- `record_failure()` escalates cooldown: 30s → 60s → 300s
- Lazy probe: after cooldown, attempt primary before staying on fallback
- Per-session isolation (future: partition by session id)

### Memory Micro-Channels

Structured memory injection into the prompt:

| Channel | Budget | Source | Content |
|---------|--------|--------|---------|
| `SESSION FACTS` | 500 tokens | `TurnLog` nodes | Key=value lines from recent turns |
| `RECALLED KNOWLEDGE` | 60% of total | BM25-scored nodes | Full node content for relevant knowledge, behind `[UNTRUSTED MEMORY]` banner |
| `MEMORY INDEX` | 300 tokens | All non-core nodes | `#id [tags] preview` compact pointers |

Budget allocation: `max_tokens = SESSION_FACTS + RECALLED + INDEX + core`.

**Injection sanitization**: all untrusted render sites pass through `neutralize_untrusted()` (triple backticks become spaced backticks, `===` becomes `= = =`) *before* token estimation, so stored nodes cannot forge channel headers or smuggle fenced tool blocks. Core identity nodes are trusted and exempt.

### Async Background Reflection

Fire-and-forget reflection worker:
- Triggered on turn completion (after `save_current_session()`)
- Extracts last 6 messages as `ReflMessage` array
- **Verified-run admission**: `spawn_reflection(msgs, turn_verified)` returns immediately when the turn had a tool error, LoopGuard veto, step exhaustion, or stream failure
- `InFlightGuard` prevents overlapping reflections
- Distills into `NodeType::Lesson`, `NodeType::Procedure`, or `NodeType::AtomicFact`
- Persists to Dendrite graph and SQLite store
- Never blocks the TUI event loop

### Verified Read Receipts

Token-saving mechanism for repeated file reads within a turn:
1. `read_file` resolves the canonical path, reads + hashes content (I/O only — no token cost on hit).
2. If the registry has a receipt for that path with an **identical hash** AND the `cynapse-read #N` anchor is verifiably present in the **last 6 messages** (the actual reprompt context window), the model receives `[File unchanged since read #N: path]` instead of the content.
3. Otherwise: full content (compressed) + post-compression `[cynapse-read #N: path | hash=...]` footer, and the receipt is refreshed.
4. The stub itself carries the anchor id, keeping it alive in the window across repeated reads; registry resets on `/clear` and `load_session`.

### Forced Finalization on Step Exhaustion

```
step_count >= 5 with new tool calls
  → warning + assistant attempt recorded (previously dropped)
  → finalization turn: === NOTICE === (do-not-call-tools + status report) 
                       + original === USER INSTRUCTION ===
  → finalization_inflight=true; reply accepted with tools_allowed=false
  → flags reset; exactly one finalization turn per exhaustion
```

### Batch Tool Execution

Model can emit array of tool calls:
```json
[{"tool": "read_file", "args": {"path": "a.txt"}}, {"tool": "grep", "args": {"query": "foo"}}]
```

Execution flow:
1. Parse array via `validate_gbnf_tool_calls`
2. Partition via `partition_tool_calls` — contiguous runs of concurrency-safe tools grouped; exclusive tools (`write_file`, `execute_command`) become singleton batches (unknown tools fail closed)
3. Iterate partitions sequentially (parallel dispatch deferred — `execute_tool_and_format` is `&mut self`):
   - `loop_guard.check(call)` → `record_call(call)` → `execute(call)` → `record_outcome(call, result)`
   - On veto: `record_veto(call)`, include veto message in results; any tool error or veto sets `turn_verified = false`
4. Combine all results into single system message
5. Single reprompt with combined output

### GBNF Tool Call Grammar

Tool calls are validated against GBNF grammar supporting:
- Direct JSON objects: `{"tool": "name", "args": {...}}`
- JSON arrays: `[{"tool": "name", "args": {...}}, ...]`
- Code fences: `` ```json {...} ``` ``
- XML tags: `<tool_call>...</tool_call>`

Parsing tries each format in order; first successful parse wins.

### Bot Profiles & Subagent Orchestration

Specialist tasks delegate to bot profiles (`bots.rs`) — TOML definitions in `~/.cynapse/bots/` carrying `tools_allow`, `tools_deny`, `tools_ask`, `workspace_restrict`, persona, and `max_concurrent`. Permission resolution is fail-closed: deny beats allow, unknown tools are rejected, unknown spawn slugs error with the available list.

`spawn_subagent` dispatches `run_subagent_loop` (`subagent.rs`): an injected `query_fn` runs up to 5 steps (plus one forced finalization turn), each step parsing GBNF tool calls, consulting `LoopGuard`, and dispatching through `execute_tool_with_profile`. Background spawns run under a concurrency semaphore (2) with results announced back into the conversation; inline spawns join on a dedicated runtime thread.

`tools_ask` tools pause background execution through an injected `approval_fn`: the loop sends a `ToolApprovalRequest` over an mpsc channel drained every event-loop tick, the `ToolApproval` modal captures a y/n decision, and a oneshot resolves the loop — approve executes via `execute_tool_with_profile_approved` (bypassing only the ask check), deny returns `[tool denied by user]`. Inline subagents (UI blocked) and stalled tasks (>120s) fail closed with steering notices.

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                     User Input                          │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Zone A (Cached Prefix)                      │
│  System directives + Tool schemas + GBNF grammar        │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Zone B (Variable Tail)                      │
│  SESSION FACTS + RECALLED KNOWLEDGE + MEMORY INDEX      │
│  + CONVERSATION LOG + NOTICE + USER INSTRUCTION         │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Provider Fallback Chain                     │
│  resolve_provider() → try_llama_server_stream           │
│  on failure → try_ollama_stream → try_native_leafcutter │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Inference (llama-server / Ollama)            │
│  slot_id: 0, cache_prompt: true, temp: 0.2             │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Parse Tool Calls                            │
│  validate_gbnf_tool_calls → Vec<ToolCall>               │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Execute Tools (Sequential)                  │
│  For each ToolCall:                                     │
│    LoopGuard.check → record_call → execute → record_outcome │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Compose Results                             │
│  Combine all tool outputs into single message           │
│  Inject LoopGuard notices as === NOTICE ===             │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Reprompt                                    │
│  Single inference with combined tool results            │
│  → continue loop or final response                      │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Background Reflection (fire-and-forget)     │
│  Distill turn into facts/procedures/lessons             │
│  Persist to Dendrite graph + SQLite                     │
└─────────────────────────────────────────────────────────┘
```

## Key Files

| File | Responsibility |
|------|---------------|
| `harness/cynapse-tui/src/app.rs` | TUI event loop, reprompt mechanism, prompt assembly, Phase 1-6 + Stage 13-21 integration (receipts wiring, finalization guard, turn_verified, partitioned dispatch, spawn_subagent dispatch, approval modal, @mention routing, /bots commands, sidebar telemetry) |
| `harness/cynapse-core/src/offline_agent.rs` | GBNF tool call validation, Zone A/B prefix compilation, LoopGuard, ToolLoopTracker |
| `harness/cynapse-core/src/lib.rs` | Tool definitions (`concurrency_safe`), path validation, `execute_command`, `partition_tool_calls`, `execute_tool_with_profile(_approved)` permission dispatch, compressor |
| `harness/cynapse-core/src/bots.rs` | BotProfile/BotRegistry, deny>allow precedence, tools_ask flags, workspace confinement, TOML seeding |
| `harness/cynapse-core/src/subagent.rs` | SubagentManager (lifecycle, cancel, semaphore), run_subagent_loop, approval_fn injection, stall detection |
| `harness/cynapse-core/src/receipts.rs` | ReadReceiptRegistry, hash-verified read receipts with context-window anchor checks |
| `engine/cynapse-engine/src/lib.rs` | Tier routing, query streaming, `ProviderFallbackChain`, shared HTTP client |
| `engine/cynapse-engine/src/daemon.rs` | llama-server daemon lifecycle |
| `engine/cynapse-engine/src/slots.rs` | KV-cache slot management |
| `memory/cynapse-memory/src/context.rs` | Dendrite prompt assembly, micro-channel partitioning, BM25 scoring |
| `memory/cynapse-memory/src/graph.rs` | In-memory knowledge graph, NodeType enum |
| `memory/cynapse-memory/src/reflection.rs` | ReflectionWorker, TurnLog persistence |
| `memory/cynapse-memory/src/store.rs` | SQLite FTS5 persistence |

## Testing

```bash
# Run all workspace tests
cargo test --workspace

# Run specific crate tests
cargo test -p cynapse-core
cargo test -p cynapse-memory
cargo test -p cynapse-engine

# Run with output
cargo test --workspace -- --nocapture
```

Current test count: **281 passed, 0 failed** (core 44, engine 8, memory 16, tui 14, leafcutter 197, e2e 1, doctest 1).

Acceptance gates G1-G22 all passed — see `GATES.md`.

## Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# System health check
./target/release/cynapse doctor
```
