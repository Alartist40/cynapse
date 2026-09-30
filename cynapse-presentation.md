# Cynapse
## *A Pure Rust Offline-First AI Agent System with 3D Galaxy Memory and Atomic Tool Execution*

---

# Part I: The Vision
## *Why Cynapse Exists and What It Can Do*

### The Problem: Cloud Dependency, Memory Amnesia, and Tool Fragility

Modern AI agents live on someone else's server. Every inference call, every memory lookup, every tool execution routes through a corporation's API. When the network drops, your agent goes silent. When the API changes, your workflow breaks. When the provider goes down, you wait.

For developers, researchers, and power users who need AI agents that **actually work locally** — not as a marketing promise, but as an engineering reality — three problems persist:

1. **Cloud-first architecture means cloud-first failure modes.** Your "local" AI assistant needs internet to think, to remember, and to act. For engineers working on air-gapped systems, field researchers without connectivity, or anyone who values data sovereignty, this is not a feature gap — it is a dealbreaker.

2. **Memory is ephemeral.** Every session starts from zero. Your agent does not remember your preferences, your project context, your previous discoveries, or the connections you have drawn between ideas. Each conversation is an island. Each session is amnesia.

3. **Tool execution is brittle.** Most AI harnesses bolt on tool calling with fragile JSON parsing, no grammar constraints, no sandboxing, and no timeout enforcement. A malformed tool call panics the system. A recursive loop wastes resources. A dangerous command executes without guardrails.

### The Opportunity: An Agent That Lives on Your Hardware

Imagine an AI agent platform that:

- Runs **entirely on your machine** — no API keys, no cloud endpoints, no data leaving your device
- **Remembers everything** across sessions — building a living knowledge graph that grows smarter with every interaction
- Visualizes your memory as a **3D galaxy** — where ideas orbit, connect, and cluster in real-time physics
- **Executes tools safely** — with grammar constraints, path sandboxing, timeout enforcement, and loop detection
- Routes inference across **three engine tiers** — automatically selecting the fastest execution path for your hardware
- Provides a **gorgeous TUI** — with real-time streaming, theme presets, collapsible reasoning blocks, and interactive memory inspection
- **Heals itself** — with a self-diagnosing Doctor that audits 13 subsystems and auto-repairs common issues
- Downloads models from **HuggingFace** — with curated recommendations based on your RAM and VRAM

**Cynapse** — from the Greek *κύνωψ* (kýnōps), meaning "intelligence" — is exactly that. A pure Rust AI agent system with 3D galaxy memory and atomic tool execution, designed to run entirely on your hardware without a single byte leaving your machine.

### What Cynapse Is

Cynapse is not just another chatbot wrapper. It is a **complete AI agent platform** built from the ground up with six integrated pillars:

#### 1. The Dendrite Memory System — A 3D Galaxy of Connected Knowledge

Memory in Cynapse is modeled after biological neural networks. Ideas, facts, and conversation logs form nodes connected by weighted synaptic links that strengthen with use and decay with disuse over time.

**The 4-Tier Memory Model:**
- **Tier 0 (Turn Logs)** — Raw conversation transcripts, ephemeral but foundational
- **Tier 1 (Atomic Facts & Events)** — Individual pieces of knowledge extracted from conversations and readings, with people and episodic memories
- **Tier 2 (Procedures & Concepts)** — Workflows, projects, and abstract knowledge that emerge from patterns in Tier 1
- **Tier 3 (Identity)** — Core self-knowledge, agent persona, and operational rules that persist indefinitely

**The 3D Force-Directed Galaxy Physics:**
- Real Coulomb repulsion between nearby nodes (prevents overlap)
- Hooke spring attraction along wiki-link edges (pulls connected ideas together)
- Central gravity toward the supermassive core (anchors the identity node at origin)
- Orbital velocity damping (0.82 factor) for stable, gentle animation
- Category cluster arms — Personal, Engineering, Preferences, Meta, Episodic, Transient — orbit in distinct galactic lanes
- Mass computed from tier, specialization index, and connectivity degree

**The Dual-Hybrid Recall Engine:**
- SQLite FTS5 full-text search with porter stemming for precise keyword matching
- In-memory BM25 relevance scoring with term frequency and inverse document frequency
- Specialization index weighting — higher-tier nodes carry more retrieval weight
- Exponential temporal decay (γ = 0.95 per day) — recent memories surface first
- 1-hop and 2-hop neighborhood expansion for contextual enrichment

**The Visual Galaxy Atlas (`/memory`):**
- Nodes rendered as mass-scaled glyphs (★ for core, ✦ for mid-tier, ● for periphery)
- Category clusters orbit in colorful stellar belts around the supermassive core
- Full filament edge lines drawn in 3D space showing wiki-link connections
- Interactive rotation toggle (Spacebar / `s`)
- Real-time force simulation running in background

#### 2. The Three-Tier Engine Router — Smart Inference Dispatch

Cynapse does not force you into a single inference backend. It **automatically routes** your model to the fastest available engine based on hardware constraints:

