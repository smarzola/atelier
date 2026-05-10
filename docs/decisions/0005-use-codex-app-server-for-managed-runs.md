# ADR 0005: Use Codex app-server for managed prompt relay

## Status

Accepted

## Context

Atelier needs to support gateway-driven work where Codex may ask for human input while a turn is in progress. Examples include command approvals, file-change approvals, permission requests, MCP elicitations, and tool user-input prompts.

A naive implementation could spawn `codex` or `codex exec` in an interactive terminal/PTY and relay terminal bytes to a gateway. That is useful for local handoff, but it is not a good primitive for Telegram/webhook/daemon integrations:

- Codex's interactive CLI is a TUI, optimized for a terminal human.
- Terminal byte streams are hard to parse safely and accurately.
- Approval choices need structured payloads, not screen-scraped text.
- Gateways need durable pending-request records and exact response routing.

Codex exposes `codex app-server`, the same JSON-RPC interface used by rich clients. It supports thread lifecycle, turn lifecycle, streamed item events, approvals, MCP elicitations, dynamic tool calls, and structured server-initiated requests.

## Decision

Atelier managed runs will use `codex app-server`, not the Codex TUI, as the prompt-relay substrate.

Atelier may still keep a local passthrough mode for humans who explicitly want to attach directly to Codex in a terminal, but daemon/gateway work must be built around the app-server protocol.

## Expected flow

1. Atelier starts or connects to `codex app-server` for a project worker.
2. Atelier sends `initialize` and `initialized`.
3. Atelier starts or resumes a Codex thread with project `cwd`, approval policy, sandbox or permissions, and no model override unless explicitly requested.
4. Atelier sends user work via `turn/start`.
5. Atelier consumes JSON-RPC notifications for status, streaming messages, command execution, file changes, MCP calls, and final turn completion.
6. When Codex sends a server-initiated request, Atelier records it as a pending prompt and routes it to the bound human/gateway:
   - `item/commandExecution/requestApproval`
   - `item/fileChange/requestApproval`
   - `item/permissions/requestApproval`
   - `item/tool/requestUserInput`
   - `mcpServer/elicitation/request`
7. The gateway response is translated back to the exact JSON-RPC response for that request id.
8. `serverRequest/resolved` and final item/turn notifications close the pending prompt and update job state.

## Local evidence

A small app-server probe against Codex CLI `0.130.0` confirmed that:

- `thread/start` returns a Codex thread id, session path, current default model, loaded instruction sources, approval policy, sandbox, and permission profile.
- `turn/start` streams structured `thread/status/changed`, `turn/started`, `item/started`, `item/completed`, `item/agentMessage/delta`, and `turn/completed` notifications.
- Command approval arrives as a structured `item/commandExecution/requestApproval` JSON-RPC request with `threadId`, `turnId`, `itemId`, command, cwd, reason, proposed policy amendments, and available decisions.
- Sending a JSON-RPC response such as `{ "decision": "accept" }` resumes the Codex turn and Codex emits `serverRequest/resolved` followed by the authoritative final command item.

This is the correct integration point for gateway prompt relay.

## Consequences

- Atelier should add a Codex app-server adapter alongside the existing `codex exec` adapter.
- Managed/gateway jobs should record app-server protocol events as structured JSONL, plus derived summaries for human-readable job status.
- Pending prompts become first-class Atelier job state, keyed by Codex request id plus thread/turn/item ids.
- Gateway UI can render safe, typed prompt cards instead of terminal text.
- Raw `cd project && codex` remains valid for direct human use, but it is not the managed gateway protocol.
- `codex exec --json` remains useful for non-interactive one-shot automation, but it does not expose approval requests as a bidirectional protocol.
