# ADR 0008: Prompt response schema

## Status

Accepted

## Context

Codex app-server can ask Atelier for approvals, user input, permission changes, and MCP elicitation responses through server-initiated JSON-RPC requests. Atelier must expose a response UX that works locally and can later be reused by gateways.

Different prompt types need different payload shapes. Command approvals may only need a decision. User input and MCP elicitation may need text. Some Codex prompt types may evolve faster than Atelier, so an escape hatch is required.

## Decision

Atelier prompt responses use a small durable JSON object written to:

```text
.atelier/jobs/<job-id>/responses/<prompt-id>.json
```

The supported local CLI forms are:

```bash
atelier prompts respond <project> <prompt-id> accept
atelier prompts respond <project> <prompt-id> answer --text "example answer"
atelier prompts respond <project> <prompt-id> accept --json '{"decision":"accept"}'
```

Semantics:

- decision-only responses become `{"decision":"<decision>"}`;
- text responses become `{"decision":"<decision>","text":"<text>"}`;
- raw JSON responses are forwarded exactly as provided after JSON parsing;
- when Codex provides `availableDecisions`, Atelier validates the decision before writing the response unless raw JSON is used.

The worker forwards the response object as the JSON-RPC `result` for the original Codex request id.

## Consequences

Benefits:

- One schema works for CLI, daemon, and gateways.
- Common approvals stay simple.
- Text prompts do not require raw JSON.
- Raw JSON provides forward compatibility with richer Codex payloads.

Trade-offs:

- Atelier does not yet provide specialized forms for every Codex decision variant.
- Gateways must preserve enough structure to write the same response object.

## Follow-ups

- Add gateway UI helpers for common decision/text cases.
- Add specialized UX for richer Codex decisions when needed.
- Keep the raw JSON escape hatch for protocol evolution.
