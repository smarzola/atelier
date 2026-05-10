# ADR 0009: Recovery and resume semantics

## Status

Accepted

## Context

Atelier has two related but distinct continuity needs:

1. Recovering an Atelier managed job whose worker stopped or timed out.
2. Resuming a Codex-native session or thread when Codex has durable session state.

Conflating these concepts would make the runtime confusing. Recovery is an Atelier job lifecycle operation. Resume is a Codex-native continuity operation.

## Decision

Atelier will keep recovery and resume separate.

Recovery means:

- use an existing Atelier job directory;
- read saved `context.md`, `request.md`, and `status.json`;
- start a new worker for that job;
- update the same job status and artifacts;
- preserve the job id.

Resume means:

- call Codex-native resume functionality such as `codex exec resume --last` or `codex exec resume <session-id>`;
- use Codex session/thread lineage when available;
- create a new Atelier job for the resumed invocation unless a future ADR defines otherwise.

Managed app-server runs record Codex thread/session metadata in the Atelier thread's `codex-sessions.jsonl` file. Recovery may use that lineage for better future behavior, but the current local recovery primitive restarts from saved Atelier context because the old app-server process no longer exists.

## Consequences

Benefits:

- Operators can understand what will happen before running a command.
- Lost or idle workers can be recovered without pretending the old process survived.
- Codex-native resume remains available without Atelier reimplementing transcripts.

Trade-offs:

- Recovered jobs may repeat some work if Codex does not resume the exact app-server turn.
- Better native resume integration requires more Codex protocol discovery.

## Follow-ups

- Prefer Codex-native resume/thread APIs when the app-server protocol exposes a stable resume path.
- Add status output that distinguishes `recoverable` from `resumable`.
- Preserve all recovery attempts in an audit log.
