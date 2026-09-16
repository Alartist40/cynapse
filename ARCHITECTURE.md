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
│           ├── lib.rs               # Tool definitions, path validation, execute_command
│           ├── offline_agent.rs     # GBNF tool call validation, Zone A/B prefix compilation, LoopGuard, ToolLoopTracker
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
- `LoopGuard` checks every tool call before execution
- Batch tool arrays `[{"tool": ...}, ...]` parsed and executed sequentially
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
| `RECALLED KNOWLEDGE` | 60% of total | BM25-scored nodes | Full node content for relevant knowledge |
| `MEMORY INDEX` | 300 tokens | All non-core nodes | `#id [tags] preview` compact pointers |

Budget allocation: `max_tokens = SESSION_FACTS + RECALLED + INDEX + core`.

### Async Background Reflection

Fire-and-forget reflection worker:
- Triggered on turn completion (after `save_current_session()`)
- Extracts last 6 messages as `ReflMessage` array
- `InFlightGuard` prevents overlapping reflections
- Distills into `NodeType::Lesson`, `NodeType::Procedure`, or `NodeType::AtomicFact`
- Persists to Dendrite graph and SQLite store
- Never blocks the TUI event loop

### Batch Tool Execution

Model can emit array of tool calls:
```json
[{"tool": "read_file", "args": {"path": "a.txt"}}, {"tool": "grep", "args": {"query": "foo"}}]
```

Execution flow:
1. Parse array via `validate_gbnf_tool_calls`
2. Iterate sequentially:
   - `loop_guard.check(call)` → `record_call(call)` → `execute(call)` → `record_outcome(call, result)`
   - On veto: `record_veto(call)`, include veto message in results
3. Combine all results into single system message
4. Single reprompt with combined output

### GBNF Tool Call Grammar

Tool calls are validated against GBNF grammar supporting:
- Direct JSON objects: `{"tool": "name", "args": {...}}`
- JSON arrays: `[{"tool": "name", "args": {...}}, ...]`
- Code fences: `` ```json {...} ``` ``
- XML tags: `<tool_call>...</tool_call>`

Parsing tries each format in order; first successful parse wins.

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
| `harness/cynapse-tui/src/app.rs` | TUI event loop, reprompt mechanism, prompt assembly, Phase 1-6 integration |
| `harness/cynapse-core/src/offline_agent.rs` | GBNF tool call validation, Zone A/B prefix compilation, LoopGuard, ToolLoopTracker |
| `harness/cynapse-core/src/lib.rs` | Tool definitions, path validation, `execute_command`, compressor |
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

Current test count: **242 passed, 0 failed**.

## Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# System health check
./target/release/cynapse doctor
```
