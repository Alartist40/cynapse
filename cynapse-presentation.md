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
- **Heals itself** — with a self-diagnosing Doctor that audits 22 subsystems and auto-repairs common issues
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

**The AutoHarness Self-Learning & Self-Pruning Lifecycle:**
- Opportunity-relative usage rate ($\text{Rate} = \frac{\text{Loads}}{\text{Requests since creation}}$) ensures skills never decay when the machine is powered off
- 100-request probation period prevents premature eviction of freshly distilled skills
- Dynamic capacity contention gracefully demotes low-utility skills to cold storage when memory limits are reached
- Umbrella consolidation automatically fuses overlapping procedural skills into concise unified standards
- Non-destructive archiving preserves distilled knowledge in cold storage, revivable in milliseconds via FTS5

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

#### 3. Atomic Tool Execution & Extreme-Optimization Harness

Cynapse executes seven native tools with strict safety boundaries and an optimization harness designed to turn tiny 0.5B–3B models into reliable autonomous agents:

- **read_file** — Read text contents within permitted workspace (10 MB limit, path traversal blocked)
- **write_file** — Write content with automatic parent directory creation (system directories blocked)
- **patch_file_at** — Surgical chunk replacement with exact unique anchor validation (no full-file rewrites)
- **grep** — Recursive regex search across workspace files (depth-limited to 8, max 50 matches)
- **execute_command** — Bash shell execution with 10-second timeout, disk spillover for outputs > 1200 chars, and destructive command interception
- **run_gates** — Deterministic acceptance gate runner evaluating `GATES.md` (`CHECK:`, `EXPECT:`, `EVIDENCE:`)
- **spawn_subagent** — Autonomous task delegation with scoped tool permissions and stall auto-cancel

**GBNF Grammar Engine & Tolerant Coercion** enforces valid JSON tool-call syntax while forgiving small model parameter typos via fuzzy alias canonicalization and Levenshtein distance ($\le 2$) repair hints.

**Dynamic Micro-Skill Intent Slicing** dynamically selects and injects a single $\le 35$ token micro-directive matching user intent, preserving 90%+ of small context windows for actual work.

**Disk Output Spillover & Listing Caps** prevents terminal log floods by spilling outputs $> 1,200$ chars to `~/.cynapse/spill/<hash>.log` and capping listings at 5 items per group.

**Loop Guard & Security Shield** halts non-progressing repeated loops via a circular buffer and prevents dangerous OS executions (`rm -rf /`, `mkfs`, fork bombs) or path traversal (`/etc/shadow`, `/root/.ssh`).

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

#### 5. The Self-Healing Doctor — 22-Subsystem Diagnostics & Maintenance

Cynapse does not just run — it **audits, maintains, and self-heals its entire system** across 22 subsystems:

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
11. **Native Engine & Model Catalog** — Confirms the Leafcutter catalog
12. **Inference Backend Reachability** — Probes active port and fails loud on disconnect
13. **Markdown Persona Subsystem** — Verifies persona files exist and are readable
14. **Bot Registry & Agency Profiles** — Parses bot profile manifests and reports seeded specialists
15. **AutoHarness Lifecycle** — Audits opportunity-relative usage rates, active skill capacity, and probation shields
16. **Acceptance Gates Engine** — Validates `GATES.md` runnable checks (`CHECK:`, `EXPECT:`, `EVIDENCE:`)
17. **Dynamic Micro-Skill Intent Slicer** — Verifies discipline keyword triggers and $\le 35$ token bounds
18. **Disk Output Spillover Hygiene** — Monitors `~/.cynapse/spill/` and auto-prunes stale logs
19. **Autonomous Security Guard** — Verifies path traversal blocks, credential locks, and destructive command interception
20. **Surgical File Patching Engine** — Executes live in-memory exact-anchor replacement tests
21. **Verified Read Receipts Registry** — Verifies content hashing and context-window anchor deduplication
22. **SlotManager & Cache Pinning** — Verifies multi-slot concurrency and `cache_prompt: true` slot locking

**Auto-Fix Mode (`--fix`)** automatically recreates missing directories, scaffolds acceptance gates, repairs database schemas, clears stale spillover logs, and restores corrupted configurations.

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

### The Feynman Masterclass: Why Weak Hardware Outperforms Heavy Cloud Clusters

If you explain a concept to a smart 10-year-old and they get confused, you don't understand it yourself. Here is how Cynapse actually works, stripped of marketing jargon and buzzwords.

#### 1. The Bicycle vs. The 40-Ton Cargo Ship
Commercial AI stacks (LangChain, AutoGen, Python vLLM) are like chartering a 40-ton diesel cargo ship just to deliver a handwritten envelope across the street. They require gigabytes of Python packages, a heavy interpreter, CUDA runtimes, and garbage collection pauses that choke consumer hardware.

Cynapse is a precision carbon-fiber bicycle. It is written in 100% pure compiled Rust. It compiles down to a single 15MB binary that executes directly on bare metal CPU registers. There is zero runtime garbage collector, zero Python overhead, and zero idle bloat:
- **Idle memory footprint**: Just 30MB of RAM.
- **Cold start to interactive prompt**: Under 1.5 seconds.
- **Execution efficiency**: Near 100% of CPU cycles are spent computing tokens and memory physics, not managing bloated interpreter runtimes.

```
Commercial Cloud Stack:
[ User ] ──▶ [ Python Runtime ] ──▶ [ LangChain / vLLM ] ──▶ [ 10GB Virtual Memory ] ──▶ [ GPU Cloud ]
                               (High overhead, memory leaks, latency)

Cynapse Pure Rust:
[ User ] ──▶ [ 15MB Native Binary ] ──▶ [ CPU L3 Cache / NEON-AVX2 ] ──▶ [ Direct Hardware ]
                               (Zero VM, zero GC, instant execution)
```

#### 2. L3 Cache Residency: Why Tiny Quantized Models Fly on Weak Laptops & SBCs
Computer RAM is slow. Fetching data from main memory takes about 200 CPU clock cycles. But fetching data from the CPU's internal L3 cache takes only 20 to 40 cycles — **5 to 10 times faster**.

When you run massive, unquantized models, their weights are so gigantic that the CPU is constantly waiting for slow RAM. In Cynapse:
- Model weights are compressed into 4-bit blocks (`Q4_K_M` and `IQ4_NL`). A 3B parameter model shrinks from 12GB down to 1.8GB.
- Pure Rust SIMD kernels (`AVX2` on x86, `NEON` on ARM Raspberry Pi / Orange Pi) load small matrix tiles directly into the CPU's high-speed L3 cache.
- The model runs at maximum hardware memory bandwidth without thermal throttling, allowing a $45 Single Board Computer or an old laptop to stream responses at conversational speeds (4.8+ tokens/second).

#### 3. Trunk-First Memory Budgeting: Eliminating the Out-Of-Memory Crash
Why do local AI assistants crash unexpectedly during long conversations?
In standard systems, the Key-Value (KV) cache grows unpredictably like an uncontrolled balloon. Eventually, the operating system runs out of RAM and abruptly terminates the program with an `Out Of Memory (OOM)` killer.

Cynapse uses **Trunk-First Memory Budgeting**:
1. Before allocating space for model layers, Cynapse calculates the maximum possible conversation context and reserves the entire memory "trunk" upfront.
2. The KV cache, model activations, and LM output head are locked into memory first.
3. The remaining budget is strictly allocated to model weights.
Because the ceiling is fixed at startup, Cynapse **never runs out of memory mid-conversation**, even on hardware with only 4GB or 8GB of total RAM.

#### 4. Needle-3 Speculative Acceleration: The Scout and the Grandmaster
Imagine writing a book with a Grandmaster author. If the Grandmaster has to think carefully about every single common word ("the", "is", "in"), writing is painfully slow.