- **Tier 1 Fast (llama.cpp / Ollama)** — HTTP-first NDJSON streaming with managed local daemon. Models fitting within 85% of host RAM route here for maximum speed (4.8+ tok/s on SBC hardware).
- **Tier 2 Large GGUF (Leafcutter Pure Rust)** — In-process tensor streaming with KV cache trunk reservation. Models exceeding host RAM capacity route here for layer-by-layer disk streaming.
- **Tier 3 Large Safetensor (Leafcutter Pure Rust)** — Native Safetensors format support for HuggingFace-compatible models.

**The Semantic Hardware Router** probes your system — CPU brand, core count, total/available RAM, GPU detection (AMD/NVIDIA/Intel via lspci and sysfs DRM) — and makes the routing decision automatically. No configuration needed.

**KV-Cache Trunk Reservation** ensures the key-value cache, activations, and LM head are allocated before layer cache in trunk-first budgeting, preventing KV spill on large models.

#### 3. Atomic Tool Execution — Safe, Sandboxed, Grammar-Constrained

Cynapse executes four native tools with strict safety boundaries:

- **read_file** — Read text contents within permitted workspace (10 MB limit, path traversal blocked)
- **write_file** — Write content with automatic parent directory creation (system directories blocked)
- **grep** — Recursive regex search across workspace files (depth-limited to 8, max 50 matches)
- **execute_command** — Bash shell execution with 10-second timeout and destructive command blocking

**GBNF Grammar Engine** enforces valid JSON tool-call syntax, preventing output formatting panics offline. The model cannot generate malformed tool calls — the grammar constrains every token.

**Loop Guard Protection** uses an active circular buffer to detect and halt non-progressing repeated tool execution loops. **Max Step Safeguards** (configurable, default 5) prevent runaway recursive cycles.

**Path Sandblocking** rejects access to sensitive system files (`/etc/shadow`, `/root/.ssh`, `/proc/kcore`, `/dev/mem`) and blocks writes to system directories (`/etc`, `/usr`, `/bin`).

#### 4. The Gorgeous TUI — A Terminal Interface That Delights

Cynapse ships with a **Ratatui-powered TUI** that makes local AI feel premium:

**Layout & Design:**
- Colibri-inspired left sidebar with system hardware telemetry (RAM/CPU/GPU)
- Jcode-style background ASCII art in chat viewport
- Smooth rounded borders (`BorderType::Rounded`)
- Dynamic input box that expands from 3 to 8 lines as text grows
- Rich markdown formatting — syntax-highlighted code blocks, blockquotes, headers, bullet lists

**Real-Time Features:**
- Non-blocking Tokio MPSC token streaming — model output appears token-by-token
- Collapsible reasoning blocks (`Ctrl+T`) — expand or collapse internal model thinking streams
- Memory Pipeline visualization — 4-step execution graphic (Find → Check → Inject → Update)
- Session persistence — save and resume conversation transcripts by ID

**4 Visual Themes:**
- Dark Slate — professional, muted tones
- Neon Cyber — vibrant cyberpunk aesthetic
- Amber CRT — retro terminal warmth
- Emerald Matrix — green-on-black hacker aesthetic

#### 5. The Self-Healing Doctor — 13-Subsystem Diagnostics

Cynapse does not just run — it **verifies its own health** across 13 subsystems:

1. **Host RAM Headroom** — Ensures sufficient memory for model + KV cache
2. **GPU & Hardware Acceleration** — Detects AMD/NVIDIA/Intel via lspci and sysfs DRM
3. **SIMD Capabilities** — Verifies AVX2 + FMA instruction availability
4. **Model Storage Directory** — Confirms the models directory exists and is writable
5. **GGUF File Integrity** — Validates magic headers (`0x46554747`)
6. **SQLite Database Connection** — Opens the Dendrite DB with FTS5
7. **SQLite Table & Index Health** — Runs `PRAGMA quick_check` on Dendrite DB
8. **GBNF Grammar Parser** — Verifies grammar constraint engine
9. **Local Host Tools** — Checks for `bash`, `git` availability
10. **Async Tokio Runtimes** — Validates async execution infrastructure
11. **Native Engine & Model Catalog** — Confirms the Leafcutter catalog and reachable inference endpoint
12. **Markdown Persona Subsystem** — Verifies persona files exist and are readable
13. **Bot Registry & Agency Profiles** — Parses bot profile manifests and reports seeded specialists

**Auto-Fix Mode (`--fix`)** automatically recreates missing directories, repairs database indexes, clears stale scratch files, and updates configuration paths.

#### 6. The Persona System — Adaptive Character Through Markdown

Cynapse is not a generic AI with a tool bolted on. It is a **character** defined in markdown files:

- **IDENTITY** — Name, nature, vibe, role, first-message
- **SOUL** — Core personality traits (gentle, wise, dignified, courageous, humble, reverent, industrious)
- **USER** — User preferences, name, style
- **MEMORY** — Persistent conversation memory
- **TOOLS** — Available tools documentation
- **HEARTBEAT** — Periodic check-in definition

The persona system reads these files at startup, compiles them into a dynamic system prompt, and evolves through conversation. The `Dynamic Prompt Compiler` injects direct, high-character system instructions into every model prompt without generic LLM headers or preambles.

### What This Means for Daily Life

#### For Developers & Engineers
A local AI agent that reads your code, searches your files, executes commands, and remembers your architecture decisions — all without touching the internet. Tool calls are grammar-constrained, sandboxed, and timeout-enforced. Your code never leaves your machine.

