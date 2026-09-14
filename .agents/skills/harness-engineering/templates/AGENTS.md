# AGENTS.md

> Project-level instructions for AI agents working in this repository.
> Place this file at the repo root. Agents should read it before starting any task.

## Project overview

<!-- One paragraph: what this project does, its tech stack, and its primary goals. -->

## Repository structure

```
src/          # Application code
tests/        # Test suite
docs/         # Documentation
scripts/      # Build and utility scripts
```

## Conventions

### Code style

<!-- Language, formatter, linter, and any project-specific conventions. -->

### Naming

<!-- File naming, function naming, variable naming conventions. -->

### Testing

<!-- How to run tests. What constitutes a passing test suite. -->

```bash
# Run all tests
<command>

# Run a single test file
<command>
```

### Commits

<!-- Commit message format, branching strategy, PR conventions. -->

## Tool permissions

<!-- What tools / file paths / commands the agent is allowed to use.
     Be explicit: agents perform better with clear boundaries than vague restrictions. -->

Allowed:
- Read and edit files under `src/`, `tests/`, `docs/`
- Run `<test command>`
- Run `<lint/format command>`

Restricted (ask before proceeding):
- Modifying `<critical config files>`
- Running destructive commands (`rm -rf`, database drops, etc.)
- Pushing to `main` or creating releases

Not allowed:
- Modifying CI/CD pipeline configuration without explicit instruction
- Installing new dependencies without explicit instruction

## Known constraints

<!-- Anything that would surprise an agent working here for the first time. -->
<!-- e.g., "The monorepo build tool is X, not Y", "Tests require a running Docker daemon" -->

## Verification gates

Before marking any task complete, the agent must verify:

- [ ] Tests pass (`<command>`)
- [ ] Linter passes (`<command>`)
- [ ] No new warnings introduced
- [ ] Changed files are within the permitted scope above

## Contact / escalation

If the agent cannot proceed without a decision that falls outside its permitted scope, it should stop and describe the blocker clearly rather than making an assumption.
