# Thread Conversation Items Design

## Summary

Atelier threads are project-scoped conversations. The user-facing interface is an ordered stream of OpenAI-style conversation items. Every surface—CLI, local HTTP API, Telegram, and future gateways—sends items to a thread and receives items from that same thread.

This design intentionally hides jobs, prompt ids, worker state, and raw runtime events from the normal UX. Those artifacts still exist for audit/debugging, but the primary product model is:

```text
project -> thread -> conversation items
```

## Goals

- One interface for sending and receiving thread messages.
- Approvals are thread items, and replies to approvals are normal user message items.
- Gateways and CLI use the same item stream.
- The API shape resembles OpenAI Conversations API conventions.
- Local project folders remain the source of truth.
- Jobs/prompts remain auditable internal artifacts.

## Non-goals

- Do not use OpenAI's hosted Conversations API as storage.
- Do not remove job/prompt artifacts; demote them to debug/internal surfaces.
- Do not add a second agent runtime. Codex remains the only agentic backend.
- Do not make Telegram-specific semantics part of the core model.

## Concept mapping

```text
OpenAI conversation        Atelier thread
OpenAI conversation item   Atelier thread item
OpenAI metadata            Atelier routing/debug metadata
Responses/tool output      Codex app-server output translated to items
```

Atelier differs from OpenAI's hosted API in three important ways:

1. Threads are scoped to projects.
2. Items are stored in project-local files.
3. Items also carry local orchestration metadata for person/gateway/thread routing.

## File model

```text
example-project/
  .atelier/
    threads/
      thread-example/
        thread.toml
        items.jsonl
        pending.json
        delivery-cursors/
          cli-follow.json
          telegram-chat-1000-topic-77.json
        events.jsonl
        codex-sessions.jsonl
    jobs/
      job-example/
        status.json
        protocol.jsonl
        prompts/
        responses/
        result.md
```

### `items.jsonl`

Primary product-facing stream.

Each line is one `conversation.item`-like object.

### `pending.json`

Optional state for currently pending thread-level interaction:
```json
{
  "kind": "input_request",
  "item_id": "item-input-example",
  "choices": ["approve", "decline", "cancel"]
}
```

Internal job and prompt ids may be present in debug metadata, but normal clients should key off the thread item id and reply in the same thread.

### `events.jsonl`

Runtime/debug stream. Existing event code can remain during migration, but clients should not use raw events for normal UX.

## Item envelope

```json
{
  "id": "item-example",
  "object": "conversation.item",
  "sequence": 1,
  "type": "message",
  "role": "user",
  "content": [
    { "type": "input_text", "text": "Please update the docs." }
  ],
  "metadata": {
    "project": "example-project",
    "thread": "thread-example",
    "person": "alice",
    "source": "cli"
  },
  "created_at": 1770000000
}
```

Required fields:

- `id`
- `object`
- `sequence`
- `type`
- `role`
- `content`
- `metadata`
- `created_at`

## Standard item types

Use standard OpenAI-like items where possible:

```text
message
```

Roles:

```text
user
assistant
system
```

Content types:

```text
input_text
output_text
```

## Atelier item types

Use namespaced types when Atelier must expose local orchestration semantics:

```text
atelier.input_request
atelier.input_response
atelier.thread_state
atelier.recovery_notice
atelier.debug_event
```

Earlier design notes used `atelier.approval_request` and `atelier.approval_response`. Those names are superseded by `atelier.input_request` and `atelier.input_response`, because Codex app-server prompts include approvals, MCP elicitations, and other human input requests.

Most user-facing output can still be rendered as `message` with `role = assistant`. The `atelier.*` types are for semantically meaningful UI behavior.

## API

### Retrieve a thread conversation

```http
GET /threads/{thread_id}?project=example-project
```

Response:

```json
{
  "id": "thread-example",
  "object": "conversation",
  "created_at": 1770000000,
  "metadata": {
    "project": "example-project",
    "title": "Example workstream"
  },
  "atelier": {
    "state": "idle"
  }
}
```

### Create items

```http
POST /threads/{thread_id}/items?project=example-project
```

Request:

```json
{
  "items": [
    {
      "type": "message",
      "role": "user",
      "content": [
        { "type": "input_text", "text": "approve" }
      ],
      "metadata": {
        "person": "alice",
        "source": "api"
      }
    }
  ]
}
```

Response:

```json
{
  "object": "list",
  "data": [
    {
      "id": "item-example",
      "object": "conversation.item",
      "sequence": 42,
      "type": "message",
      "role": "user",
      "content": [
        { "type": "input_text", "text": "approve" }
      ],
      "metadata": {
        "project": "example-project",
        "thread": "thread-example",
        "person": "alice"
      },
      "created_at": 1770000000
    }
  ],
  "first_id": "item-example",
  "last_id": "item-example",
  "has_more": false
}
```

### List items

```http
GET /threads/{thread_id}/items?project=example-project&after=41
```

