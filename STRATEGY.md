# Cynapse-Mini Strategy & Audit Log

> **Purpose**: Living document tracking error audits, performance issues, and strategic direction.
> **Convention**: Each audit round replaces the previous. Unresolved items persist, resolved items are removed.

---

## Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| Core Framework | ✅ Stable | All 196 tests pass, clean working tree |
| TUI/CLI | ✅ Stable | Persona editor, slash menu, theme system working |
| Memory (Dendrite) | ✅ Stable | FTS5+BM25, relevance gating, reflection |
| Engine Routing | ❌ Broken | 35B model classified as Tier 1 |
| Inference Speed | ❌ Critical | 1-2 tok/s vs Ollama's 11 tok/s |
| Streaming Quality | ⚠️ Suboptimal | No NDJSON, no buffering optimization |

---

## Round 1 Audit (2026-09-09)

### Critical Issues

#### 1. Engine Routing Broken — 35B Model Classified as Tier 1

**File**: `engine/cynapse-engine/src/lib.rs:267-305`

**Root Cause**: Routing logic only checks `needed_bytes <= ram_bytes` (total RAM), not available RAM. For the 35B model:
- Model: 21.7GB + RESERVE_BYTES: 1.5GB = **23.2GB needed**
- System RAM: **15GB total, ~6.6GB available**
- `23.2GB <= 15GB` is false, BUT `prefer_gpu` is checked first

The routing doesn't parse GGUF metadata to determine model parameter count. A 35B model needs Tier 2 (layer streaming) regardless of `prefer_gpu`.

**Impact**: 35B model loads into RAM, thrashes, runs at 1-2 tok/s instead of using layer streaming.

**Fix Required**:
```rust
pub fn route_model(model_path: &Path, prefer_gpu: bool) -> RouteDecision {
    // Parse GGUF metadata for parameter count
    let param_count = detect_parameter_count_from_gguf(model_path);
    
    let tier = if is_safetensors {
        EngineTier::Tier3LargeSafetensor
    } else if param_count > 20_000_000_000 {  // >20B params
        EngineTier::Tier2LargeGguf  // Always Tier2 for large models
    } else if prefer_gpu || needed_bytes <= ram_available_bytes {
        EngineTier::Tier1Fast
    } else {
        EngineTier::Tier2LargeGguf
    };
}
```

---

#### 2. Inference Speed Critical — 1-2 tok/s vs 11 tok/s

**Root Causes**:

**A. No Batch Inference** (`engine/leafcutter_core/rust/src/api/mod.rs`)
```rust
// Current: Single token generation
for _ in 0..max_tokens - 1 {
    let mut logits = self.forward_native(&[next_token])?;  // ← One token at a time!
}
```
Each token requires a full forward pass through 60+ layers. Ollama/llama.cpp uses `llama_decode()` with batch processing.

**B. Prefetch Disabled for Large Models** (`engine/leafcutter_core/rust/src/api/mod.rs:950-974`)
```rust
let use_prefetch = match std::env::var("LEAFCUTTER_PREFETCH").ok().as_deref() {
    Some("0") | Some("false") => false,
    Some("1") | Some("true") => true,
    _ => {
        let total_ram = crate::detect::probe_hardware().ram_total_mb;
        let model_mb = (self.model.file.file_size_bytes() / (1024 * 1024)) as u64;
        total_ram >= model_mb  // ← 15GB >= 21.7GB = false!
    }
};
```
35B model (21.7GB) doesn't fit in 15GB RAM → prefetch disabled → layers loaded sequentially from disk.

**C. RESERVE_BYTES Too Small** (`lib.rs:9`)
```rust
const RESERVE_BYTES: u64 = 1536 * 1024 * 1024;  // 1.5GB
```
For 35B model:
- 21.7GB weights
- 4-8GB KV cache (35B, 4096 context)
- 2-4GB activations
- **Total: 27-33GB**
- 1.5GB reserve doesn't account for KV cache → thrashing

**D. No GPU Offloading**
Leafcutter native engine is CPU-only. Ollama uses llama.cpp with CUDA/Metal via `-ngl 99`.

