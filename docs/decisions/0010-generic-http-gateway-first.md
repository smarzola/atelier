# ADR 0010: Generic HTTP gateway first

## Status

Accepted

## Context

Atelier needs gateway support so external chat systems, webhooks, local tools, and future platform adapters can send messages into project-native workstreams. Telegram is an important target, but baking Telegram concepts directly into the core runtime would violate the existing design principle that gateway bindings are abstractions and platform-specific details should stay at the edge.

The current CLI already provides file-backed project registry, person memory, thread bindings, jobs, prompt records, and managed Codex app-server workers. The first gateway should reuse those primitives instead of inventing a separate agent loop or platform-specific model.

## Decision

Atelier will implement a generic local HTTP gateway before any platform-specific adapter.

The initial gateway command is:

```bash
atelier gateway serve --listen 127.0.0.1:8787
```

The gateway exposes JSON endpoints for generic events and runtime inspection:

- `GET /health` — readiness check.
- `GET /status` — global status dashboard data.
- `GET /jobs` — registered-project job list.
- `GET /prompts` — cross-project pending prompt inbox.
- `POST /prompts/respond` — write a prompt response using the existing durable response schema.
- `POST /events/message` — resolve a generic gateway message into person/thread/project routing and optionally start managed work.

The generic message event shape uses neutral fields:

```json
{
  "gateway": "example-gateway",
  "external_thread": "thread-example",
  "external_user": "user-example",
  "project": "example-project",
  "thread": "thread-example",
  "person": "alice",
  "text": "Do the work"
}
```

Gateway-specific adapters, such as Telegram, should translate platform events into this shape and call the generic gateway, or reuse the same core functions.

## Consequences

Benefits:

- Keeps Telegram assumptions out of core runtime.
- Creates a dogfoodable API for local and future gateway use.
- Reuses existing project aliases, person memory, prompt response schema, and managed worker artifacts.
- Makes integration tests deterministic with normal HTTP requests and fake Codex binaries.

Trade-offs:

- The first gateway is not a full Telegram bot.
- Authentication/authorization is minimal in the initial local gateway and must be hardened before exposing beyond localhost.

## Follow-ups

- Add gateway identity binding commands for external users to people.
- Add access control before non-local deployment.
- Add Telegram adapter once the generic gateway event model is stable.
- Add a daemon/supervisor if long-lived worker management outgrows CLI-spawned workers.