Response:

```json
{
  "object": "list",
  "data": [],
  "first_id": null,
  "last_id": null,
  "has_more": false,
  "next_after": 41,
  "atelier": {
    "state": "working"
  }
}
```

The `after` cursor is a numeric sequence for local file-first efficiency. Item ids remain stable and can be used for retrieval/debugging.

Implemented status: the daemon currently supports `GET /threads/{thread_id}`, `POST /threads/{thread_id}/items`, and `GET /threads/{thread_id}/items`. Single-item retrieval remains a planned follow-up.

### Send a message for processing

```http
POST /threads/{thread_id}/messages?project=example-project
```

This endpoint appends a user message and runs the normal thread interaction decision path: answer pending input, surface project busy state, or start Codex work.

Preferred request:

```json
{
  "role": "user",
  "content": [
    { "type": "input_text", "text": "Update the README." }
  ],
  "metadata": {
    "person": "alice",
    "source": "api"
  }
}
```

Response:

```json
{
  "id": "item-example",
  "object": "conversation.item",
  "sequence": 43,
  "type": "message",
  "role": "user",
  "status": "started",
  "content": [
    { "type": "input_text", "text": "Update the README." }
  ],
  "metadata": {
    "project": "example-project",
    "thread": "thread-example",
    "person": "alice",
    "source": "api"
  },
  "debug": {
    "job_id": "job-example"
  },
  "created_at": 1770000000
}
```

Normal responses must not expose `job_id`, `job_dir`, `prompt_id`, or raw event names at top level. Debug identifiers belong under `debug` or `metadata.debug` only.

### Retrieve one item

```http
GET /threads/{thread_id}/items/{item_id}?project=example-project
```

## Input and approval flow

### 1. User asks for work

Inbound item:

```json
{
  "type": "message",
  "role": "user",
  "content": [
    { "type": "input_text", "text": "Update the README." }
  ],
  "metadata": { "person": "alice" }
}
```

### 2. Codex requests input or approval

Atelier stores the Codex app-server prompt internally and appends:
```json
{
  "type": "atelier.input_request",
  "role": "assistant",
  "content": [
    {
      "type": "output_text",
      "text": "Codex wants to edit README.md. Reply approve, decline, or cancel."
    }
  ],
  "metadata": {
    "choices": "approve,decline,cancel",
    "debug": {
      "job_id": "job-example",
      "prompt_id": "prompt-example"
    },
    "method": "item/fileChange/requestApproval"
  }
}
```

`pending.json` points to this item.

### 3. User replies to the same thread

Inbound item:

```json
{
  "type": "message",
  "role": "user",
  "content": [
    { "type": "input_text", "text": "approve" }
  ],
  "metadata": { "person": "alice" }
}
```

Atelier detects pending input state, validates the reply, writes the internal prompt response, and appends:

```json
{
  "type": "atelier.input_response",
  "role": "user",
  "content": [
    { "type": "input_text", "text": "approve" }
  ],
  "metadata": {
    "decision": "accept",
    "debug": {
      "prompt_id": "prompt-example"
    }
  }
}
```

No normal user-facing path requires the job id or prompt id.

## Gateway behavior

Gateway inbound messages map to `POST /threads/{thread_id}/messages` when the message should enter the managed Atelier/Codex interaction path, or `POST /threads/{thread_id}/items` when an adapter only needs to append an inert/audit item.

Gateway outbound delivery polls the thread item stream using a delivery cursor. Gateways render assistant messages, `atelier.input_request`, and relevant `atelier.thread_state` items to the external channel, and keep raw job/event names out of normal user-facing messages.

Telegram example:

```text
chat:1000:topic:77 -> project=example-project, thread=thread-example
from:2000 -> person=alice
```

Telegram inbound text becomes a user message item. Telegram outbound delivery renders assistant, input-request, thread-state, and final items from the same item stream.

## CLI behavior

Preferred:

```bash
atelier thread send example-project --thread thread-example --as alice "approve"
atelier thread follow example-project --thread thread-example
```

`thread follow` renders items by default:

```text
[1] alice: Update the README.
[2] assistant: Codex wants to edit README.md. Reply approve, decline, or cancel.
[3] alice: approve
[4] assistant: Approved. Continuing.
[5] assistant: Done.
```

Debug options may expose raw event/job metadata:

```bash
atelier thread follow example-project --thread thread-example --debug
```

## Migration notes

- Keep `/events/message` as a compatibility alias for item creation.
- Keep `/events` as a debug endpoint while introducing `/threads/{thread}/items`.
- Convert Codex app-server prompt records into input request items.
- Make gateway delivery consume items instead of raw events.
- Update docs so thread items are introduced before jobs/prompts.

## Related

- Decision: `docs/decisions/0012-openai-style-thread-items.md`
- Plan: `docs/plans/0007-openai-style-thread-items.md`
- GitHub issue: https://github.com/smarzola/atelier/issues/18