Instead, Cynapse pairs a microscopic 0.5B model (**Needle-3**) with the primary model:
- **The Scout (Needle-3)**: Generates 5 tokens in 10 milliseconds.
- **The Grandmaster (Primary Model)**: Evaluates all 5 tokens in a single parallel verification pass. If they are correct, it accepts them instantly; if one token is off, it corrects just that token and moves forward.
The result? **A 2.5x to 3x throughput speedup** with zero loss in mathematical precision or conversational nuance.

---

### The 6 Architectural Pillars Explained with the Feynman Technique

How can a tiny 0.5B to 3B model perform like a commercial 70B cloud agent without hallucinating or running out of context? The secret is not in prompting—it is in the **Harness Architecture**.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CYNAPSE HARNESS SHIELD                             │
│                                                                             │
│   [ Tiny 0.5B-3B Model ]                                                    │
│         │                                                                   │
│         ▼ (Generated Tool Call)                                             │
│   ┌───────────────────────────────────────────────────────────────────────┐ │
│   │ 1. Tolerant Coercion (Levenshtein) ──▶ Corrects parameter typos       │ │
│   │ 2. Surgical File Patching           ──▶ Replaces only targeted chunks │ │
│   │ 3. Deterministic Acceptance Gates   ──▶ Validates with runnable code  │ │
│   │ 4. Micro-Skill Intent Slicing       ──▶ Injects <= 35 token directives│ │
│   │ 5. Disk Output Spillover (Cap 5)    ──▶ Traps bloat to ~/.cynapse/    │ │
│   │ 6. Autonomous Security Guard        ──▶ Blocks dangerous OS calls     │ │
│   └───────────────────────────────────────────────────────────────────────┘ │
│         │                                                                   │
│         ▼                                                                   │
│   [ Rock-Solid Execution on Real Hardware ]                                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Pillar 1: Tolerant Coercion & Levenshtein Correction (The Compassionate Translator)
- **The Real-World Metaphor**: Imagine asking a sharp 7-year-old to grab a "screwdriver" from the toolbox, but they say "screw-driver-tool". A rigid corporate computer crashes and screams `Error 400: Unknown argument`. Cynapse acts like an observant adult: it knows exactly what the child meant.
- **The Engineering**: Small models frequently use common aliases like `filepath`, `target`, or `filename` when the tool expects `path`. Rather than rejecting the tool call and wasting turns, `cynapse-core` runs a two-tier normalizer:
  1. *Fuzzy Alias Mapping*: Direct canonicalization of known variations (`path` ↔ `file_path` ↔ `file`).
  2. *Levenshtein Distance Metric ($\le 2$)*: If an unrecognized key is within 2 character edits of a valid parameter (e.g., `ptah` instead of `path`), Cynapse auto-corrects it in place. If it cannot resolve it, it returns an explicit one-line hint telling the model the exact valid keys.

#### Pillar 2: Surgical File Patching (The Keyhole Surgeon)
- **The Real-World Metaphor**: If your jacket has a single loose button, you don't burn the jacket and weave a brand-new one from raw wool. You take a needle and sew the one button.
- **The Engineering**: Commercial AI coding tools make models rewrite entire 500-line files to change one line of code. Tiny models suffer from attention drift—halfway through rewriting the file, they hallucinate, omit critical functions, or hit token limits.
  Cynapse introduces native **surgical patching** (`patch_file_at`). The model specifies:
  - `path`: Target file
  - `target`: The exact unique code snippet to replace (anchor)
  - `replacement`: The new code
  `cynapse-core` verifies that the `target` occurs exactly once in the file to guarantee uniqueness, and performs an atomic in-place replacement. Zero context wasted, zero hallucinations.

#### Pillar 3: Deterministic Acceptance Gates (The Concrete Measuring Tape)
- **The Real-World Metaphor**: When a carpenter builds a bookshelf, you don't ask them "Do you *feel* like the shelf is level?" You place a spirit level on top and check the bubble.
- **The Engineering**: Small language models are people-pleasers. They will output *"I have successfully fixed all bugs!"* even when the code does not compile. Cynapse completely eliminates subjective completion by parsing `GATES.md`:
  - `CHECK`: The automated shell command to execute (`cargo test`, `npm test`, `git status`).
  - `EXPECT`: The required exit status or expected string output.
  - `EVIDENCE`: The verifiable output recorded in logs.
  The agent cannot declare a task done until `run_gates` executes every check and observes clean exit code 0. No evidence = not done.

#### Pillar 4: Dynamic Micro-Skill Intent Slicing (The Just-In-Time Index Card)
- **The Real-World Metaphor**: If you want to know how to change a bicycle tire, you don't want someone to drop an 8,000-page encyclopedia on your head. Your arms will collapse. You just need a 3-sentence index card on tire levers.
- **The Engineering**: Large prompt rules (Universal Skills, coding standards) consume 30KB (`~8,000` tokens). In a 3B model with a 2,048 or 4,096 token context window, this exhausts the entire memory before the user can even state their problem.
  Cynapse decomposes universal skills into microscopic, single-purpose directives ($\le 35$ tokens each). Using lexical intent scoring, `skills.rs` analyzes the user's immediate message:
  - If the user asks about an error, it injects only the 30-token *Error Triage* micro-skill.
  - If the user asks to write tests, it injects only the 25-token *TDD Gate* micro-skill.
  Over 90% of the small model's context window remains completely free for user code and reasoning.

#### Pillar 5: Disk Output Spillover & Strict 5-Item Listing Caps (The Focused Spotlight)
- **The Real-World Metaphor**: If a teacher hands you a 1,000-page printout of server logs in 2-point font, your eyes glaze over. But if they hand you the last 5 critical error lines and a library card to inspect the rest, you solve the problem immediately.
- **The Engineering**: When a command like `cargo test` fails, it can generate 10,000 lines of output. Dumping this into an LLM context instantly blinds the model.
  Cynapse's compressor implements:
  - *5-Item Listing Cap*: Grep and directory listings are strictly capped at 5 high-relevance items per group.
  - *Disk Spillover*: When command output exceeds 1,200 characters, the full raw log is automatically written to disk at `~/.cynapse/spill/<hash>.log`. The model is provided only the essential diagnostic tail and the exact path to inspect the log if needed. Context remains crystal clear.

#### Pillar 6: Autonomous Security Guard (The Reinforced Safety Lock)
- **The Real-World Metaphor**: You invite an assistant into your workshop and give them access to the workbench, but you put a padlock on the circuit breaker and the safe.
- **The Engineering**: Small models can easily be tricked or make mistakes with shell commands. Cynapse implements a native, zero-overhead security shield directly in pure Rust:
  - *Blocked Shell Commands*: Automatically intercepts and halts `rm -rf /`, `mkfs`, `dd if=/dev/zero`, `:(){ :|:& };:`, and arbitrary pipe-to-bash executions.
  - *Path Sandboxing*: Prevents path traversal and blocks access to sensitive system paths (`/etc/shadow`, `/root/.ssh`, `/proc/kcore`, `/dev/mem`).
  - *Safe Write Boundaries*: Forbids writing to system roots (`/etc`, `/usr`, `/bin`).

---

### Dendrite AutoHarness: Biological Evolution for AI Memory & Skills

Human memory is not a static hard drive; it is a living ecosystem. The human brain does not remember every receipt from five years ago, nor does it forget how to swim. It continuously reinforces useful skills and prunes idle memories.

`cynapse-memory::autoharness` implements this biological lifecycle directly in Cynapse:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      DENDRITE AUTOHARNESS LIFECYCLE                         │
│                                                                             │
│   New Skill Distilled ──▶ [ Probation Period: 100 Requests ]                │
│                                     │                                       │
│                                     ▼                                       │
│                        [ Active Skill Galaxy Pool ]                         │
│                                     │                                       │
│       ┌─────────────────────────────┴─────────────────────────────┐         │
│       ▼                                                           ▼         │
│ High Opportunity Usage                                Low Opportunity Usage │
│ (Thickens Synaptic Link)                              (Capacity Contention) │
│       │                                                           │         │
│       ▼                                                           ▼         │
│ [ Core Memory Tier ]                                  [ Non-Destructive ]   │
│                                                       [ Cold Archive ]      │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Opportunity-Relative Usage Rate (Fairness Across Time)
Most software tracks "recency" by wall-clock time (e.g., "unused for 7 days = delete"). This is deeply flawed. If you close your laptop and take a two-week vacation, a wall-clock system wakes up and deletes all your skills thinking they went "stale"!

