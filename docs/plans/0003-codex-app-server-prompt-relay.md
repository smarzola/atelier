# Plan 0003: Codex app-server prompt relay

## Goal

Replace terminal-screen prompt relay ideas with a structured Codex app-server run path. Atelier should be able to start a Codex turn, observe typed events, persist pending prompts, and answer Codex requests from a CLI or gateway later.

## Non-goals

- Do not build a second agent loop.
- Do not screen-scrape the Codex TUI.
- Do not force a model override; omitted model means Codex default.
- Do not mutate `~/.codex`, project `.codex/config.toml`, or project `AGENTS.md` for runtime context injection.
- Do not implement Telegram in this slice.

## Slice 1: Protocol fixtures and event model

Add a small internal event model for app-server traffic:

- outbound client request;
- inbound server response;
- inbound server notification;
- inbound server request;
- pending prompt record.

Tests should use JSON fixture lines for:

- `item/commandExecution/requestApproval`;
- `item/fileChange/requestApproval`;
- `item/permissions/requestApproval`;
- `item/tool/requestUserInput`;
- `mcpServer/elicitation/request`;
- `serverRequest/resolved`.

Expected output: Atelier can parse the request id, method, thread id, turn id where present, item id where present, reason/message, and safe renderable summary.

## Slice 2: Minimal app-server client

Implement a small adapter that spawns:

```bash
codex app-server
```

with stdin/stdout pipes, then:

1. sends `initialize` with `experimentalApi: true`;
2. sends `initialized`;
3. sends `thread/start` with `cwd`, `approvalPolicy`, sandbox/permissions, and optional explicit model;
4. sends `turn/start` with Atelier's explicit context preamble plus the user's prompt;
5. stores raw protocol JSONL under the job directory.

The fake-Codex test binary should support a deterministic app-server mode that emits fixture events and waits for a response to a request.

## Slice 3: Local prompt commands

Add local prompt-inspection commands before any gateway integration:

```bash
atelier work <project> --thread <thread> --as <person> "prompt"
atelier prompts list <project> --thread <thread>
atelier prompts show <project> <prompt-id>
atelier prompts respond <project> <prompt-id> accept
atelier prompts respond <project> <prompt-id> decline
atelier prompts respond <project> <prompt-id> cancel
```

The initial implementation can keep the process alive only for the foreground command. If durable background processes are not yet available, document that as a limitation and still persist prompt records and protocol logs.

## Slice 4: Durable daemon worker

Move app-server sessions behind an Atelier daemon/worker:

- one worker process per active project or job;
- durable job state on disk;
- pending prompts survive CLI reconnects;
- `respond` writes the JSON-RPC response to the correct app-server stdin or websocket;
- stale prompts close when `serverRequest/resolved`, turn completion, interruption, or process exit arrives.

## Slice 5: Gateway binding

Route pending prompts to gateways using the existing gateway binding model:

- render command approvals with command, cwd, reason, and available decisions;
- render file approvals with paths/diff summaries and available decisions;
- render permission requests with requested read/write/network scope;
- render MCP elicitations as form or URL cards;
- render tool user-input prompts as short questions;
- restrict who can answer based on project/thread binding and person identity.

## Prompt record shape

```json
{
  "id": "prompt_...",
  "codex_request_id": 0,
  "method": "item/commandExecution/requestApproval",
  "project": "example-project",
  "atelier_thread": "example-thread",
  "codex_thread_id": "019e...",
  "codex_turn_id": "019e...",
  "codex_item_id": "call_...",
  "status": "pending",
  "summary": "Approve command: /bin/bash -lc '...'",
  "params": {},
  "available_decisions": ["accept", "decline", "cancel"],
  "created_at": "...",
  "resolved_at": null
}
```

## Open questions

- Should Atelier run one long-lived app-server per daemon, per project, or per job?
- Should app-server transport be stdio first, then websocket/unix socket later?
- How much of Codex's protocol should Atelier type strongly versus pass through as JSON values?
- How should a gateway present `acceptForSession` and policy-amendment options safely?
- Which prompt decisions require project admin rights versus ordinary participant rights?
