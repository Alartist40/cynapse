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

## Acceptance Gates
Defined in `GATES.md` (G1 through G7: ALL PASSED).
