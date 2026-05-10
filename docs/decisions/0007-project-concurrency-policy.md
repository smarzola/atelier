# ADR 0007: Project concurrency policy

## Status

Accepted

## Context

Atelier projects are durable folders shared by humans, Codex, and potentially multiple gateway users. Multiple Atelier threads may exist in one project at the same time, but write-capable Codex jobs can edit files, run commands, and mutate project-local state.

Allowing multiple managed workers to write to the same folder concurrently risks conflicting edits, confusing prompts, and corrupted runtime state. A future implementation may support safer parallelism through git worktrees or explicit read-only jobs, but the default must be safe and understandable.

## Decision

Atelier's default managed-work concurrency policy is `single-writer` per project.

A new managed job refuses to start while another managed job in the same project is:

- `running`; or
- `waiting-for-prompt`.

Before enforcing the writer slot, Atelier reconciles active jobs against `worker.json`. If the recorded worker process is gone, the job is marked `worker-lost` and no longer blocks the writer slot.

## Consequences

Benefits:

- Protects shared project folders from overlapping writes.
- Keeps the early runtime predictable.
- Makes conflicts explicit instead of surprising.
- Allows recovery/reconciliation to free stale writer slots.

Trade-offs:

- Legitimate parallel work is serialized for now.
- Long prompt waits can block new managed work until answered, timed out, or reconciled.

## Future options

Explicit project configuration may later opt into:

- read-only parallel jobs;
- git-worktree-per-job write isolation;
- queued single-writer jobs;
- unsafe parallel writes for projects that intentionally allow it.
