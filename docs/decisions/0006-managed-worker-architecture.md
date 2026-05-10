# ADR 0006: Managed worker architecture

## Status

Accepted

## Context

Atelier managed work uses Codex `app-server` so prompt requests can be surfaced as durable Atelier prompt records and answered later. The first implementation runs from the CLI and starts a hidden Atelier worker process per managed job. The worker owns one Codex app-server child process and writes file-first artifacts into the job directory.

Future gateway work will need a daemon that can supervise workers over longer periods, but the repository already needs a concrete local architecture that is inspectable, testable, and dogfoodable without requiring a daemon.

## Decision

Atelier will keep the current local model as the first managed-worker architecture:

- `atelier work --managed` creates a job directory and spawns `atelier __managed-worker`.
- The worker starts `codex app-server` and owns its stdin/stdout protocol stream.
- The worker records raw app-server JSON-RPC traffic in `protocol.jsonl`.
- The worker writes prompt records under `prompts/` and waits for response files under `responses/`.
- The worker forwards response files to Codex as JSON-RPC responses.
- The launcher records `worker.json`, `worker-stdout.log`, and `worker-stderr.log`.
- The job directory remains the durable source of truth for status, context, request, prompts, responses, logs, and result.

A later daemon may supervise these workers and provide an API/gateway surface, but the daemon must preserve the same file-first job contract instead of replacing it with opaque process state.

## Consequences

Benefits:

- Managed work is useful before a daemon exists.
- Jobs are inspectable and recoverable from files.
- CI can test the workflow with fake Codex app-server processes.
- Gateway work can build on proven local primitives.

Trade-offs:

- CLI-spawned workers are less ergonomic than daemon-supervised workers.
- Cross-process coordination is intentionally simple and file-backed.
- Recovery after worker exit restarts from saved context rather than reviving the old process.

## Follow-ups

- Add a daemon that supervises workers and exposes local gateway APIs.
- Keep daemon state derived from job files where possible.
- Continue using Codex app-server as the prompt-relay substrate.
