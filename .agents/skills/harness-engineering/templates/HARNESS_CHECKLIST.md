# Harness Review Checklist

> Run through this before shipping a harness to production or handing it off.
> A failing item is a blocker; a skipped item needs a written justification.

## Agent instructions (AGENTS.md)

- [ ] Project overview is accurate and up to date
- [ ] Repository structure reflects the current layout
- [ ] Tool permissions are explicit — allowed, restricted, and not-allowed are all specified
- [ ] Verification gates are defined and commands are correct
- [ ] No ambiguous instructions that could be interpreted multiple ways

## Tool design

- [ ] Each tool has a clear, unambiguous name
- [ ] Tool schemas are minimal — no optional fields that the agent won't use
- [ ] Error messages tell the agent what to do next, not just what went wrong
- [ ] Tool return values are consistent (same shape on success and failure)
- [ ] No tool does more than one conceptual thing

## Context delivery

- [ ] Context is scoped to what the agent needs for this task — not the entire codebase
- [ ] Long-lived state (plans, decisions, progress) is in files, not in the prompt
- [ ] Context compaction strategy is defined for multi-session tasks
- [ ] No sensitive data (secrets, credentials) in agent-accessible context

## Planning artifacts

- [ ] PLAN.md exists for non-trivial tasks
- [ ] Milestones have explicit verification commands
- [ ] Scope boundaries (in-scope / out-of-scope) are written down
- [ ] IMPLEMENT.md captures decisions and deviations as they happen

## Permissions & sandbox

- [ ] Agent runs with the minimum permissions needed for the task
- [ ] Destructive operations require explicit confirmation
- [ ] Network access is scoped if possible
- [ ] File system access is scoped to project directories

## Verification loop

- [ ] Tests exist for the agent's outputs
- [ ] The agent can run the verification command itself (not just "human review")
- [ ] Verification runs automatically on task completion, not just on PR
- [ ] Eval criteria are written down before the task starts, not after

## When this harness component should be removed

> Every harness component exists because the model can't do something yet.
> Document what capability improvement would make this component unnecessary.

| Component | Exists because | Can be removed when |
|---|---|---|
| | | |

---

*Reviewed: YYYY-MM-DD*
*Reviewer:*