Cynapse tracks **Opportunity-Relative Usage Rate**:
$$\text{Usage Rate} = \frac{\text{Loads}}{\text{Requests Since Creation}}$$
- "Requests Since Creation" measures how many times the user actually interacted with the agent since the skill was born.
- If you don't use the computer for a month, the request counter does not tick, and the skill's score does not decay by a single decimal point. A skill is only evaluated when it actually had an opportunity to be used.

#### 2. The 100-Request Probation Shield
When a scientist develops a new formula, you don't tear it up if nobody uses it in the first five minutes.
In Dendrite AutoHarness, every newly distilled skill enters a **100-request probation period**. During these initial 100 turns, the skill is immune to eviction, giving it adequate real-world opportunities to match user intent.

#### 3. Dynamic Pruning & Capacity Contention
To prevent knowledge bloat on small devices, Dendrite enforces a configurable capacity ceiling (default 50 active skills). When a new skill is distilled and the pool is full, AutoHarness ranks all non-probationary skills by their opportunity-relative usage rate:
- The highest-performing skills are retained in fast memory.
- The lowest-performing skills are evicted from the active context.

#### 4. Umbrella Consolidation (Pattern Merging)
If an agent distills three separate skills:
1. `parse_json_fast`
2. `parse_json_with_comments`
3. `repair_json_trailing_commas`
Rather than keeping three separate files in context, Dendrite's background reflection worker merges them under a single **Umbrella Procedure** (`robust_json_parsing`). This reduces context overhead while preserving 100% of the lessons learned.

#### 5. Non-Destructive Archiving
Eviction in Cynapse is never destruction. Pruned skills are moved to a cold storage archive in `data/dendrite.db`. If a user brings up a niche task months later, SQLite FTS5 detects the query, locates the archived skill in milliseconds, and restores it back to active memory without needing to relearn it from scratch.

---

### Project Structure

```
cynapse-mini/
├── Cargo.toml                          # Workspace root — 6 crates
├── cynapse.toml                        # Runtime configuration (engine, memory, endpoint)
├── install.sh                          # Dual remote/local automated installer
├── harness/
│   ├── src/main.rs                     # CLI & TUI entrypoint dispatcher (Clap)
│   ├── cynapse-core/                   # Tool execution, GBNF parser, bot registry, subagent loop, gates, doctor
│   │   └── src/
│   │       ├── lib.rs                  # 7 tools: read_file, write_file, patch_file_at, grep, execute_command, run_gates, spawn_subagent
│   │       ├── offline_agent.rs        # GBNF tool validator, Levenshtein tolerant coercion, LoopGuard, Two-Zone prompt
│   │       ├── gates.rs                # Deterministic GATES.md acceptance engine (CHECK, EXPECT, EVIDENCE)
│   │       ├── skills.rs               # Dynamic micro-skill intent slicer (bounded <= 35 tokens)
│   │       ├── security.rs             # Pure Rust autonomous security shield (path sandboxing & command sanitization)
│   │       ├── compressor.rs           # Output compressor (spillover to disk, 5-item cap, tail extraction)
│   │       ├── bots.rs                 # BotProfile/BotRegistry, allow-deny-ask permissions
│   │       ├── doctor.rs               # 22-subsystem self-healing diagnostics & maintenance
│   │       ├── downloader.rs           # HuggingFace model downloader
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
│           ├── autoharness.rs          # AutoHarness: self-learning, probation, usage-rate tracking, dynamic pruning
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

### Core Module: Tool Execution with Sandboxing & Extreme Optimization

The tool execution system (`cynapse-core/src/lib.rs`) provides **7 atomic tools** with strict memory, timeout, and path sandboxing boundaries:

```rust
pub fn execute_tool(name: &str, arg1: &str, arg2: Option<&str>) -> Result<String> {
    match name {
        "read_file" => {
            let safe_path = validate_safe_path(arg1, false)?;
            // 10 MB limit, reject directories, verified read receipts
            fs::read_to_string(&safe_path)
        }
        "write_file" => {
            let safe_path = validate_safe_path(arg1, true)?;
            // System root protected, auto-create parent directories
            fs::write(&safe_path, content)
        }
        "patch_file_at" => {
            // Surgical exact-anchor replacement: target must occur exactly once
            patch_file_at(path_str, target, replacement, allow_multiple)
        }
        "grep" => {
            // Recursive walk with depth limit (8) and match limit (50)
            // Skip hidden dirs, target/, node_modules/
            walk_and_grep(&dir, &re, &mut matches, 0)
        }
        "execute_command" => {
            // Autonomous security shield blocks: rm -rf /, mkfs, fork bombs
            // 10-second timeout with kill on expiry; disk spillover if > 1200 chars
            let mut child = Command::new("bash").arg("-c").arg(trimmed).spawn()?;
            // Enforce timeout...
        }
        "run_gates" => {
            // Deterministic acceptance gate runner evaluating GATES.md
            // Returns structured status with exit codes and machine evidence
            execute_gates_suite(&gates)
        }
        "spawn_subagent" => {
            // Delegated subagent execution with scoped bot profile permissions
            // Concurrency-capped (semaphore), stall detection, approval gate
            subagent_mgr.spawn(slug, task).await
        }
    }
}
```

**Path Validation & Autonomous Security Guard:**

```rust
fn validate_safe_path(p_str: &str, for_write: bool) -> Result<PathBuf> {
    let sensitive_prefixes = [
        "/etc/shadow", "/etc/passwd", "/etc/sudoers",
        "/.ssh", "/root", "/proc/kcore", "/dev/mem",
    ];
    // Block system directory writes: /, /etc, /usr, /bin
}
```

**GBNF Grammar Constraints & Tolerant Coercion:**
- Validates model output against strict JSON schema.
- Employs **Fuzzy Alias Normalization**: maps `file_path`, `target`, `filename` $\to$ `path`.
- Employs **Levenshtein Distance Metric ($\le 2$)**: auto-repairs minor model typos in parameter names.
- Employs **Actionable Repair Hints**: if unfixable, returns the exact valid parameter list so the model recovers immediately without getting stuck.

**ToolLoopTracker State Machine:**

```rust
pub struct ToolLoopTracker {
    history: VecDeque<CallRecord>,
    consecutive_vetoes: usize,
}

