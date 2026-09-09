# Cynapse-Mini Strategy & Audit Log

> **Purpose**: Living document tracking error audits, performance issues, and strategic direction.
> **Convention**: Each audit round replaces the previous. Unresolved items persist, resolved items are removed.

---

## Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| Core Framework | ✅ Stable | All 196 tests pass, clean working tree |
| TUI/CLI | ✅ Stable | Persona editor, slash menu, theme system working |
| Engine Routing | ✅ Fixed | 35B routes to Tier 2, dynamic headroom scaling |
| Memory (Dendrite) | ⚠️ Partial | Graph structure real, galaxy visualization is decorative |
| Inference Speed | ❌ Critical | 1-2 tok/s vs Ollama's 11 tok/s (still unresolved) |
| Streaming Quality | ⚠️ Suboptimal | Buffered but not true NDJSON |

---

## Round 1 Audit — Resolved Items

| ID | Issue | Status | Resolution |
|----|-------|--------|------------|
| R1-01 | Engine routing broken for 35B | ✅ Resolved | `route_model` now uses `model_gb > 20.0 && needed_bytes > ram_bytes` gate |
| R1-04 | RESERVE_BYTES too small | ✅ Resolved | `compute_reserve_bytes()` scales 1.0–8.0 GB by model size |
| R1-06 | Streaming not NDJSON | ⚠️ Partial | Buffered line parsing, but not true NDJSON protocol |

---

## Round 2 Audit (2026-09-09) — Engine Changes

### Verified: What Was Actually Implemented

**1. Dynamic Headroom Scaling** (`lib.rs:9-20`) — ✅ Correct
```rust
pub fn compute_reserve_bytes(model_bytes: u64) -> u64 {
    let model_gb = model_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if model_gb <= 3.0 { 1GB }
    else if model_gb <= 9.0 { 2.5GB }
    else if model_gb <= 20.0 { 5GB }
    else { 8GB }
}
```
Properly scales reserve for KV cache + activations. 35B model gets 8GB reserve.

**2. Routing Logic** (`lib.rs:306-314`) — ✅ Correct
```rust
let tier = if is_safetensors {
    EngineTier::Tier3LargeSafetensor
} else if model_gb > 20.0 && needed_bytes > ram_bytes {
    EngineTier::Tier2LargeGguf  // ← 35B goes here
} else if needed_bytes <= ram_bytes || (prefer_gpu && model_gb <= 12.0) {
    EngineTier::Tier1Fast
} else {
    EngineTier::Tier2LargeGguf
};
```
The `model_gb > 20.0` gate prevents large models from entering Tier 1.

**3. Leafcutter detect.rs** (`detect.rs:270-312`) — ✅ Correct
`choose_tier` mirrors the same dynamic reserve logic. Consistent with lib.rs.

**4. Doctor Check** (`doctor.rs:121-159`) — ⚠️ Incomplete
RAM check uses static thresholds (8GB/3GB/1GB) but doesn't reference the dynamic reserve tiers. Works but doesn't reflect actual model routing logic.

**5. Main.rs Route Command** (`main.rs:79-92`) — ✅ Correct
Passes `prefer_gpu: false` to `route_model`, so routing is purely size-based.

### What Was NOT Implemented (Contrary to Commit Message)

| Claimed | Actual |
|---------|--------|
| "NDJSON streaming pipeline" | Still `bytes_stream()` with manual newline parsing at `lib.rs:579-616` |
| "Buffered NDJSON line parsing with chunk channels" | Buffer is just `String::new()`, no 64KB/8MB sizing |
| "Layer cache sliding-window enabled" | No change to leafcutter engine code |

---

## Round 2 Audit — Dendrite Memory System

### Question: Is the Galaxy View Legitimate?

**Answer: NO. It is decorative, not functional.**

The 3D galaxy renderer (`app.rs:2442-2603`) creates a visual impression of a galaxy but does NOT implement the gravitational/clustering model you described.

#### What the Code Actually Does

**Central Core** (`app.rs:2462-2470`):
- Fixed yellow `✸` at coordinates (0,0,0). Hardcoded. Not computed from node importance.

**Category Orbits** (`app.rs:2495-2510`):
```rust
let category_clusters = [
    (Meta, 6.0, 0.0 + anim_spin, ...),      // Fixed distance 6.0
    (Preferences, 10.0, 1.2 + anim_spin, ...), // Fixed distance 10.0
    (Personal, 14.0, 2.4 + anim_spin, ...),    // Fixed distance 14.0
    (Engineering, 18.0, 3.6 + anim_spin, ...), // Fixed distance 18.0
    (Episodic, 22.0, 4.8 + anim_spin, ...),    // Fixed distance 22.0
    (Transient, 26.0, 5.8 + anim_spin, ...),   // Fixed distance 26.0
];
```
Each category has a **predetermined orbit radius**. There is NO gravitational computation. The "pull" is just a fixed number.