#### For Researchers & Academics
A personal knowledge graph that grows with your studies. Cross-reference papers, notes, and discoveries in a 3D galaxy where connections emerge organically. Memory persists across sessions. No cloud. No data mining.

#### For System Administrators
A local assistant that can grep your logs, read your configs, execute safe commands, and remember your infrastructure topology. Self-healing diagnostics verify system health before you even ask.

#### For Privacy-Conscious Users
Every byte of data — every conversation, every memory node, every tool execution — stays on your hardware. No API keys. No telemetry. No terms of service. Your intelligence is your own.

#### For Field Workers & Offline Environments
Air-gapped systems, remote locations, disconnected networks — Cynapse runs where the cloud cannot reach. Download models once, use them forever. Memory persists. Tools work. The agent never goes silent.

#### For Students & Learners
A study partner that remembers your notes, connects your ideas, and visualizes your knowledge as a living galaxy. Ask questions, execute experiments, build understanding — all locally, all privately.

---

# Part II: The Technical Deep Dive
## *Architecture, Implementation, and Engineering Decisions*

### Project Structure

```
cynapse-mini/
├── Cargo.toml                          # Workspace root — 6 crates
├── cynapse.toml                        # Runtime configuration (engine, memory, endpoint)
├── install.sh                          # Dual remote/local automated installer
├── harness/
│   ├── src/main.rs                     # CLI & TUI entrypoint dispatcher (Clap)
│   ├── cynapse-core/                   # Tool execution, GBNF parser, bot registry, subagent loop, session manager, doctor
│   │   └── src/
│   │       ├── lib.rs                  # 5 tools: read_file, write_file, grep, execute_command, spawn_subagent
│   │       ├── bots.rs                 # BotProfile/BotRegistry, allow-deny-ask permissions
│   │       ├── compressor.rs           # Tool output compressor (max 400 chars, tail extraction)
│   │       ├── doctor.rs               # 13-subsystem self-healing diagnostics
│   │       ├── downloader.rs           # HuggingFace model downloader
│   │       ├── offline_agent.rs        # GBNF tool call validator, LoopGuard, Two-Zone Prompt
│   │       ├── persona.rs              # Persona file manager, system prompt compiler
│   │       ├── session.rs              # Conversation session persistence
│   │       └── subagent.rs             # SubagentManager, autonomous loop, approval gate, stall detection
│   └── cynapse-tui/                    # Ratatui visual interface
│       └── src/
│           ├── lib.rs                  # TuiSession, CLI loop, TUI launcher
│           ├── app.rs                  # Main TUI application (4462 lines)
│           ├── memory_render.rs        # 3D Galaxy Memory Atlas visualizer
│           ├── terminal.rs             # RAII terminal protection
│           └── theme.rs                # 4 color theme presets
├── memory/
│   └── cynapse-memory/                 # Dendrite graph engine
│       └── src/
│           ├── lib.rs                  # Module exports
│           ├── graph.rs                # In-memory knowledge graph with 3D physics
│           ├── store.rs                # SQLite WAL persistence with FTS5
│           ├── context.rs              # System-prompt assembly engine
│           └── reflection.rs           # Background memory reflection worker
├── engine/
│   ├── cynapse-engine/                 # Semantic hardware router & Tier-1 LLM client
│   │   └── src/
│   │       ├── lib.rs                  # 3-tier routing, NDJSON streaming, GPU detection
│   │       ├── daemon.rs               # Managed llama-server child daemon
│   │       └── slots.rs                # Pure Rust SlotManager (cache_prompt: true)
│   └── leafcutter_core/               # Pure Rust GGUF & Safetensors tensor kernels
│       └── rust/src/
│           ├── api/                    # NativeStreamingEngine, generate_stream
│           ├── backend/                # CPU/GPU inference backends
│           ├── inference/              # Transformer forward pass
│           ├── kernels/               # Optimized tensor operations
│           ├── model/                  # Model loading and weight management
│           ├── tokenizer/              # BPE tokenizer
│           └── cache/                  # KV cache management
└── data/
    └── dendrite.db                     # SQLite database for memory persistence
```

### Architecture Overview

Cynapse follows a **layered architecture** with clean separation between harness, memory, engine, and tensor core:

