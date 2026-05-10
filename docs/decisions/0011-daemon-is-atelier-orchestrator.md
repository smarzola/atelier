# ADR 0011: Daemon is the Atelier orchestrator

## Status

Accepted

## Context

Atelier is an orchestrator around Codex, not merely a convenience CLI that sometimes starts background workers. Raw Codex usage remains valid without Atelier: a person can still enter a project folder and run `codex` directly. That path intentionally loses Atelier-specific orchestration such as person memory injection, gateway routing, prompt relay, job supervision, and multi-person coordination.

Atelier-managed work is different. If work is launched through Atelier, it must run through an always-alive orchestration layer. A CLI command that directly spawns a managed worker and then exits creates the wrong mental model: it makes orchestration incidental rather than central, and it weakens supervision, prompt delivery, completion notification, and recovery semantics.

The current alpha implementation uses `atelier gateway serve --supervise-workers` as the long-lived process. That is useful for dogfooding but names the boundary incorrectly. The gateway should be an interface hosted by the daemon, not the daemon itself.

## Decision

Atelier has a daemon, and the daemon is the required orchestration layer for Atelier-managed work.

The daemon owns:

- worker supervision and reconciliation;
- starting and tracking managed Codex app-server workers;
- gateway hosting, including generic HTTP and platform adapters;
- prompt relay and eventual prompt notifications;
- job completion notification and audit logging;
- project/thread/person routing for external events;
- recovery workflows for stale or interrupted managed jobs.

The gateway is contained inside the daemon. It is not the architectural owner of orchestration.

The CLI remains useful, but it is primarily a control client for the daemon and a file-inspection tool. CLI commands may still initialize home/project folders, inspect file-backed state, and perform explicit maintenance operations, but managed Atelier work should require a reachable daemon. If the daemon is not running, commands that start managed work should fail with a clear message explaining how to start it.

Raw Codex remains outside this requirement. `cd project && codex` must remain valid, but it is not an Atelier-managed run.

## Target command model

Preferred long-lived runtime:

```bash
atelier daemon run \
  --listen 127.0.0.1:8787 \
  --auth-token-env ATELIER_GATEWAY_TOKEN
```

Transitional compatibility may keep:

```bash
atelier gateway serve ...
```

as an alias or developer-facing wrapper around the daemon's HTTP gateway service, but documentation should teach the daemon as the primary runtime.

Managed work should flow through the daemon:

```text
CLI / HTTP gateway / platform adapter
        |
        v
atelier daemon
        |
        +--> identity + project + thread routing
        +--> job creation and writer-slot policy
        +--> managed Codex worker lifecycle
        +--> prompt relay and completion notifications
```

## Consequences

Benefits:

- Atelier's product model is clear: Atelier is the orchestrator, and orchestration is always alive.
- Managed work has one owner for worker lifecycle, prompt relay, notifications, recovery, and audit logs.
- Gateway adapters become daemon-hosted interfaces rather than independent orchestration surfaces.
- Future schedulers, watchers, and cross-project coordination have a natural home.

Trade-offs:

- Managed work now depends on daemon availability.
- The CLI needs client/server behavior for work-starting commands.
- Local alpha behavior that directly spawns hidden workers must be migrated or explicitly marked as legacy/developer-only.

## Migration notes

1. Add `atelier daemon run` as the primary long-lived command.
2. Move worker supervision from `atelier gateway serve --supervise-workers` into the daemon default behavior.
3. Host the existing HTTP gateway endpoints inside the daemon.
4. Make `atelier gateway serve` a compatibility alias, a submode of `atelier daemon run`, or a deprecated developer command.
5. Change `atelier work --managed` to submit work to the daemon instead of spawning `atelier __managed-worker` directly.
6. Keep file-first job directories as the source of truth, so daemon restart does not discard project state.
7. Update README, usage docs, and roadmap to say managed Atelier work requires the daemon.