**Node Placement** (`app.rs:2515-2557`):
```rust
let local_radius = 2.0 + ((idx % 4) as f32 * 1.5);
let local_angle = (idx as f32 * 1.4) + anim_spin * 1.5;
```
Nodes are placed in a **spiral pattern within their category** using modular arithmetic. Position depends on `idx` (insertion order), NOT on content similarity or link strength.

**Edge Drawing** (`app.rs:2559-2570`):
- Only draws a **single midpoint dot** between connected nodes, not the full edge line.
- Multiple edges can overwrite the same midpoint.

#### What's Missing (Compared to Your Vision)

| Your Vision | Actual Implementation | Gap |
|-------------|----------------------|-----|
| Biggest memory in center | Fixed `✸` at (0,0,0) | Central mass is decorative |
| Similar themes cluster together | Fixed orbit radius per category | No content-based clustering |
| Gravitational pull on planets/moons | Modular arithmetic positioning | No force simulation |
| Galaxy grows organically | Nodes placed by insertion order | No dynamic restructuring |
| Connected nodes orbit together | Category-based grouping only | Links don't affect position |

### What IS Real in Dendrite

| Feature | Status | Notes |
|---------|--------|-------|
| Wiki-link parsing `[[target]]` | ✅ Working | `graph.rs:12-15` regex |
| Backlink maintenance | ✅ Working | Auto-wired on upsert |
| Node categories | ✅ Working | `NodeCategory` enum with tag-based classification |
| BM25 search | ✅ Working | `graph.rs:422-486` |
| FTS5 + SQLite persistence | ✅ Working | `store.rs` with triggers |
| Relevance gating | ✅ Working | `MIN_RELEVANCE_SCORE = 5.0` |
| Multi-hop traversal | ✅ Working | BFS 1/2/3 hop |
| Recency decay scoring | ✅ Working | `0.95^age_days` in `context.rs:340` |
| Spec index | ✅ Working | Weighted sum of tags + links + tier |

### What's Broken in Dendrite

**1. Graph Has No Spatial Model**
`Dendrite` is a flat `HashMap<String, Node>`. Nodes have no x/y/z coordinates. The galaxy renderer computes positions on-the-fly from insertion order, not from graph structure.

**2. `category()` is Shallow** (`graph.rs:161-188`)
```rust
pub fn category(&self) -> NodeCategory {
    for tag in &self.tags {
        let t = tag.to_lowercase();
        if t.contains("pref") || t.contains("like") ... { return Preferences; }
        if t.contains("code") || t.contains("rust") ... { return Engineering; }
    }
    // fallback by node_type
}
```
Only checks first matching tag. A node tagged `#rust #favorite` returns `Preferences` (first match), not `Engineering`.

**3. `spec_index()` is Naive** (`graph.rs:191-201`)
```rust
pub fn spec_index(&self) -> f32 {
    let tag_score = (self.tags.len() as f32 * 0.25).min(0.5);
    let link_score = ((self.links.len() + self.backlinks.len()) as f32 * 0.1).min(0.3);
    let tier_base = match self.node_type.tier() { 3 => 0.9, 2 => 0.7, ... };
    (tier_base + tag_score + link_score).min(1.0)
}
```
More tags = higher spec index, regardless of tag quality. A node with 10 random tags scores higher than one with 2 meaningful tags.

**4. No Content Similarity Clustering**
Two nodes about "Rust error handling" don't cluster together unless they explicitly `[[link]]` to each other. There's no embedding or semantic similarity.

**5. Growth Model is Static**
New nodes are added at the "edge" of their category orbit. The galaxy doesn't reorganize when new nodes arrive. No force-directed layout.

---

## Round 2 Audit — Unresolved Engine Issues

| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| R2-01 | No batch inference (single token forward pass) | Critical | Open |
| R2-02 | Prefetch logic not verified for 35B | High | Open |
| R2-03 | No GPU offloading in native engine | High | Open |
| R2-04 | NDJSON protocol not implemented | Medium | Open |
| R2-05 | `prefer_gpu` guard at `lib.rs:310` allows Tier 1 for `model_gb <= 12.0` | Medium | Open |

---

## Next Strategic Direction: Dendrite Memory Overhaul