**Impact**: 5-10x slower than Ollama on same hardware.

---

#### 3. Streaming Quality Suboptimal

**File**: `engine/cynapse-engine/src/lib.rs:555-612`

**Issues**:
- Uses `bytes_stream()` with manual newline parsing
- No NDJSON protocol (Ollama uses `application/x-ndjson`)
- No streaming buffer optimization (Ollama uses 64KB initial, 8MB max)
- No channel-based pipeline (Ollama decouples inference from HTTP response)

**Ollama's Approach**:
```go
// Server: NDJSON streaming
c.Header("Content-Type", "application/x-ndjson")
c.Stream(func(w io.Writer) bool {
    val, ok := <-ch
    bts, _ := json.Marshal(val)
    bts = append(bts, '\n')
    w.Write(bts)
    return true
})
```

**Impact**: Token delivery less smooth, higher latency perception.

---

### Status of Round 1 Audit Items

| ID | Issue | Priority | Status | Resolution / Notes |
|----|-------|----------|--------|---------------------|
| R1-01 | Engine routing broken for large models | Critical | ✅ Resolved | Fixed in `cynapse-engine::route_model` & `detect::choose_tier`. 35B models route strictly to Tier 2 Layer Streaming. |
| R1-02 | No batch inference in native engine | Critical | In Progress | Prompt prefill evaluated in batches; token-by-token generation with sliding window. |
| R1-03 | Prefetch disabled for large models | High | In Progress | Layer cache sliding-window enabled for models exceeding RAM. |
| R1-04 | RESERVE_BYTES too small | High | ✅ Resolved | Implemented `compute_reserve_bytes()` dynamically scaling 1.0GB–8.0GB based on model size. |
| R1-05 | No GPU offloading | High | Open | FFI / llama-server bridge provides GPU offload where available. |
| R1-06 | Streaming not NDJSON | Medium | ✅ Resolved | Buffered NDJSON line parsing with chunk channels integrated into `query_tier1_stream`. |

---

## Strategic Direction

### Phase 1: Fix Routing & Performance (Immediate)

**Goal**: Make 35B model route correctly and run at acceptable speed.

1. **Fix routing logic** — Parse GGUF metadata for parameter count, apply size-based heuristics
2. **Dynamic RESERVE_BYTES** — Scale reserve based on model size (8GB for >10GB models)
3. **Enable prefetch for large models** — Override RAM check when model is >15GB
4. **Integrate llama.cpp as primary backend** — Already exists as `FfiEngine`, needs proper wiring

### Phase 2: Engine Architecture (Short-term)

**Goal**: Match Ollama's performance characteristics.

1. **llama.cpp FFI Integration**
   - Use existing `FfiEngine` for Tier 1 instead of HTTP subprocess
   - Enable GPU offloading via CUDA/Metal
   - Implement batch inference via `llama_decode()`

2. **NDJSON Streaming Protocol**
   - Match Ollama's `application/x-ndjson` format
   - Implement channel-based pipeline (inference → channel → HTTP response)
   - Add 64KB/8MB buffer sizing

3. **KV Cache Management**
   - Implement prompt caching (`cache_prompt: true`)
   - Add context shifting for long conversations
   - VRAM-based context sizing

### Phase 3: Advanced Features (Medium-term)

**Goal**: Exceed Ollama's capabilities with Colibri-inspired architecture.

1. **Tiered Memory Hierarchy** (from Colibri)
   - Treat VRAM/RAM/disk as single memory hierarchy
   - Per-layer LRU cache with learned pin decisions
   - PILOT prefetch (router-lookahead thread)

2. **Batch-Union Optimization** (from Colibri)
   - Read each unique expert once per batch, not once per token
   - Critical for MoE models

3. **Speculative Decoding**
   - MTP draft heads for 2-3x speedup
   - Grammar-forced drafts for structured output

4. **Multi-Backend Execution** (from llama.cpp)
   - Abstract backend interface (CPU/CUDA/Metal/Vulkan)
   - Automatic backend selection via hardware detection
   - Graph reuse across ubatches

