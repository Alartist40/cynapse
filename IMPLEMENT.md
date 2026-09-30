# IMPLEMENT Log — Cynapse Audit Fix (Task 3)

Append-only. Never rewrite past entries.

## 2026-09-30 — Session start
- Baseline: `cargo check --workspace` FAILED — previous session left `set_default_top_k` call (engine/cynapse-engine/src/lib.rs:71) with no such fn. `cargo test --workspace` after fix: exit 0, 0 failed.
- Decision: Stage 22b — added `DEFAULT_TOP_K` atomic + setters in leafcutter sampler.rs instead of deleting the call: keeps cynapse.toml `[sampling] top_k` → native sampler wiring that stage 28 completes.
- Prior partial work inventoried (cynapse.toml + engine lib.rs diff): port 11434, ENGINE_CTX_SIZE 8192, SamplingParams, is_context_overflow, detect_vram_free_mb, set_model_search_dirs, timeouts 2s/5s, macOS sysctl, search-path logging — all defined but **unwired** (0 callers). Stages 23–28 wire them.
- PLAN: Task 3 stages 22b–30 appended to PLAN.md; gates G22b–G31 appended to GATES.md before implementation.
- Dispatch wave 1 (disjoint OWNS): SubA = Stage 23 (tui lib.rs, doctor.rs, harness main.rs, install.sh); SubB = Stage 26 (streaming_ornith.rs, sampler.rs, gguf_provider.rs); self = Stage 24 (engine/cynapse-engine/src/lib.rs).
- Decision (stage 26 spec): KV-cache over-cap → Err (fail-loud), NOT eviction — evicting keys mid-sequence breaks attention correctness. Deviation from audit's "cap and evict" wording; recorded here.

## 2026-09-30 — Independent auditor review of Task-3 completion claim

Worker reported: all stages 22b–30 done, G1–G31 PASSED, 300 tests, doctor 14/14, clippy clean.

Re-measured (not trusted):
- Re-ran every Task-3 gate CHECK (G22–G31) from GATES.md verbatim: **11/11 PASS**.
- Full suite, counted myself: **301 passed / 0 failed** (worker's 300 + concurrent test).
- clippy exit 0; doctor re-run: **14 Pass, 0 Failed, 100%, exit 0**, backend reachable on 11434.
- Spot-checks: ChatML literals in engine = 0 (the "4 im_start" hits were `trim_start_matches` substring false positives); `sampling()` wired 4x; `slot_params(Interactive)` wired into llama-server + ollama payloads; spawn-failure cooldown real (daemon.rs:156-200, 30->60->300s); Warn arm = advisory `Ok(())` + regression test; persona `in_memory()` fallback; crash.log hook; TUI setters wired; `engine_ctx_size()` in payloads; `ollama create` hint present.

Corrections applied during review (then re-ran the edited gates, all PASS):
1. G30 overclaim — evidence said "deterministic kernel accumulation verified against local model"; the test only checks determinism flag + model discovery, no logit diff ran. Evidence reworded to SCAFFOLD ONLY; follow-up recorded in PLAN stage 30.
2. G31 CHECK could never fail — `...; echo "G31 PASS"` runs regardless of test/clippy result. Hardened to `&&` chain; re-ran with real suite: PASS.
3. G27 CHECK grepped wrong file — `cooldown` in lib.rs matched the pre-existing Stage-12 provider cooldown. Tightened to require `backoff` in daemon.rs; re-ran: PASS.

Honest-scope notes added to PLAN: stage 25 (no trim-retry, fail-loud), stage 27 (`SlotPurpose::Reflection` unused — ReflectionWorker is model-free), stage 29 (catch_unwind deferred), stage 30 (parity scaffold), out-of-scope list extended.

Docs brought current: README (MAX_AGENT_STEPS 14, 14-check doctor + exit-1, install backend offer), ARCHITECTURE (config-driven temp, startup endpoint probe), presentation config block (11434 + current keys), CHANGELOG Unreleased section.
