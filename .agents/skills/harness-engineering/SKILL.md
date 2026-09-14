---
name: harness-engineering
description: Scaffolding, planning artifacts, context delivery, verification loops, tool interface design, memory architecture, and defensive engineering for reliable AI agent execution.
---

# Harness Engineering

Harness engineering is the discipline of designing the scaffolding — context delivery, tool interfaces, planning artifacts, verification loops, memory systems, and sandboxes — that surrounds an AI agent and determines whether it succeeds or fails on real tasks.

## Core Principles

1. **Scaffold Before Model**: Reliability comes from environment design, deterministic verification, and clean interfaces, not raw prompt coaxing.
2. **Acceptance Ledgers & Planning Artifacts**: Non-trivial tasks require durable planning files (`PLAN.md`, `GATES.md`, `IMPLEMENT.md`) with explicit verification commands before writing production code.
3. **Context Engineering as a Finite Resource**: Scope context surgically. Keep long-lived state in versioned filesystem artifacts rather than inflating prompts.
4. **Tool Design is Agent UX**:
   - Single conceptual responsibility per tool.
   - Deterministic, consistent output schemas.
   - Errors that explain exactly what to do next.
5. **Continuous Verification Loops**:
   - Every milestone must have an automated check or oracle.
   - An agent must verify its own work before declaring completion.
6. **Sub-agent Dispatch Discipline**:
   - Multi-agent teams must run with decoupled, disjoint file ownership (`OWNS:`).
   - Independent tasks are batched and dispatched concurrently.
   - Only dependent tasks run sequentially.