### Phase 4: Production Hardening (Long-term)

**Goal**: Enterprise-grade reliability and performance.

1. **Process Isolation** (from Ollama)
   - Spawn llama-server as subprocess for crash isolation
   - HTTP communication layer for reliability
   - Dynamic GPU backend loading

2. **Memory Management**
   - VRAM prediction and eviction
   - Semaphore-based concurrency control
   - Automatic context sizing based on available memory

3. **Monitoring & Telemetry**
   - Per-device VRAM accounting
   - Routing heat maps (Colibri `.coli_usage` style)
   - Performance regression testing

---

## Reference Architecture Comparison

### What Makes Ollama Fast

| Feature | Ollama | Cynapse | Gap |
|---------|--------|---------|-----|
| Inference Backend | llama.cpp (C++) | Leafcutter (Rust) | Ollama has mature SIMD/GPU kernels |
| Batch Processing | `llama_decode()` with ubatch | Single token forward pass | 5-10x slower |
| GPU Offloading | CUDA/Metal via `-ngl` | CPU-only | No GPU acceleration |
| KV Cache | Paged attention, prompt caching | Basic | Missing optimization |
| Streaming | NDJSON with channel pipeline | Manual bytes_stream | Less smooth |
| Process Isolation | Subprocess per model | In-process | Crash risk |
| Memory Management | VRAM prediction, eviction | Static RESERVE_BYTES | Thrashing |

### What Makes Colibri Fast (for Large Models)

| Feature | Colibri | Cynapse | Gap |
|---------|---------|---------|-----|
| Memory Tiering | VRAM/RAM/disk hierarchy | RAM only | No disk streaming |
| Expert Caching | Per-layer LRU + learned pins | None | No caching |
| Prefetch | PILOT (router-lookahead) | Disabled for large models | No overlap |
| Batch-Union | Read expert once per batch | N/A (no MoE) | Not implemented |
| Speculative Decoding | MTP heads | None | No speedup |

### What Makes AirLLM Work (for Memory-Constrained)

| Feature | AirLLM | Cynapse | Gap |
|---------|--------|---------|-----|
| Hook-Based Streaming | Pre/post hooks per module | None | Not implemented |
| Disk-as-Memory | Stream weights on-demand | Load entire model | Memory inefficient |
| Pinned Memory Prefetch | 2GB pinned for faster copy | None | No optimization |

---

## Performance Targets

| Metric | Current | Ollama | Target |
|--------|---------|--------|--------|
| 9B Model (Q4_K_M) | ~2-3 tok/s | ~11 tok/s | 10+ tok/s |
| 35B Model (Q4_K_M) | ~1-2 tok/s | ~5-7 tok/s | 5+ tok/s |
| First Token Latency | 5-10s | 1-2s | <2s |
| Streaming Smoothness | Choppy | Smooth | Smooth |
| Memory Usage | Thrashing | Stable | Stable |

---

## Next Action Items

1. [ ] Parse GGUF metadata for parameter count in routing
2. [ ] Implement dynamic RESERVE_BYTES based on model size
3. [ ] Enable prefetch for large models with override flag
4. [ ] Wire FfiEngine as primary backend for Tier 1
5. [ ] Implement NDJSON streaming protocol
6. [ ] Add prompt caching to KV cache
7. [ ] Benchmark 9B model vs Ollama baseline
8. [ ] Document GPU offloading requirements

---

## Lessons Learned

1. **Never trust `prefer_gpu` flag** — Parse actual hardware capabilities
2. **RAM check must use available, not total** — `/proc/meminfo` MemAvailable
3. **Large models need dynamic reserves** — 1.5GB is insufficient for 35B
4. **Batch inference is non-negotiable** — Single-token forward pass is 5-10x slower
5. **NDJSON beats manual parsing** — Match Ollama's streaming protocol
6. **Process isolation matters** — In-process inference risks entire TUI crash
7. **GPU offloading is table stakes** — CPU-only is unacceptable for production

---

*Last Updated: 2026-09-09*
*Audit Round: 1*
*Next Review: After Phase 1 completion*
