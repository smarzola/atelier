# ADR 0006: Managed worker architecture

## Status

Superseded by [ADR 0011: Daemon is the Atelier orchestrator](0011-daemon-is-atelier-orchestrator.md).

## Context

Atelier managed work uses Codex `app-server` so prompt requests can be surfaced as durable Atelier prompt records and answered later. The first implementation ran from the CLI and started a hidden Atelier worker process per managed job. The worker owns one Codex app-server child process and writes file-first artifacts into the job directory.

This local model proved the worker/job file contract, but it is not the final product architecture. Atelier is an orchestrator, so managed Atelier work requires an always-alive daemon. Raw `cd project && codex` remains valid outside Atelier-managed work.

## Historical decision

The first alpha implementation used this local model:

- `atelier work --managed` creates a job directory and spawns `atelier __managed-worker`.
- The worker starts `codex app-server` and owns its stdin/stdout protocol stream.
- The worker records raw app-server JSON-RPC traffic in `protocol.jsonl`.
- The worker writes prompt records under `prompts/` and waits for response files under `responses/`.
- The worker forwards response files to Codex as JSON-RPC responses.
- The launcher records `worker.json`, `worker-stdout.log`, and `worker-stderr.log`.
- The job directory remains the durable source of truth for status, context, request, prompts, responses, logs, and result.

ADR 0011 changes the owner: the daemon should start and supervise these workers, while preserving the same file-first job contract instead of replacing it with opaque process state.

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
