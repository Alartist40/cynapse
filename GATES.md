# Acceptance Ledger: Cynapse Overhaul & Stabilization

- [x] G1: Repository bloat cleaned and build targets fresh
  CHECK: test ! -f cynapse && test ! -d engine/leafcutter_core/rust/target && echo "G1 PASS"
  EXPECT: G1 PASS
  EVIDENCE: Removed top-level binary and duplicate leafcutter target dir; step dumps deleted; gitignore updated.

- [x] G2: Workspace compiles cleanly with cargo check
  CHECK: cargo check --workspace && echo "G2 PASS"
  EXPECT: G2 PASS
  EVIDENCE: Finished dev profile [unoptimized + debuginfo] in 26.99s with exit code 0.

- [x] G3: Tool security boundaries enforced in cynapse-core
  CHECK: cargo test -p cynapse-core -- --nocapture && echo "G3 PASS"
  EXPECT: G3 PASS
  EVIDENCE: 9 passed; 0 failed. Path traversal blocked, destructive commands rejected, 10s command execution timeout enforced.

- [x] G4: Memory store and graph search tests pass with fixed fallback
  CHECK: cargo test -p cynapse-memory -- --nocapture && echo "G4 PASS"
  EXPECT: G4 PASS
  EVIDENCE: 5 passed; 0 failed. Non-FTS fallback queries dendrite_nodes via LIKE, graph upsert reduced to 1 relaxation step.

- [x] G5: Entire workspace tests pass
  CHECK: cargo test --workspace && echo "G5 PASS"
  EXPECT: G5 PASS
  EVIDENCE: 211 total unit & integration tests across leafcutter, cynapse-core, cynapse-memory, cynapse-tui, and cynapse passed (0 failed).

- [x] G6: Dynamic tier routing, Paraclea memory gating, theme-adaptive ASCII, and Jcode spinner
  CHECK: cargo test --workspace && echo "G6 PASS"
  EXPECT: G6 PASS
  EVIDENCE: 216 tests passed (0 failed). Model size vs RAM properly routes 35B to Tier 2 and small to Tier 1; MEMORY.md supported in ~/.cynapse/persona; theme ASCII styled; Jcode 10-frame Braille spinner active; cynapse doctor 100% health score.

- [x] G7: Atomic-Agent architecture elevation: Two-zone prompt, result compression, slot pinning, and procedural memory
  CHECK: cargo test --workspace && ~/.local/bin/cynapse doctor && echo "G7 PASS"
  EXPECT: G7 PASS
  EVIDENCE: 226 tests passed (0 failed). Two-Zone Prompt Architecture (Zone A invariant prefix + Zone B dynamic tail) active; tool results compressed via compress_tool_result; SlotManager pins slot 0 for chat and slot 1 for reflection; LlamaServerDaemon process supervisor operational; NodeType::Lesson and NodeType::Procedure memory distillation verified; cynapse doctor 100% health score.

- [x] G8: Agent Loop Stability & System Hardening: Stream race fix, Dendrite reconnection, symlink security, and connection pool reuse
  CHECK: cargo test --workspace && echo "G8 PASS"
  EXPECT: G8 PASS
  EVIDENCE: 228 tests passed (0 failed). Reprompt stream receiver preserved across tool cycles; Zone A prefix unified; Dendrite memory context & TurnLog persistence wired; symlink path traversal blocked; shared reqwest client active.

- [x] G9: Two-Phase ToolLoopTracker & LoopGuard Hardening: Prospective checks, no-progress streak tracking, wandering detection, and circuit breaker
  CHECK: cargo test -p cynapse-core && cargo test --workspace && echo "G9 PASS"
  EXPECT: G9 PASS
  EVIDENCE: 231 tests passed (0 failed). Prospective check evaluation (+1 thresholding), consecutive no-progress streak detector, wandering exploration detector, and circuit breaker verified.

- [x] G10: Memory Micro-Channels & Async Reflection: Structured Zone B, punctuation-stripped search, and out-of-band reflection
  CHECK: cargo test -p cynapse-memory && cargo test --workspace && echo "G10 PASS"
  EXPECT: G10 PASS
  EVIDENCE: 233 tests passed (0 failed). SESSION FACTS, RECALLED KNOWLEDGE, and MEMORY INDEX micro-channels active; ReflectionWorker fire-and-forget background distillation wired on turn completion.

- [x] G11: Batch Tool Call Array Execution: Multi-tool parsing, sequential execution, and unified reprompt
  CHECK: cargo test --workspace && echo "G11 PASS"
  EXPECT: G11 PASS
  EVIDENCE: 234 tests passed (0 failed). validate_gbnf_tool_calls batch parsing integrated in app.rs; individual LoopGuard tracking; single-pass reprompting verified.

- [x] G12: Structured Prompt Tail: === NOTICE === tail injection for loop detector and steering
  CHECK: cargo test --workspace && echo "G12 PASS"
  EXPECT: G12 PASS
  EVIDENCE: 235 tests passed (0 failed). compile_zone_b_tail notice positioning verified; notice injected before USER INSTRUCTION; veto notices composed from LoopGuard interventions.

- [x] G13: Provider Fallback Chain: Wired multi-provider fallback with escalating cooldown and candidate iteration
  CHECK: cargo test --workspace && target/release/cynapse doctor && echo "G13 PASS"
  EXPECT: G13 PASS
  EVIDENCE: 242 tests passed (0 failed). ProviderFallbackChain wired into query_tier1_stream with resolve_provider(), record_success(), record_failure(); candidate iteration tries LlamaServer -> Ollama -> NativeLeafcutter; escalating cooldown 30s -> 60s -> 300s; lazy probe recovery; doctor 92% score.