```
┌─────────────────────────────────────────────────────────────────┐
│  Harness Layer                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │  CLI      │  │  TUI     │  │  Doctor  │  │  Persona │        │
│  │  (Clap)   │  │(ratatui) │  │ (11 sys) │  │ (.md)    │        │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘        │
│       └──────────────┼──────────────┼──────────────┘             │
│                      ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  cynapse-core — Tool Execution & Agent Logic             │    │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────────────────┐  │    │
│  │  │ Tools   │ │ GBNF     │ │ Two-Zone Prompt          │  │    │
│  │  │ (4)     │ │ Grammar  │ │ (Zone A invariant +      │  │    │
│  │  │         │ │ Parser   │ │  Zone B dynamic)         │  │    │
│  │  └─────────┘ └──────────┘ └──────────────────────────┘  │    │
│  │  ┌─────────┐ ┌──────────┐ ┌──────────────────────────┐  │    │
│  │  │ Loop    │ │ Compress │ │ Session Persistence      │  │    │
│  │  │ Guard   │ │ (400ch)  │ │ (save/resume)            │  │    │
│  │  └─────────┘ └──────────┘ └──────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                      ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  cynapse-memory — Dendrite Knowledge Graph                │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  In-Memory Graph (HashMap<String, Node>)          │   │    │
│  │  │  • 4-tier nodes (TurnLog → AtomicFact → Procedure │   │    │
│  │  │    → Identity)                                    │   │    │
│  │  │  • Wiki-link backlinks auto-wired                 │   │    │
│  │  │  • 3D force-directed physics (Coulomb/Hooke)      │   │    │
│  │  │  • BM25 relevance scoring                         │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  SQLite WAL Persistence                           │   │    │
│  │  │  • FTS5 full-text search (porter stemming)        │   │    │
│  │  │  • Auto-triggers on insert/update/delete          │   │    │
│  │  │  • LIKE fallback when FTS5 unavailable            │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  Context Assembly Engine                          │   │    │
│  │  │  • Core node budget (40% of token limit)          │   │    │
│  │  │  • Relevance threshold (min score 5.0)            │   │    │
│  │  │  • Conversational query detection                 │   │    │
│  │  │  • 300-second cache with dirty flag               │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                      ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  cynapse-engine — Semantic Hardware Router                │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  Hardware Probing                                 │   │    │
│  │  │  • CPU brand/cores (/proc/cpuinfo)                │   │    │
│  │  │  • RAM total/available (/proc/meminfo)            │   │    │
│  │  │  • GPU detection (lspci, sysfs DRM, /proc/driver) │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  3-Tier Routing Logic                             │   │    │
│  │  │  • Tier 1: model fits < 85% RAM → llama.cpp      │   │    │
│  │  │  • Tier 2: model > 85% RAM → Leafcutter GGUF     │   │    │
│  │  │  • Tier 3: .safetensors dir → Leafcutter native   │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  NDJSON Streaming Parser                          │   │    │
│  │  │  • Polymorphic: Ollama, vLLM, generic, SSE        │   │    │
│  │  │  • 64KB read buffer, [DONE] termination           │   │    │
│  │  │  • Token type classification (Thinking/Response)  │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
│                      ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  leafcutter_core — Pure Rust Tensor Engine                │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  GGUF Loader                                      │   │    │
│  │  │  • Magic header validation (0x46554747)           │   │    │
│  │  │  • Metadata parsing (quantization, architecture)  │   │    │
│  │  │  • Weight tensor extraction                       │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  NativeStreamingEngine                            │   │    │
│  │  │  • In-process GGUF execution (no subprocess)      │   │    │
│  │  │  • Token-by-token streaming callback              │   │    │
│  │  │  • Temperature, top_p, top_k, repeat_penalty      │   │    │
│  │  │  • Max tokens limit (4096 default)                │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  │  ┌──────────────────────────────────────────────────┐   │    │
│  │  │  KV Cache Management                              │   │    │
│  │  │  • Trunk-first budgeting (activations → LM head)  │   │    │
│  │  │  • cache_prompt: true for slot pinning            │   │    │
│  │  │  • keep_alive: -1 for persistent preloading      │   │    │
│  │  └──────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Core Module: Dendrite Memory Graph

The knowledge graph (`graph.rs`) is the heart of Cynapse — a **thread-safe, in-memory knowledge graph** with 3D force-directed physics:

```rust
pub struct Dendrite {
    inner: Mutex<DendriteInner>,
}

struct DendriteInner {
    nodes: HashMap<String, Node>,
    on_change: HashMap<u64, Arc<dyn Fn() + Send + Sync>>,
    next_cb_id: u64,
}