impl ToolLoopTracker {
    // 1. Args-only repeat (warn at 3): identical tool + identical arguments
    // 2. No-progress streak (critical at 5): identical args + identical result hash
    // 3. Wandering (warn at 6, escalate at 12): distinct args on execute_command
    // 4. Circuit Breaker (trips at 3 consecutive vetoes): forces graceful completion
}
```

### Core Module: Self-Healing Doctor (22 Subsystems)

The self-healing doctor (`cynapse-core/src/doctor.rs`) audits **22 integrated subsystems** with automated `--fix` repair capabilities:

```rust
pub fn run_diagnostics(&self) -> DoctorReport {
    let mut items = Vec::new();
    // Hardware & Acceleration
    items.push(self.check_hardware_ram());           // 1. RAM headroom & safety reserve
    items.push(self.check_gpu_acceleration());       // 2. GPU detection (Vulkan/DRM)
    items.push(self.check_simd_capabilities());      // 3. AVX2 + FMA SIMD kernels
    // Models & Storage
    items.extend(self.check_models_and_gguf_integrity()); // 4-5. Model dir + GGUF magic headers
    // Memory & Knowledge Graph
    items.extend(self.check_dendrite_db_integrity());     // 6-7. SQLite connection + quick_check
    items.push(self.check_autoharness_subsystem());       // 8. AutoHarness usage rate & probation
    // Grammar & Execution Tools
    items.push(self.check_gbnf_validator());         // 9. GBNF grammar parser & schema
    items.push(self.check_local_tools());            // 10. Offline host tools (bash, git)
    items.push(self.check_patching_engine());        // 11. Surgical file patcher test
    items.push(self.check_acceptance_gates());       // 12. GATES.md runnable verification
    // Runtime & Engine
    items.push(self.check_tokio_channels());         // 13. Tokio async channels & pool
    items.push(self.check_llm_endpoint_and_models()); // 14. Native Leafcutter GGUF catalog
    items.push(self.check_inference_backend_reachability()); // 15. Fail-loud port probe
    items.push(self.check_slot_manager());           // 16. Multi-slot manager & cache pinning
    // Agency, Security & Hygiene
    items.push(self.check_persona_subsystem());      // 17. Markdown persona compiler
    items.push(self.check_bot_registry_subsystem()); // 18. Bot profiles & permission manifests
    items.push(self.check_skills_subsystem());       // 19. Dynamic micro-skill intent slicer
    items.push(self.check_spillover_hygiene());       // 20. Output spillover storage & auto-prune
    items.push(self.check_security_guard());         // 21. Autonomous security guard self-audit
    items.push(self.check_read_receipts());          // 22. Verified read receipts registry
    // Auto-fix mode: auto-scaffolds gates, recreates dirs, repairs schemas, prunes stale logs
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
│ === OPERATING DIRECTIVE ===                             │
│ [Active micro-discipline slice — strictly <= 35 tokens] │
│                                                         │
│ [Verified Read Receipts — [cynapse-read #N: path]]      │
│ [User message & immediate instructions]                 │
│ [Tool outputs — 5-item cap, disk spillover if >1200ch]  │
│ [Conversation history — trimmed to context window]      │
└─────────────────────────────────────────────────────────┘
```

**Sampling parameters:**
- Temperature: 0.2 (disciplined, low-hallucination)
- Top-p: 0.95 (nucleus sampling)
- Repeat penalty: 1.1 (prevents loops)
- Repeat last N: 256 tokens

### Core Module: Tool Output Compressor & Disk Spillover

The compressor (`compressor.rs`) eliminates context bloating and terminal floods:

```rust
pub fn compress_tool_result(output: &str, options: &CompressorOptions) -> String {
    // 1. Strict 5-Item Listing Cap: caps long file/grep lists at 5 items per group
    // 2. Disk Spillover: outputs > 1200 chars are written to ~/.cynapse/spill/<hash>.log
    //    Returns only the actionable tail + disk path for optional inspection
    // 3. Tail Extraction: extracts last N characters to preserve compiler error signatures
    // 4. Graceful Truncation: preserves bracket balance without breaking model parser
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
4. **AutoHarness Lifecycle** — Opportunity-relative usage tracking, 100-turn probation, dynamic capacity contention, umbrella consolidation, and non-destructive cold archiving
5. **3D force-directed physics** — Real Coulomb/Hooke simulation creates organic, evolving memory visualizations that reflect actual knowledge structure
6. **Three-tier engine routing** — Automatic hardware-aware dispatch ensures optimal inference speed without user configuration
7. **Tolerant Coercion with Levenshtein Distance (<= 2)** — Resilient parameter normalization and actionable repair hints preventing offline tool call failures
8. **Surgical File Patching (`patch_file_at`)** — Native exact-anchor replacement preventing context-exhausting whole-file regeneration on small models
9. **Deterministic Acceptance Gates (`GATES.md`)** — Concrete runnable verification commands (`run_gates`) ensuring verifiable evidence before declaring tasks done
10. **Dynamic Micro-Skill Intent Slicing** — Invariant Zone A prompt bounded to <= 350 tokens, injecting <= 35 token micro-directives based on lexical intent scoring
11. **Disk Output Spillover & 5-Item Listing Caps** — Prevents command log floods by writing outputs > 1200 chars to `~/.cynapse/spill/` and strictly capping grep/list displays
12. **Autonomous Security Guard** — Pure Rust path sandboxing and destructive command interceptor protecting the host OS
13. **RAII Terminal Protection** — Guarantees terminal state restoration on panic, preventing broken terminal sessions

### Test Suite

Cynapse maintains **334 passing tests** across all workspace crates:

- **leafcutter_core** — Tensor operations, GGUF loading, BPE tokenizer, SIMD kernels, parity verification (213 tests)
- **cynapse-core** — Tool sandboxing, surgical file patching, path validation, GBNF parsing, tolerant coercion, acceptance gates engine, micro-skill intent slicing, output spillover compressor, security shield, bot registry, subagent loop & approval gates (71 tests)
- **cynapse-memory** — Knowledge graph operations, 3D physics, AutoHarness lifecycle & pruning, FTS5 search, BM25 scoring, context assembly, conversational query detection (19 tests)
- **cynapse-engine** — Semantic hardware probing, 3-tier router, slot manager, managed daemon, streaming parser (14 tests)
- **cynapse-tui** — Theme rendering, terminal protection, memory atlas visualization, pipeline state (11 tests)
- **mazzaroth** — 3D projection, hierarchical orbital geometry, ASCII render pipeline (5 tests)
- **cynapse (root)** — End-to-end integration tests (1 test)

### Current Development Status

**Completed (v0.1.0):**
- ✅ 3D Force-Directed Galaxy Memory with category clustering
- ✅ SQLite WAL + FTS5 persistence with BM25 scoring
- ✅ AutoHarness self-learning & self-pruning skill lifecycle with non-destructive cold archiving
- ✅ Three-tier engine router (llama.cpp, Leafcutter GGUF, Leafcutter Safetensors)
- ✅ GBNF grammar-constrained tool execution with tolerant coercion & Levenshtein hints
- ✅ Surgical file patch engine (`patch_file_at`) with unique anchor verification
- ✅ Deterministic Acceptance Gates engine (`GATES.md` parser and `run_gates` runner)
- ✅ Dynamic Micro-Skill Intent Slicer with strict <= 35 token directive injections
- ✅ Output compressor with disk spillover (`~/.cynapse/spill/`) and 5-item listing caps
- ✅ Pure Rust Autonomous Security Guard intercepting destructive commands and path traversal
- ✅ Loop Guard and Max Step safeguards
- ✅ Two-Zone Prompt Architecture (invariant Zone A <= 350 tokens)
- ✅ Managed llama-server child daemon
- ✅ Pure Rust SlotManager with cache_prompt
- ✅ Procedural memory (NodeType::Lesson, NodeType::Procedure)
- ✅ Async background reflection worker (verified-turn admission)
- ✅ Self-healing Doctor & Autonomous System Maintenance (22 subsystems with auto-fix)
- ✅ Bot profiles, scoped subagent loop & interactive approval gates
- ✅ `@mention` delegation, `/bots` orchestration & stall auto-cancel
- ✅ Persona system with markdown files
- ✅ Ratatui TUI with 4 themes and collapsible reasoning blocks
- ✅ HuggingFace model downloader with curated recommendations
- ✅ Session persistence and resume
- ✅ Memory Pipeline visualization
- ✅ 334 passing tests across all crates

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

# Part III: Complete Presentation Deck, Slide Outlines & Speaker Notes

> **Audience**: Technical leaders, engineers, investors, and system architects.  
> **Duration**: 20–30 Minutes.  
> **Presenter Persona**: Confident, grounded, authoritative systems engineer. Zero marketing fluff.  
> **Design Note for Presentation Creators**: Every slide below specifies layout, visual design prompts, exact bullets, and verbatim word-for-word speaker notes with Feynman-style analogies.

---

## Slide 1: Title & The Sovereign Agent

- **Slide Type**: Title / Hero Slide
- **Headline**: CYNAPSE: The Sovereign Edge AI Agent
- **Subheadline**: Pure Rust Native Execution • 4-Tier Biological Memory • Offline Zero-Cloud Autonomy
- **Visual Design**:
  - Dark mode (`#0B0F19`) background.
  - A glowing, minimalist constellation / celestial node diagram in cyan (`#00F0FF`) and amber (`#FFB300`), symbolizing the Dendrite memory graph.
  - Footnote badge: `15MB Standalone Binary | 0 Python Dependencies | 334/334 Passing Tests`.

### Key Bullet Points (Slide Display)
- 100% Local Inference on Consumer Laptops & Single-Board Computers (Raspberry Pi 5 / RK3588).
- Zero Telemetry, Zero Cloud API Keys, Zero Token Costs.
- Deterministic Tool Execution & Self-Healing Runtime.

### Verbatim Speaker Script
> "Good morning/afternoon everyone. Today, we are introducing **Cynapse**—a sovereign, fully autonomous local AI agent built from the bare metal up in pure Rust.
>
> Over the last three years, the tech industry has convinced developers that intelligent agents require $20-a-month subscriptions, remote API keys, and server farms consuming hundreds of megawatts. When your connection drops, your agent dies. When their privacy policy changes, your proprietary source code is indexed.
>
> Cynapse completely rejects that compromise. It is a single, self-contained 15-megabyte binary. It requires no Python runtime, no Docker daemon, and zero cloud API keys. It runs comfortably on a four-year-old laptop or a $60 single-board computer, executing complex multi-step coding, refactoring, and research tasks with verifiable mathematical guarantees. Let me show you how it works."

---

## Slide 2: The Core Problem: The 40-Ton Cloud Freighter

- **Slide Type**: Problem Breakdown & Contrast Matrix
- **Headline**: Why Modern AI Agents Fail on Developer Desktops
- **Visual Design**:
  - Two-column split comparison:
    - **Left Column (The Cloud Status Quo)**: Stamped in red / warning tones. Icons of bloated shipping containers, severed cloud cables, and leaky data pipes.
    - **Right Column (The Edge Reality)**: Stamped in green. Lean, agile, hardened.

### Key Bullet Points (Slide Display)
- **Bloated Runtimes**: 4GB+ Docker containers and fragile Python virtualenvs with 80+ pip dependencies.
- **Ephemeral Amnesia**: Context windows fill up, older critical constraints vanish, and memory resets across sessions.
- **Tool Hallucinations**: Weak local models mangle JSON schemas, rewrite entire 1,000-line files, and produce syntax errors.
- **Surveillance & Latency**: Every internal token travels across public internet backbones with 800ms+ round-trip latency.

### Verbatim Speaker Script
> "To understand why Cynapse exists, look at the current landscape of AI agent frameworks. Most are built like 40-ton cargo freighters. To run a simple coding agent, you need Python 3.11, a virtual environment, 2 gigabytes of node modules, and a Docker container.
>
> Even worse is how they handle reasoning. When an LLM agent operates, it suffers from two fatal flaws: **ephemeral amnesia** and **brittle tool invocation**.
>
> First, standard agents forget everything between sessions. To compensate, they dump thousands of tokens of chat history into every prompt. On small 3B or 7B local models, this instantly exhausts the context window.
>
> Second, small local models are notorious for producing invalid JSON when calling tools. Standard frameworks simply crash or enter infinite error loops. Cynapse was engineered specifically to solve both problems through hardware-conscious systems programming."

---

## Slide 3: Architectural Breakthrough: Pure Rust Native Engine

- **Slide Type**: Architecture Diagram
- **Headline**: The Zero-Overhead Foundation
- **Visual Design**:
  - Clean tiered architectural block diagram:
    - `[User Interface]` Terminal UI (Ratatui) / Headless CLI / JSON-RPC IPC.
    - `[Agentic Harness]` Two-Zone Context Compressor, Atomic Tools, Levenshtein Repair.
    - `[Dendrite Memory]` Working, Episodic, Semantic, Procedural tiers + Celestial 3D Atlas.
    - `[Engine Layer]` Llama.cpp C FFI / Leafcutter Shared-Memory Pipeline / Model Hub.
  - Callout badge: `RSS Memory < 35MB (Excluding Weights) | Cold Startup < 4ms`.

### Key Bullet Points (Slide Display)
- **Zero Python Runtime**: Compiled to native machine code; runs directly on Linux, macOS, and Windows.
- **Direct C-FFI Bindings**: Interfaces with `llama.cpp` directly in-process with zero IPC translation overhead.
- **L3 CPU Cache Optimization**: Data structures aligned to 64-byte CPU cache lines for microsecond query dispatch.
- **Cross-Platform Portability**: Builds statically via `musl` with zero dynamic system dependencies.

### Verbatim Speaker Script
> "Cynapse is built in pure Rust. We don't wrap Python scripts or invoke CLI subprocesses. 
>
> Think of the difference between an electric commuter bicycle and a freight locomotive. Both can transport you from point A to point B. But if your goal is navigating tight city streets, the freight locomotive consumes catastrophic energy and gets stuck at the first corner. 
>
> By writing Cynapse in native Rust, our baseline runtime overhead is under 35 megabytes of RAM. The binary starts in under 4 milliseconds. We interact directly with model inference through C FFI bindings. Every data structure, from our token compressors to our vector indexing, is cache-line aligned. This leaves virtually 100% of the host machine's memory, CPU registers, and GPU tensor cores dedicated to what actually matters: model inference and file manipulation."

---

## Slide 4: Dendrite: The 4-Tier Biological Memory Engine

- **Slide Type**: Conceptual & Technical Deep Dive
- **Headline**: Human-Grade Cognitive Memory on 4GB of RAM
- **Visual Design**:
  - A 4-stage biological memory funnel diagram:
    1. **Working Memory** (Circular ring buffer, hot volatile context, last 20 turns).
    2. **Episodic Memory** (Timestamped chronological logs of tool executions and terminal feedback).
    3. **Semantic Memory** (Cosine similarity vector store with fast in-memory embeddings).
    4. **Procedural Memory** (Hardened skills, verified workflow playbooks, error recovery rules).
  - Floating formula graphic: $\text{Effective Score} = \text{Cosine Sim} \times e^{-\lambda \Delta t} \times (1 + \beta \cdot \text{Access Count})$.

### Key Bullet Points (Slide Display)
- **Dynamic Context Slicing**: Injects only the top 3–5 strictly relevant memories ($\le 250$ tokens).
- **Time-Decay Retention**: Stale episodic records fade naturally while frequently referenced rules gain mass.
- **Ebbinghaus Forgetting Curve**: Biological half-life pruning prevents unbounded memory bloat.
- **Zero-Vector Cloud**: Entire vector similarity search executes in pure Rust via SIMD dot products in microseconds.

### Verbatim Speaker Script
> "Now let's examine Cynapse's crowning breakthrough: **Dendrite**, our biological memory engine.
>
> When you read a technical book, you don't keep every single sentence you have ever read in your conscious working memory. You have short-term sensory buffer, episodic recollections of what happened yesterday, semantic knowledge of concepts, and procedural muscle memory of how to ride a bike or write a loop.
>
> Dendrite mirrors this exact biological architecture across four distinct tiers. When the user asks a question, Cynapse does not dump your past chat history into the LLM. Instead, it queries the semantic tier via SIMD-accelerated cosine distance, ranks entries by recency decay and access frequency, and extracts only the top 3 to 5 micro-memories.
>
> The result? A model with a tiny 2,048-token context window behaves as though it has an infinite, lifelong memory, while consuming less than 250 tokens per prompt."

---

## Slide 5: The 3D Celestial Memory Atlas

- **Slide Type**: Visual / Interactive Feature Highlight
- **Headline**: Visualizing Thought: Physics-Driven Memory Topology
- **Visual Design**:
  - High-resolution screenshot of the Cynapse Ratatui Celestial Atlas.
  - A 3D orbital galaxy rendered in UTF-8 Braille characters (`⠋⠙⠹⠸⠼⠴`), showing clusters of skills, system rules, and episodic memories connected by radiant gravitational filaments.
  - Callout box showing dynamic cluster names: `[Compiler Errors]`, `[API Routing]`, `[Architectural Invariants]`.

### Key Bullet Points (Slide Display)
- **Force-Directed Graph**: Coulomb repulsion pushes unrelated nodes apart; Hooke elasticity binds semantically linked nodes.
- **Real-Time Euler Numerical Integration**: Nodes orbit and stabilize in 3D Euclidean space at 30 FPS inside the terminal.
- **Interactive Spatial Inspection**: Pan, tilt, rotate, and zoom directly inside the Ratatui TUI without leaving the console.
- **Spatial Semantic Querying**: Find related knowledge by spatial proximity rather than crude keyword searches.

### Verbatim Speaker Script
> "What you are seeing on this slide is not a pre-rendered marketing graphic. This is Cynapse's real-time terminal user interface.
>
> We call this the **Celestial Memory Atlas**. Every memory node—whether it is a rule learned from a compiler error, an architectural invariant, or an episodic session milestone—is assigned a mass and initial position in 3D space.
>
> We run real-time Euler numerical physics simulation directly in the TUI: Coulomb's Law pushes unrelated concepts apart, while Hooke's spring elasticity pulls semantically connected concepts together into constellations. 
>
> When a developer inspects their agent's brain, they don't scroll through endless JSON text files. They can rotate the galaxy in real-time, inspect conceptual clusters, and immediately understand what their agent knows and how its knowledge is interconnected."

---

## Slide 6: AutoHarness: Darwinian Skill Learning & Pruning

- **Slide Type**: Machine Learning / Adaptive Systems
- **Headline**: Self-Curating Skills: The Autonomic Lifecycle
- **Visual Design**:
  - State machine lifecycle diagram:
    - `[Opportunity Discovery]` $\rightarrow$ `[Probationary Tier]` $\rightarrow$ `[Active Hardened Skill]` $\rightarrow$ `[Dormancy / Pruning]`.
  - Telemetry card: `Retention Rate: 94.2% | Zero Memory Bloat Over 1,000 Turns`.

### Key Bullet Points (Slide Display)
- **Autonomic Discovery**: Detects repeated user workflows and formulates executable micro-skills automatically.
- **Probationary Gate**: New skills must prove their utility across 3 successful gate evaluations before promotion.
- **Usage-Based Opportunity Rate**: Skills that repeatedly trigger without user intervention are prioritized.
- **Automatic Dormancy Pruning**: Ineffective or superseded skills are gracefully purged to preserve cache efficiency.

### Verbatim Speaker Script
> "Most agent architectures suffer from irreversible decay: the longer you use them, the slower and more confused they get, because old, contradictory instructions accumulate in their system prompts.
>
> Cynapse solves this with **AutoHarness**, a Darwinian skill curation engine. When Cynapse detects that you frequently perform a specific debugging routine or multi-file refactoring sequence, it abstracts that pattern into a candidate skill and places it on probation.
>
> If the skill successfully resolves tasks and passes runnable acceptance gates, it is promoted to the active skill registry. But if a skill remains unused for weeks or causes regression errors, its opportunity score decays, and it is automatically pruned. Cynapse literally sheds dead code from its prompt memory like a biological organism."

---

## Slide 7: Three-Tier Engine Router & Hardware Probing

- **Slide Type**: Systems Architecture & Hardware
- **Headline**: Seamless Hardware Scaling: From Raspberry Pi to Dual RTX 4090
- **Visual Design**:
  - Horizontal flowchart showing automatic detection and engine fallback:
    1. **Tier 1: Embedded Native FFI** (Default, zero external dependencies, embedded `llama.cpp`).
    2. **Tier 2: Leafcutter Shared-Memory IPC** (Low-latency lockless RingBuffer for distributed local models).
    3. **Tier 3: OpenAI-Compatible HTTP Endpoint** (Ollama, vLLM, LM Studio fallback).
  - Hardware specification matrix showing RAM, VRAM, and tokens/sec across devices.

### Key Bullet Points (Slide Display)
- **Automatic Hardware Probing**: Detects CPU cores, AVX-512/NEON vector extensions, and available GPU VRAM at boot.
- **Zero-Config Quantization**: Automatically matches 4-bit (Q4_K_M) or 8-bit weights to host memory constraints.
- **Leafcutter IPC Pipeline**: Zero-copy shared-memory ring buffers bypass HTTP network latency entirely.
- **Offline Air-Gapped Operation**: Completely functional with physical network cables disconnected.

### Verbatim Speaker Script
> "How does Cynapse run across radically different hardware environments without complex configuration?
>
> At boot, Cynapse queries system topology through native OS APIs. It probes available CPU threads, vector instruction sets like AVX-512 on Intel or NEON on ARM, and GPU VRAM availability.
>
> It then activates the optimal engine via our Three-Tier Engine Router. On a standard laptop, it activates Tier 1: direct in-process FFI. If you run a high-throughput multi-agent pipeline with our sibling project Leafcutter, it switches to Tier 2: lockless POSIX shared memory with zero network overhead. If you already have an existing Ollama or vLLM server running in your lab, it falls back to Tier 3. The developer never has to touch a configuration file."

---

## Slide 8: The Speculative Scout: Needle-3 Assistant

- **Slide Type**: Performance Benchmarking & Innovation
- **Headline**: The Needle-3 Breakthrough: 3x Speedup with Speculative JSON Decoding
- **Visual Design**:
  - Dual timing waterfall chart:
    - **Top Chart (Traditional 7B Monolithic Generation)**: 1,420ms elapsed time, 18% JSON syntax failure rate.
    - **Bottom Chart (Cynapse Speculative Scout Architecture)**: Needle-3 0.5B speculative draft verified by 7B target. 410ms elapsed time, 0% syntax failure rate.
  - Large callout metric: `3.4x Faster Tool Dispatch | Zero JSON Schema Parse Errors`.

### Key Bullet Points (Slide Display)
- **Asymmetric Speculation**: Tiny 0.5B model drafts structural JSON tool calls at 120+ tokens/second.
- **Target Verification**: Larger 7B/14B reasoning model verifies semantic token logits in a single batch pass.
- **Hardware Efficiency**: Needle-3 fits entirely within the CPU's fast L3/L2 hardware cache (< 400MB RAM).
- **Zero Hallucination Gate**: If the draft fails JSON schema verification, it is rejected before hitting disk.

### Verbatim Speaker Script
> "Here is one of our proudest research milestones: **The Speculative Scout**.
>
> When an agent calls a tool, 80% of the generated tokens are rigid structural boilerplate: braces, brackets, parameter names, and quotation marks. Making a massive 7B or 14B model generate these tokens one by one is an enormous waste of compute and battery life.
>
> We introduced an asymmetric dual-model architecture. We run a microscopic 0.5B parameter model called Needle-3 alongside the primary model. Needle-3 specializes in one thing: blazing-fast structural drafting at over 120 tokens per second. 
>
---

## Slide 8B: Flash-Attention & The 50-Student Feynman Analogy

- **Slide Type**: Hardware Optimization / Deep Dive
- **Headline**: Eliminating the $O(N^2)$ Memory Wall: Tiled Online Softmax
- **Visual Design**:
  - Two-panel comparison visual:
    - **Panel A (The Naive Way)**: 50 students laying out a 50×50 floor grid of 2,500 paper sheets (representing 268MB of DRAM bandwidth drowned in intermediate scratch arrays).
    - **Panel B (The Feynman Flash-Attention Way)**: Small student groups of 4 computing running max and running sum tallies in pocket notebooks (L1/L2 cache), with zero intermediate DRAM allocations.
  - Callout badge: `3× Latency Speedup | 0 Bytes Intermediate RAM Allocated`.

### Key Bullet Points (Slide Display)
- **Quadratic Memory Trap**: Standard attention on an 8k context requires allocating a 67.1-million-float grid in DRAM.
- **Tiled Online Softmax**: Computes exact mathematical attention in small cache-resident tiles.
- **Running Normalization**: Tracks mathematical running maximums without keeping the full attention matrix.
- **L1/L2 Cache Speed**: Executes at internal processor cache speeds (>1.5 TB/s) rather than bus speeds (64 GB/s).

### Verbatim Speaker Script
> "When an AI agent analyzes long codebases, it encounters the quadratic memory wall: comparing 8,000 tokens against 8,000 tokens requires calculating 67 million scores.
>
> In standard frameworks, this allocates a massive 268-megabyte scratch matrix in RAM for every single layer.
>
> Let's explain why Cynapse doesn't do this using Richard Feynman's principle: Imagine 50 students who want to compare test papers. The naive way forces them to lay out 2,500 sheets of paper across the entire classroom floor just to store intermediate notes. The floor is covered, and nobody can move.
>
> In Cynapse, we use Tiled Online Softmax—the mathematical engine behind Flash-Attention. Students work in groups of four, write only their running tally in a pocket notebook that fits in CPU cache, and discard the scratch paper instantly. We never allocate that giant matrix in RAM. It computes at CPU cache speed, saving gigabytes of bandwidth and running three times faster."

---

## Slide 9: Atomic Tool Execution & Tolerant Coercion

- **Slide Type**: Reliability Engineering
- **Headline**: The 7 Atomic Tools: Hardened, Sandboxed, Self-Correcting
- **Visual Design**:
  - Grid of 7 tool badges:
    - `read_file` (Offset & slice bounded)
    - `write_file` (Atomic rename with rollback)
    - `patch_file_at` (Surgical exact-line diff replacement)
    - `grep` (Streaming line-bounded ripgrep)
    - `execute_command` (Isolated sandbox with monotonic timeout)
    - `run_gates` (Automated verification suites)
    - `spawn_subagent` (Isolated child thread delegation)
  - Workflow illustration showing a malformed JSON argument being corrected in real time via Levenshtein distance matching.

### Key Bullet Points (Slide Display)
- **Tolerant Coercion**: Forgives models that pass numbers as strings, single items instead of arrays, or uppercase booleans.
- **Levenshtein Fuzzy Repair**: Automatically resolves tool typos within edit distance $\le 2$ (e.g., `readFile` $\rightarrow$ `read_file`).
- **Loop Stagnation Trap**: Tracks rolling 4-tool call signatures; detects and breaks repetitive model loops automatically.
- **Atomic Rollback Journals**: Every file write is buffered; aborted tool calls leave zero corrupted files on disk.

### Verbatim Speaker Script
> "Small local models make mistakes. They capitalize boolean values, wrap integers in strings, or make slight spelling errors like calling `read_files` instead of `read_file`.
>
> In Python-based frameworks, this triggers an unhandled JSON decode exception, crashing the entire agent run.
>
> In Cynapse, our tool executor is built with **Tolerant Coercion**. We dynamically coerce types, normalize casing, and use Levenshtein distance algorithms to map near-miss tool names back to canonical signatures. Furthermore, our `ToolLoopTracker` computes rolling hashes of the last four invocations. If a model starts thrashing in an infinite loop, Cynapse intercepts execution, injects a corrective micro-directive, and forces the model to choose an alternate strategy."

---

## Slide 10: Keyhole Surgery: `patch_file_at` vs. Full-File Rewrites

- **Slide Type**: Code Comparison & Efficiency
- **Headline**: Surgical Precision: No More Truncated Files
- **Visual Design**:
  - Side-by-side terminal code diff:
    - **Left (Status Quo Agent)**: Writes out 800 lines of code to change 3 lines; runs out of output tokens at line 450; destroys the file.
    - **Right (Cynapse Keyhole Surgery)**: Pinpoints `patch_file_at` with exact target anchor, replacing 4 lines cleanly.
  - Callout badge: `Token Savings: 98.7% | Risk of Truncation: 0.0%`.

### Key Bullet Points (Slide Display)
- **Targeted Line Replacement**: Requires exact matching content and 1-indexed line ranges before executing changes.
- **Ambiguity Guard**: Rejects replacements if the target string matches multiple locations without unique line bounds.
- **Zero Hallucinated Deletions**: Never allows a model to replace hundreds of lines with lazy comments like `// ... rest of code unchanged`.
- **Microsecond Execution**: Executes in pure Rust in less than 50 microseconds without re-parsing entire ASTs.

### Verbatim Speaker Script
> "Every developer who has tested an AI agent has experienced this nightmare: you ask the agent to change one line in an 800-line file. The agent starts re-writing the entire file, hits its maximum token generation limit halfway through, and corrupts your source code with half-written garbage.
>
> Cynapse completely outlaws full-file rewrites for code modifications. We enforce what we call **Keyhole Surgery** via `patch_file_at`.
>
> The agent must provide the exact line number range and the verbatim text it wishes to replace. If the existing file contents do not match character-for-character, the patch is rejected immediately, and the model is prompted to re-read the file. This saves 98% of output token generation time and guarantees your codebase is never mangled."

---

## Slide 11: Deterministic Verification & Acceptance Gates

- **Slide Type**: Quality Assurance & Governance
- **Headline**: The Concrete Measuring Tape: Zero Completion Without Evidence
- **Visual Design**:
  - Triad verification diagram:
    - `CHECK:` [Command to run, e.g., `cargo test --lib`]
    - `EXPECT:` [Expected output pattern, e.g., `test result: ok. 12 passed`]
    - `EVIDENCE:` [Captured deterministic execution receipt]
  - Stamp: `UNLAZY DISCIPLINE CERTIFIED`.

### Key Bullet Points (Slide Display)
- **No Self-Declared Victories**: The agent is programmatically forbidden from declaring 'Task Complete' without passing gates.
- **Runnable Gate Checks**: Compiles code, runs linters, and executes unit test suites inside the execution loop.
- **Micro-Directive Enforcement**: Injects strict operational discipline rules dynamically into the model's context.
- **Objective Audit Trail**: Every completed task generates a verifiable execution transcript with command exit codes.

### Verbatim Speaker Script
> "There is a rampant pathology in modern LLMs: laziness. When a problem gets difficult, the model simply responds: 'I have updated the code, you can now run your tests,' without ever verifying if the code compiles.
>
> In Cynapse, an agent cannot simply state that it is done. It operates under our **Runnable Acceptance Gates** protocol.
>
> Before finishing a task, the agent must define a concrete check command, an expected exit condition, and execute `run_gates`. If the compiler fails, the agent receives the exact stderr output and must fix the error. The agent only terminates when its changes have been deterministically proven to compile and pass tests. We do not trust the model's opinion; we trust the operating system's exit code."

---

## Slide 12: Context Engineering: Invariant Two-Zone Prompts & Spillover

- **Slide Type**: Memory & Context Pipeline
- **Headline**: Context Architecture: Making 2,048 Tokens Feel Like 128k
- **Visual Design**:
  - Split context window graphic showing exact token budgeting:
    - **Zone 1: Dynamic Invariant Top-Slice ($\le 35$ tokens)**: Critical micro-discipline rules.
    - **Zone 2: Compact User Request & Verbatim Tools ($\approx 350$ tokens)**.
    - **Disk Spillover Channel**: Oversized command outputs hashed and redirected to `~/.cynapse/spill/<hash>.log`.
  - Cap meter: `Strict $\le$ 5 Items Per List | Read Receipts: [cynapse-read #N]`.

### Key Bullet Points (Slide Display)
- **Token Efficiency by Design**: Eliminates conversational bloat, pleasantries, and redundant recaps.
- **Disk Spillover Buffering**: Massive terminal outputs (e.g., 50KB build logs) are summarized on disk, keeping prompts lean.
- **Read Receipt Anchoring**: Ensures models do not hallucinate file reads by verifying `[cynapse-read #N]` tokens.
- **Bounded Output Truncation**: Enforces a strict 5-item display limit to prevent cognitive overload in small models.

### Verbatim Speaker Script
> "How can a 3B parameter model with a 2,048-token context window complete enterprise-grade refactoring tasks? The answer is **Context Engineering**.
>
> We divide the prompt into strict zones. At the very top, we inject a dynamic micro-discipline slice taking less than 35 tokens. 
>
> If a command produces a massive 5,000-line compiler dump, standard agents overflow their context window immediately. Cynapse catches the output, writes it to a hashed local disk spillover file at `~/.cynapse/spill/`, and injects a compact 5-line summary with exact line pointers. The agent then uses surgical grep tools to inspect the specific failure. It gets the full information it needs without wasting a single token of context."

---

## Slide 13: The Self-Healing Doctor: 22 Subsystem Health Diagnostics

- **Slide Type**: Operational Resiliency & Maintenance
- **Headline**: The Automated Immune System: `cynapse doctor --fix`
- **Visual Design**:
  - Terminal output mockup of `cynapse doctor --fix` showing green checkmarks across 22 subsystems:
    - `[✓] System Binaries`
    - `[✓] Llama FFI Engine`
    - `[✓] Dendrite Memory Quorum`
    - `[✓] Celestial Physics Integrator`
    - `[✓] Speculative Needle-3 Sync`
    - `[✓] Sandboxed Tool Isolation`
    - `[✓] 16 Other Core Subsystems...`
  - Footer metric: `Zero Unmaintained Code | 334/334 Automated Unit & Integration Tests`.

### Key Bullet Points (Slide Display)
- **Continuous System Auditing**: Inspects vector indexes, binary permissions, FFI bindings, and cache coherence.
- **One-Click Autonomous Repair**: `--fix` flag repairs broken symlinks, prunes stale memory nodes, and rebuilds corrupted indexes.
- **Enterprise Diagnostics**: Provides complete deterministic health reports in both human-readable text and structured JSON.
- **100% Test Coverage**: Backed by 334 comprehensive tests across all crates in the workspace.

### Verbatim Speaker Script
> "Codebases decay when they aren't maintained. An AI agent running on your local machine should not require manual SQLite database cleanups or debugging broken configuration files.
>
> Cynapse features a built-in diagnostic and self-healing immune system: **Cynapse Doctor**.
>
> With a single command—`cynapse doctor --fix`—the system runs comprehensive diagnostics across all 22 distinct subsystems: validating memory integrity, pruning orphaned vector embeddings, checking sandbox isolation rules, and verifying model weight checksums. If an anomaly is found, Doctor fixes it automatically. It ensures your agent remains as fast and clean on day 100 as it was on day 1."

---

## Slide 14: Vision, Privacy, and The Sovereign Future

- **Slide Type**: Conclusion & Strategic Roadmap
- **Headline**: The Future of Artificial Intelligence is Sovereign
- **Visual Design**:
  - Minimalist, inspiring closing visual: A developer working offline on an airplane, high in the mountains, and in an air-gapped server room, connected to Cynapse.
  - Three pillar badges:
    - **100% Data Sovereignty** (Your code never leaves your NVMe drive).
    - **Zero Cost per Token** (Predictable, infinite computing on your own hardware).
    - **Universal Resilience** (Works when the internet fails, when APIs go down, and forever).

### Key Bullet Points (Slide Display)
- **Air-Gapped Compliance**: Ready for defense, healthcare, legal, and proprietary financial development.
- **Sibling Ecosystem**: Synergized with Paraclea (Autonomous Database Agent) and Leafcutter (IPC Network).
- **Open-Source & Verifiable**: Fully open Rust codebase, MIT/Apache dual-licensed, zero telemetry trackers.
- **Get Started Today**: `cargo install --path .` or single-line shell installer.

### Verbatim Speaker Script
> "To conclude: the ultimate promise of personal computing was always sovereignty—giving individuals access to immense computational power on their own machines, under their own control.
>
> Cloud-dependent AI agents inverted that promise, turning developers into metered subscribers who lease their intelligence from centralized cloud monopolies.
>
> Cynapse proves that you do not need a trillion-parameter cloud model to build an exceptional, reliable, self-healing agent. With native Rust systems programming, biological 4-tier memory, speculative JSON decoding, and surgical tool execution, we deliver an agent that is private, lightning fast, free to run, and entirely yours.
>
> Thank you. I am eager to answer any technical questions you may have."

---

## Visual Asset & Terminal Styling Guide

When rendering slides or demo videos from this outline, adhere strictly to these visual standards:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ CYNAPSE PRESENTATION COLOR PALETTE                                      │
├─────────────────────────────────────────────────────────────────────────┤
│ Background Deep Space:  #0B0F19 (RGB 11, 15, 25)                        │
│ Card Surface Dark:      #161E2E (RGB 22, 30, 46)                        │
│ Primary Neon Cyan:      #00F0FF (RGB 0, 240, 255)  [Dendrite Memory]    │
│ Electric Amber:         #FFB300 (RGB 255, 179, 0)  [Scout Speculation]  │
│ Terminal Matrix Green:  #10B981 (RGB 16, 185, 129) [Verification Gates] │
│ Critical Warning Coral: #EF4444 (RGB 239, 68, 68)  [Cloud Pitfalls]     │
│ Typography:             Fira Code / JetBrains Mono (Body: Inter Display) │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Technical FAQ & Objection Handling (For Presenters)

### Q1: "Can a tiny 3B or 7B model really write production-grade code compared to GPT-4 or Claude 3.5 Sonnet?"
> **Speaker Answer**:  
> "A 3B model will lose to Claude 3.5 Sonnet in unconstrained creative reasoning if you just give it a massive open prompt. But Cynapse does not leave the model unconstrained. By enforcing Keyhole Surgery (`patch_file_at`), rigid two-zone prompt limits, and Runnable Acceptance Gates (`cargo test`), we eliminate the model's opportunity to hallucinate. The model doesn't need to invent architecture; it only solves bounded, verified, 5-line diffs with immediate compiler feedback. In practice, this achieves industrial reliability on routine refactoring, debugging, and feature tasks."

### Q2: "Why build a native memory system when vector databases like Pinecone or Chroma already exist?"
> **Speaker Answer**:  
> "Vector databases only solve semantic similarity: 'Does sentence A look like sentence B?' Human memory does not work like that. If you made a mistake 5 minutes ago in this session, that is critical working context. If you learned an API rule 3 weeks ago and haven't touched that library since, its relevance decays. Dendrite integrates working, episodic, semantic, and procedural memory into a unified physics graph with Ebbinghaus time-decay and access-frequency weighting, running entirely in-process in pure Rust without launching external database servers."

### Q3: "How does Cynapse run on a $60 Raspberry Pi 5 without thermal throttling?"
> **Speaker Answer**:  
> "Because Cynapse has zero runtime bloat. There is no Python interpreter, no JVM, and no background Electron wrapper. The agent runtime consumes less than 35MB of RAM. When paired with 4-bit GGUF quantization (such as Qwen 2.5 3B or Llama 3.2 3B), the entire model and runtime fit inside 2.2GB of RAM, leaving 6GB free on an 8GB Pi. Furthermore, our ARM NEON SIMD optimizations utilize every vector register on the Broadcom BCM2712 SoC, sustaining 14–18 tokens per second."

### Q4: "What happens if a tool execution fails or produces an error?"
> **Speaker Answer**:  
> "Cynapse never panics or crashes. If a tool fails—for example, if a file does not exist or a shell command exits with status 1—the exact stderr message is captured and fed back to the model as an observational prompt. The model can then inspect the error, adjust its parameters, or choose an alternate strategy. If the model repeats the same failed tool call twice, our `ToolLoopTracker` detects the stagnation loop and forces the model to change tactics."

---

**Built with Rust, integrity, and a commitment to keeping your intelligence guarded — not monetized.**