### Goal
Transform Dendrite from a flat graph with decorative visualization into a true **self-organizing knowledge galaxy** where:
- Central nodes (high connectivity) form the core
- Similar nodes cluster by content, not just tags
- Links create gravitational attraction between nodes
- The galaxy reorganizes as it grows
- The visualization reflects actual graph structure

### Phase 1: Spatial Graph Model (Immediate)

**Add coordinates to nodes:**
```rust
pub struct Node {
    // ... existing fields ...
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub mass: f32,  // = spec_index * (1 + backlinks.len())
}
```

**Implement force-directed layout:**
- Repulsion between all nodes (Coulomb's law)
- Attraction along edges (Hooke's law: spring force)
- Central gravity toward origin (galaxy core)
- Category-specific gravity wells (sub-clusters)

```rust
fn simulate_forces(graph: &Dendrite, iterations: usize) {
    for _ in 0..iterations {
        for node in graph.all() {
            // Repulsion from all other nodes
            for other in graph.all() {
                if node.id != other.id {
                    let dx = node.x - other.x;
                    let dist = dx.norm().max(0.1);
                    let force = REPULSION / (dist * dist);
                    node.velocity += force * dx.normalize();
                }
            }
            // Attraction along edges
            for link_id in &node.links {
                if let Some(target) = graph.get(link_id) {
                    let dx = target.position - node.position;
                    let dist = dx.norm();
                    let force = SPRING_K * (dist - REST_LENGTH);
                    node.velocity += force * dx.normalize();
                }
            }
            // Central gravity
            let to_center = -node.position;
            node.velocity += CENTRAL_GRAVITY * to_center.normalize();
        }
        // Apply velocities with damping
        for node in graph.all_mut() {
            node.position += node.velocity * DAMPING;
            node.velocity *= DAMPING;
        }
    }
}
```

### Phase 2: Content-Based Clustering (Short-term)

**Replace tag-based `category()` with embedding similarity:**
- Compute lightweight TF-IDF or MinHash for each node's content
- Use similarity to assign cluster membership
- Category orbits become dynamic, not fixed

**Implement `attract()` method:**
```rust
impl Dendrite {
    pub fn attract(&mut self, node_a: &str, node_b: &str, strength: f32) {
        // Increase link weight between similar nodes
        // Pull their coordinates closer during next layout pass
    }
}
```

### Phase 3: Dynamic Galaxy Growth (Medium-term)

**On each `upsert()`:**
1. Compute new node's content fingerprint
2. Find nearest cluster center
3. Place node at cluster edge with initial velocity
4. Run 10-20 force simulation steps to settle
5. Update visualization coordinates

**On each `delete()`:**
1. Remove node
2. Re-run局部layout for affected cluster
3. Galaxy contracts naturally

### Phase 4: Visualization Overhaul (Medium-term)

**Replace current renderer with force-directed layout:**
- Nodes positioned by actual (x,y,z) coordinates
- Full edge lines (not midpoint dots)
- Node size proportional to `mass`
- Category shown by color, not orbit radius
- Zoom/pan/rotate with real 3D perspective

**Add interaction:**
- Click node to inspect content
- Drag node to reposition (manual override)
- Highlight 1-hop neighborhood on hover
- Show path between any two nodes

---

## Performance Targets

| Metric | Current | Target |
|--------|---------|--------|
| Galaxy renders actual graph structure | No (decorative) | Yes |
| Node clustering by content | No (tag-based) | Yes (TF-IDF/embedding) |
| Links affect spatial position | No | Yes (spring force) |
| Galaxy reorganizes on growth | No | Yes (force-directed) |
| Edge rendering complete | No (midpoint dots) | Yes (full lines) |
| 9B inference speed | ~2-3 tok/s | 10+ tok/s |
| 35B inference speed | ~1-2 tok/s | 5+ tok/s |

---

## Lessons Learned

1. **Visualization ≠ Function** — Pretty ASCII art doesn't mean the underlying model works
2. **Spatial models need spatial data** — Graph must store coordinates, not compute them on-the-fly
3. **Category by tag is fragile** — First-match wins, ignores tag quality
4. **Force-directed layouts are the standard** — D3.js, Graphviz, Cytoscape all use this
5. **Growth requires reorganization** — Static placement = dead galaxy
6. **Edges are the signal** — Links encode relationship strength; use them for clustering
7. **Mass = importance** — High-connectivity nodes should be larger and more central

---

*Last Updated: 2026-09-09*
*Audit Round: 2*
*Next Review: After Dendrite spatial model implementation*