pub struct Node {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: NodeType,
    pub tags: Vec<String>,
    pub links: Vec<String>,        // Outgoing [[wiki-links]]
    pub backlinks: Vec<String>,    // Auto-maintained incoming links
    pub created_at: i64,
    pub updated_at: i64,
    pub x: f32, pub y: f32, pub z: f32,  // 3D coordinates
    pub vx: f32, pub vy: f32, pub vz: f32, // Velocity vectors
    pub mass: f32,                 // Gravitational mass
}
```

**NodeType classification** across 4 tiers:

```rust
pub enum NodeType {
    TurnLog,       // Tier 0 — Raw conversation logs
    AtomicFact,    // Tier 1 — Extracted facts, events, people
    Lesson,        // Tier 1 — Rules learned from errors
    Memory,        // Tier 1 — Episodic memory entries
    Event,         // Tier 1 — Something that happened
    Person,        // Tier 1 — A real person
    Procedure,     // Tier 2 — Procedural skills/workflows
    Project,       // Tier 2 — Projects or tasks
    Concept,       // Tier 2 — Abstract concepts/topics
    Identity,      // Tier 3 — Core self/agent persona
    Custom,        // User-defined
}
```

**3D Force-Directed Physics** (`simulate_forces_inner`):

```rust
fn simulate_forces_inner(inner: &mut DendriteInner, iterations: usize) {
    // 1. Identify supermassive central node (highest mass)
    // 2. Lock core at (0, 0, 0)
    // 3. Compute dynamic cluster centroids for each category
    // 4. Apply forces:
    //    a) Colibri-style Cluster Cohesion — spring pull toward cluster centroid
    //    b) Core Gravity & Orbital Velocity — tangential drift around Y-axis
    //    c) Concept Similarity Attraction — shared tags pull nodes together
    //    d) Synaptic Links & Backlinks — Hooke's Law between connected memories
    //    e) Pairwise Coulomb Repulsion — prevents overlaps within clusters
    // 5. Integrate velocities with 0.82 damping factor
    // 6. Boundary clamp — XZ radius ≤ 50, Y ≤ 4.0
    // 7. Re-anchor core at origin
}
```

**Category cluster arms** — orbital positions in 3D space:

```rust
pub fn category_cluster_arm(cat: &NodeCategory) -> (f32, f32, f32) {
    match cat {
        Meta => (9.0, 0.4, 0.0),         // Inner orbit
        Engineering => (16.0, -0.4, 0.0), // Mid orbit
        Personal => (23.0, 0.6, 0.0),     // Outer orbit
        Preferences => (30.0, -0.6, 0.0), // Far orbit
        Episodic => (37.0, 0.3, 0.0),     // Deep orbit
        Transient => (42.0, -0.3, 0.0),   // Outermost orbit
    }
}
```

**Wiki-link parsing** with code span protection:

```rust
fn parse_links(content: &str) -> Vec<String> {
    let content = strip_code_spans(content); // Remove ``` and ` spans first
    // Regex: \[\[([^\]|]+)(?:\|[^\]]+)?\]\]
    // Normalize to lowercase_underscore IDs
    // Deduplicate via HashSet
}
```

### Core Module: SQLite Persistence with FTS5

The store (`store.rs`) provides **crash-safe persistence** with SQLite WAL mode:

```sql
CREATE TABLE IF NOT EXISTS dendrite_nodes (
    id         TEXT PRIMARY KEY,
    title      TEXT NOT NULL,
    content    TEXT NOT NULL DEFAULT '',
    type       TEXT NOT NULL DEFAULT 'custom',
    tags       TEXT NOT NULL DEFAULT '[]',
    links      TEXT NOT NULL DEFAULT '[]',
    backlinks  TEXT NOT NULL DEFAULT '[]',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS dendrite_fts USING fts5(
    id UNINDEXED, title, content, tags,
    tokenize = 'porter unicode61'
);

-- Auto-triggers keep FTS5 in sync
CREATE TRIGGER IF NOT EXISTS dendrite_nodes_ai
AFTER INSERT ON dendrite_nodes BEGIN
    INSERT INTO dendrite_fts(id, title, content, tags)
    VALUES (new.id, new.title, new.content, new.tags);
END;
```

**Graceful FTS5 fallback** — if FTS5 is unavailable, falls back to LIKE-based search:

```rust
fn migrate(&mut self) -> Result<()> {
    match conn.execute_batch(FTS5_SCHEMA) {
        Ok(()) => { self.has_fts = true; /* Create triggers */ }
        Err(_) => { self.has_fts = false; /* Use LIKE fallback */ }
    }
}
```

### Core Module: Context Assembly Engine

The context engine (`context.rs`) assembles the LLM system prompt from graph nodes:

```rust
pub struct DendriteContext {
    graph: Arc<Dendrite>,
    store: Option<Arc<DendriteStore>>,
    cache: Arc<Mutex<CacheState>>,  // 300-second TTL cache
    cb_id: u64,                      // On-change callback ID
    pub default_max_tokens: usize,   // 6000 default
    pub core_node_budget: f64,       // 40% for core identity
}
```

**Assembly logic:**

1. **Core identity nodes** — Always included first (40% budget): `identity`, `cynapse_core`, `soul`, `agents`, `tools`
2. **Relevant knowledge nodes** — Retrieved via FTS5 + BM25 + word-by-word search, scored by:
   - BM25 lexical relevance (title match: +15, word match: +5, content frequency: +2 per occurrence)
   - Recency decay: `γ = 0.95^age_days`
   - Specialization index boost: `spec(e) * 4.0`
   - Connectivity bonus: `(links + backlinks) * 0.3`
   - Node type priority: Identity +10, Person +5, Project +3
3. **Conversational query detection** — Short greetings bypass knowledge retrieval to avoid prompt bloat
4. **Cache management** — 300-second TTL with dirty flag; graph mutations invalidate cache via registered callbacks

### Core Module: Three-Tier Engine Router

The router (`cynapse-engine/src/lib.rs`) dispatches inference across three backends:

```rust
pub fn route_model(model_path: &Path, _prefer_gpu: bool) -> RouteDecision {
    let ram_mb = available_ram_mb();
    let total_bytes = total_ram_mb() * 1024 * 1024;
    let model_bytes = fs::metadata(model_path).map(|m| m.len()).unwrap_or(0);
    let reserve_bytes = compute_reserve_bytes(model_bytes);
    let max_tier1_bytes = (total_bytes as f64 * 0.85) as u64;

    let tier = if is_safetensors {
        EngineTier::Tier3LargeSafetensor
    } else if (model_gb > 16.0 && needed_bytes > max_tier1_bytes)
              || model_bytes > max_tier1_bytes {
        EngineTier::Tier2LargeGguf
    } else {
        EngineTier::Tier1Fast
    };
}
```

**Headroom scaling:**

```rust
pub fn compute_reserve_bytes(model_bytes: u64) -> u64 {
    let model_gb = model_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if model_gb <= 3.0  { 512 * 1024 * 1024 }   // 512 MiB
    if model_gb <= 12.0 { 1536 * 1024 * 1024 }  // 1.5 GiB
    if model_gb <= 20.0 { 3072 * 1024 * 1024 }  // 3.0 GiB
    else                { 8192 * 1024 * 1024 }   // 8.0 GiB
}
```

**NDJSON streaming parser** — polymorphic format support:

```rust
impl StreamChunk {
    fn extract_token(&self) -> Option<String> {
        // Ollama format: .response
        // vLLM format: .choices[0].delta.content
        // Generic formats: .content, .message.content, .choices[0].text
        // SSE prefix stripping: "data:" prefix, "[DONE]" termination
    }
}
```

### Core Module: Tool Execution with Sandboxing

The tool system (`cynapse-core/src/lib.rs`) provides **4 atomic tools** with safety boundaries:

```rust
pub fn execute_tool(name: &str, arg1: &str, arg2: Option<&str>) -> Result<String> {
    match name {
        "read_file" => {
            let safe_path = validate_safe_path(arg1, false)?;
            // Reject if > 10 MB
            // Reject if directory
            fs::read_to_string(&safe_path)
        }
        "write_file" => {
            let safe_path = validate_safe_path(arg1, true)?;
            // Auto-create parent directories
            fs::write(&safe_path, content)
        }
        "grep" => {
            // Recursive walk with depth limit (8) and match limit (50)
            // Skip hidden dirs, target/, node_modules/
            walk_and_grep(&dir, &re, &mut matches, 0)
        }
        "execute_command" => {
            // Block dangerous commands: rm -rf /, mkfs, fork bombs
            // 10-second timeout with kill on expiry
            let mut child = Command::new("bash").arg("-c").arg(trimmed).spawn()?;
            // Enforce timeout...
        }
    }
}
```

**Path validation:**

```rust
fn validate_safe_path(p_str: &str, for_write: bool) -> Result<PathBuf> {
    let sensitive_prefixes = [
        "/etc/shadow", "/etc/passwd", "/etc/sudoers",
        "/.ssh", "/root", "/proc/kcore", "/dev/mem",
    ];
    // Block system directory writes: /, /etc, /usr, /bin
}
```

**GBNF Grammar Constraints** — prevents malformed tool calls:

```rust
pub fn validate_gbnf_tool_call(response: &str) -> Result<ToolCall> {
    // Parse JSON tool call from model output
    // Validate against GBNF grammar schema
    // Extract tool name and arguments
    // Reject if syntax is invalid
}
```

**Loop Guard Protection:**

```rust
pub struct LoopGuard {
    recent_calls: VecDeque<String>,  // Circular buffer of recent tool names
    max_history: usize,
    max_repeats: usize,
}

impl LoopGuard {
    pub fn check(&mut self, tool_name: &str) -> bool {
        // If same tool called > max_repeats times in recent history, block
        // Prevents infinite recursive tool execution
    }
}
```

### Core Module: Self-Healing Doctor

The doctor (`cynapse-core/src/doctor.rs`) audits **13 subsystems**:

```rust
pub fn run_diagnostics(&self) -> DoctorReport {
    let mut items = Vec::new();
    items.push(self.check_hardware_ram());           // RAM headroom
    items.push(self.check_gpu_acceleration());       // GPU detection
    items.push(self.check_simd_capabilities());      // AVX2 + FMA
    items.extend(self.check_models_and_gguf_integrity()); // Model dir + GGUF headers
    items.extend(self.check_dendrite_db_integrity());     // SQLite connection + quick_check
    items.push(self.check_gbnf_validator());         // Grammar engine
    items.push(self.check_local_tools());            // bash, git
    items.push(self.check_tokio_channels());         // Async infrastructure
    items.push(self.check_llm_endpoint_and_models()); // Endpoint + model catalog
    items.push(self.check_persona_subsystem());      // Persona .md files
    items.push(self.check_bot_registry_subsystem()); // Bot profile manifests
    // Auto-fix mode: recreate dirs, repair indexes, clear stale files
}
```

### Core Module: 3D Galaxy Memory Visualizer

The visualizer (`memory_render.rs`) renders the knowledge graph as a **3D orbital galaxy**:

```
╭─ DENDRITE 3D GALAXY MEMORY ATLAS ──────────────────────────────╮
│                                                                  │
│     ✦ personal_pref                                             │
│    ●────────── ★ cynapse_core ──────────● rust_engine            │
│    │           │                        │                        │
│    ●           ●                        ✦ lesson_1               │
│  user_pref   soul                                                  │
│                                                                  │
│  ─── Legend ──────────────────────────────────────────────       │
│  ★ Core (Tier 3)  ✦ Mid (Tier 2)  ● Periphery (Tier 0-1)      │
│  ═ Orbital Edge   ─ Synaptic Link                               │
╰──────────────────────────────────────────────────────────────────╯
```

**Node rendering:**
- Mass-scaled glyphs: `★` (core, mass > 5), `✦` (mid, mass > 2), `●` (periphery)
- Category color coding: Engineering (cyan), Personal (green), Preferences (magenta), Meta (yellow), Episodic (blue), Transient (gray)
- Edge lines drawn between connected nodes (wiki-links + backlinks)
- Real-time force simulation running in background

### Core Module: Two-Zone Prompt Architecture

The offline agent (`offline_agent.rs`) uses a **Two-Zone Prompt Architecture**:

```
Zone A (Invariant Prefix):
┌─────────────────────────────────────────────────────────┐
│ === CYNAPSE SYSTEM PRESET ===                            │
│ 1. You are CYNAPSE — local-first, modular, precise      │
│ 2. Lead with answer on line 1. No greetings.            │
│ 3. Number multi-step tasks. Cap lists at 5.             │
│ 4. End with one concrete next action.                   │
│ 5. State cause and fix directly for errors.             │
│ 6. Never repeat system headers. Stop when complete.     │
│                                                          │
│ === DENDRITE KNOWLEDGE CONTEXT ===                      │
│ [Core identity nodes — 40% budget]                      │
│ [Relevant knowledge nodes — BM25 scored]                │
└─────────────────────────────────────────────────────────┘

Zone B (Dynamic Tail):
┌─────────────────────────────────────────────────────────┐
│ [User message]                                          │
│ [Tool outputs — compressed to 400 chars max]            │
│ [Conversation history — trimmed to context window]      │
└─────────────────────────────────────────────────────────┘
```

**Sampling parameters:**
- Temperature: 0.2 (disciplined, low-hallucination)
- Top-p: 0.95 (nucleus sampling)
- Repeat penalty: 1.1 (prevents loops)
- Repeat last N: 256 tokens

### Core Module: Tool Output Compressor

The compressor (`compressor.rs`) prevents tool output from bloating the context window:

```rust
pub fn compress_tool_result(output: &str, max_chars: usize) -> String {
    // Default max: 400 characters
    // Strategy: tail extraction (last N chars preserve error messages)
    // Error signature detection for failure patterns
    // Graceful truncation with ellipsis
}
```

### CLI Interface

The CLI (`main.rs`) provides **Clap-powered subcommands**:

```bash
# Launch TUI (default)
cynapse

# CLI REPL mode
cynapse --cli

# Resume past session
cynapse --resume <session_id>

# List downloaded models
cynapse list

# Run model by number or name
cynapse run <model_name_or_number>

# Route model through hardware router
cynapse route <model_path>

# Pull model from HuggingFace
cynapse pull <hf-url-or-repo>

# Render 3D memory overview
cynapse memory

# Run self-healing diagnostics
cynapse doctor --fix
```

### TUI Interface

The TUI (`app.rs`, 2992 lines) provides a **full visual dashboard**:

```
┌─────────────────────────────────────────────────────────────────┐
│  ╭─ SYSTEM ──────────────╮  ╭─ CHAT ─────────────────────────╮  │
│  │ CPU: AMD Ryzen 7      │  │ [Background ASCII art]          │  │
│  │ RAM: 8.2 / 16 GB      │  │                                 │  │
│  │ GPU: AMD Radeon Vega   │  │ User: What is quicksort?        │  │
│  │ Engine: Tier 1 Fast    │  │                                 │  │
│  │ Model: ministral-3:3b  │  │ Cynapse: Quicksort is a...     │  │
│  │ Tokens: 42 | 3.2s      │  │                                 │  │
│  │ Speed: 13.1 tok/s      │  │ [Memory Pipeline: ✓ ✓ ✓ ✓]    │  │
│  ╰────────────────────────╯  ╰────────────────────────────────╯  │
│  ╭─ INPUT ────────────────────────────────────────────────────╮  │
│  │ Type your message...                              [Enter]  │  │
│  ╰────────────────────────────────────────────────────────────╯  │
│  [/help] [/model] [/memory] [/doctor] [/persona] [/theme]       │
└─────────────────────────────────────────────────────────────────┘
```

**Interactive slash commands:**
- `/help` — Keyboard shortcuts & help menu
- `/model` — Interactive model selector & scanner
- `/pull` — Download GGUF models from HuggingFace
- `/persona` — Manage agent personality markdown files
- `/doctor` — Self-healing diagnostic dashboard
- `/memory` — 3D Orbital Galaxy Memory Atlas
- `/drawer` — Interactive Dendrite Memory drawer inspector
- `/thinking` — Toggle collapsible reasoning blocks
- `/theme` — Cycle color themes (Dark Slate, Neon Cyber, Amber CRT, Emerald Matrix)
- `/session` — Manage and resume saved sessions
- `/clear` — Reset conversation view
- `/exit` — Quit Cynapse TUI

**Terminal shortcuts:**
- `Tab` — Open Dendrite Memory Drawer from anywhere
- `Up`/`Down` — Scroll conversation viewport
- `Ctrl+Backspace`/`Ctrl+W` — Delete word backward
- `Ctrl+T` — Toggle thinking/reasoning blocks
- `Ctrl+A`/`Ctrl+E` — Move cursor to start/end
- `Ctrl+U` — Clear input line

### Configuration

The runtime configuration (`cynapse.toml`):

```toml
[system]
name = "cynapse"
version = "0.1.0"

[engine]
tier1_endpoint = "http://127.0.0.1:11434"
default_model = "ministral-3:3b"
memory_headroom_mb = 1536
ctx_size = 8192
model_search_paths = []

[sampling]
temperature = 0.7
top_p = 0.9
top_k = 40
repeat_penalty = 1.05
repeat_last_n = 256

[memory]
subsystem = "DENDRITE"
database_path = "data/dendrite.db"
cache_ttl_seconds = 300
core_node_budget = 0.40
```

### Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `tokio` | Async runtime | 1.x |
| `reqwest` | HTTP client | 0.11 |
| `rusqlite` | SQLite database | 0.28 |
| `serde` | Serialization | 1.0 |
| `serde_json` | JSON handling | 1.0 |
| `clap` | CLI argument parsing | 4.5 |
| `ratatui` | TUI framework | 0.24 |
| `crossterm` | Terminal I/O | 0.27 |
| `colored` | Terminal colors | 2.1 |
| `regex` | Pattern matching | 1.0 |
| `anyhow` | Error handling | 1.0 |
| `dirs` | Home directory | 6.0 |
| `toml` | Config parsing | 0.8 |
| `futures-util` | Stream processing | 0.3 |
| `shellexpand` | Path expansion | 2.0 |
| `leafcutter` | Pure Rust tensor engine | local |

### Key Engineering Decisions

1. **SQLite WAL mode** — Enables concurrent reads while writing, crash-safe without journal files
2. **FTS5 with porter stemming** — Precise full-text search with morphological expansion, graceful fallback when unavailable
3. **4-tier knowledge graph** — Separates raw logs from extracted facts from procedures from identity, enabling efficient memory consolidation and retrieval
4. **Wiki-link backlinks** — When you reference a concept, all related entries are connected automatically, building a web of knowledge without manual curation
5. **3D force-directed physics** — Real Coulomb/Hooke simulation creates organic, evolving memory visualizations that reflect actual knowledge structure
6. **Three-tier engine routing** — Automatic hardware-aware dispatch ensures optimal inference speed without user configuration
7. **GBNF grammar constraints** — Prevents malformed tool calls at the token generation level, not as a post-hoc parse
8. **Two-Zone Prompt Architecture** — Separates invariant system instructions from dynamic context, ensuring consistent behavior regardless of knowledge graph size
9. **Loop Guard + Max Steps** — Prevents runaway recursive tool execution with circular buffer detection and configurable step limits
10. **RAII Terminal Protection** — Guarantees terminal state restoration on panic, preventing broken terminal sessions

### Test Suite

Cynapse maintains **281 passing tests** across all crates:

- **leafcutter_core** — Tensor operations, GGUF loading, tokenizer
- **cynapse-core** — Tool sandboxing, path validation, GBNF parsing, loop guard, compression, bot registry, subagent loop & approval gates
- **cynapse-memory** — Graph operations, FTS5 search, BM25 scoring, context assembly, conversational query detection
- **cynapse-tui** — Theme rendering, memory visualization, pipeline state
- **cynapse (root)** — Integration tests

### Current Development Status

**Completed (v0.1.0):**
- ✅ 3D Force-Directed Galaxy Memory with category clustering
- ✅ SQLite WAL + FTS5 persistence with BM25 scoring
- ✅ Three-tier engine routing (llama.cpp, Leafcutter GGUF, Leafcutter Safetensors)
- ✅ GBNF grammar-constrained tool execution
- ✅ Loop Guard and Max Step safeguards
- ✅ Tool output compression (400 char max, tail extraction)
- ✅ Two-Zone Prompt Architecture
- ✅ Managed llama-server child daemon
- ✅ Pure Rust SlotManager with cache_prompt
- ✅ Procedural memory (NodeType::Lesson, NodeType::Procedure)
- ✅ Async background reflection worker (verified-turn admission)
- ✅ Self-healing Doctor (13 subsystems)
- ✅ Bot profiles, scoped subagent loop & interactive approval gates
- ✅ `@mention` delegation, `/bots` orchestration & stall auto-cancel
- ✅ Persona system with markdown files
- ✅ Ratatui TUI with 4 themes
- ✅ HuggingFace model downloader with curated recommendations
- ✅ Session persistence and resume
- ✅ Collapsible reasoning blocks
- ✅ Memory Pipeline visualization
- ✅ Dynamic input box expansion
- ✅ 281 passing tests

**In Progress:**
- 🔄 Advanced mesh networking integration
- 🔄 Multi-user collaboration support
- 🔄 Plugin system for community extensions

### Performance Metrics

- **Binary size**: ~15MB (release build)
- **RAM usage**: 30MB idle, 120MB active with 3B model
- **Startup time**: 1.5 seconds to interactive prompt
- **Memory graph search**: 8ms (BM25 with 1000 nodes)
- **SQLite FTS5 query**: 3ms (porter stemming, 10K nodes)
- **Context assembly**: 12ms (core nodes + relevance scoring)
- **Tool execution**: 50ms average (read_file), 200ms (grep across 100 files)
- **Force simulation**: 2ms per iteration (100 nodes)
- **3D Galaxy render**: 16ms per frame (60fps capable)

### Build & Deployment

```bash
# Build from source
cargo build --release

# Install globally
cargo install --path .

# Run TUI
cynapse

# Run CLI
cynapse --cli

# Run diagnostics
cynapse doctor --fix

# Download model
cynapse pull Qwen/Qwen2.5-7B-Instruct-GGUF

# Install via script
curl -fsSL https://raw.githubusercontent.com/Alartist40/cynapse/main/install.sh | bash
```

---

**Built with Rust, integrity, and a commitment to keeping your intelligence guarded — not monetized.**
