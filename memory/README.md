# Cynapse Memory Subsystem (DENDRITE Rust Core)

**Dendrite** is Cynapse's native, lightweight, dependency-free Rust memory core built on embedded SQLite (`rusqlite`) with FTS5 full-text indexing, BM25 graph ranking, and 4-tier knowledge classification.

## Key Features

1. **4-Tier Memory Model**:
   - **L0 (TurnLog)**: Raw conversation turns & chat history logs.
   - **L1 (AtomicFact / Memory / Event / Person)**: Episodic facts, user preferences, and entity nodes.
   - **L2 (Procedure / Project / Concept)**: Workflows, procedural skills, and domain concepts.
   - **L3 (Identity)**: Core agent identity and persona nodes (`identity`, `soul`, `agents`, `tools`).

2. **Wiki-Links & Backlinks**:
   - Automatic `[[wikilink]]` extraction and `#tag` indexing.
   - Bi-directional backlink management (`links` and `backlinks`) updated automatically on every node mutation.

3. **Hybrid Search & Prompt Assembly (`DendriteContext`)**:
   - SQLite FTS5 porter-unicode tokenized search + in-memory BM25 ranker.
   - Token-budgeted system prompt generation (40% identity budget + 60% message relevance / recent nodes).
   - 5-minute prompt cache with instant dirty-flag invalidation upon graph mutations.

4. **Async Reflection Worker (`ReflectionWorker`)**:
   - Non-blocking Tokio background worker that distills chat turns (L0) into atomic facts (L1) and procedures (L2).
   - Guarded by atomic concurrency flags (`in_flight`) to prevent overlapping LLM background calls.

---

## Directory Structure

```
memory/
├── dendrite_core/
│   ├── mod.rs           # Core module re-exports
│   ├── graph.rs         # In-memory graph, 4-tier model, BM25 ranker, wikilinks
│   ├── store.rs         # SQLite persistence, FTS5 index & fallback tables
│   ├── context.rs       # Token-budgeted system prompt builder & prompt cache
│   └── reflection.rs    # Non-blocking async background reflection worker
└── README.md
```
