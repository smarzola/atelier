# Decision 0002: No hidden Codex home swaps for context injection

## Status

Accepted for initial design.

## Context

One possible design was to create per-person `CODEX_HOME` directories and run Codex with a different home for each person. This would make it easy to inject person memory through generated global `AGENTS.md` files.

However, this has drawbacks:

- It may surprise users because Codex behavior changes depending on an invisible environment variable.
- It risks divergence from normal `cd project && codex` behavior.
- It may require duplicating or shadowing existing Codex configuration.
- It creates ambiguity about whether global Codex config belongs to the person, Atelier, or the machine.

## Decision

Atelier should not use hidden `CODEX_HOME` swapping as the default context injection mechanism.

Atelier should not rewrite:

- a user's `~/.codex/AGENTS.md`;
- a user's `~/.codex/config.toml`;
- project `AGENTS.md`;
- project `.codex/config.toml`;

merely to inject runtime person context.

For gateway and non-interactive tasks, Atelier should inject person context in the invocation itself, for example as a clearly delimited preamble to `codex exec`.

For interactive local use, Atelier may provide an explicit wrapper such as:

```bash
atelier codex
```

The wrapper must be transparent about what it adds. It should not silently mutate Codex files.

## Consequences

Positive:

- Raw Codex behavior remains understandable.
- Project folders remain the source of project truth.
- Person context injection is explicit and auditable.
- Multi-person gateway use does not require filesystem-level Codex home juggling.

Negative:

- Interactive context injection may be less elegant until Codex exposes a first-class runtime context hook.
- Some session resume flows need careful handling to preserve person context without duplicating it excessively.

## Revisit when

- Codex provides a first-class system/developer/runtime context injection mechanism suitable for wrappers.
- Users explicitly request opt-in managed Codex profiles.
- We need stronger isolation for hosted multi-tenant deployments.
