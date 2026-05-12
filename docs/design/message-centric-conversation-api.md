# Message-centric conversation API

Atelier's product surface is message-centric:

```text
project -> thread/conversation -> conversation items/messages
```

Jobs, prompts, raw events, worker metadata, protocol logs, queue files, and recovery internals remain file-first runtime artifacts, but they are not the normal user or gateway vocabulary.

## API principles

- Follow OpenAI Conversations/Responses-style shapes when practical.
- A thread is a conversation object.
- Conversation items are ordered by durable `sequence`.
- User input is represented as `message` items with `role = user` and `input_text` content.
- Assistant output is represented as `message` items with `role = assistant` and `output_text` content.
- Atelier-specific runtime needs use explicit item types such as `atelier.input_request`, `atelier.input_response`, `atelier.thread_state`, and `atelier.recovery_notice`.
- Normal responses expose item-facing fields (`object`, `id`, `sequence`, `status`, `metadata`) and keep job/prompt ids only in `debug` metadata.

## Conversation object

```http
GET /threads/{thread_id}?project=<project>
```

```json
{
  "id": "thread-example",
  "object": "conversation",
  "created_at": 0,
  "metadata": {
    "project": "example-project",
    "title": "Example workstream",
    "state": "idle"
  }
}
```

## Append/read items

```http
POST /threads/{thread_id}/items?project=<project>
GET  /threads/{thread_id}/items?project=<project>&after=<sequence>
```

Item append is for durable conversation item creation. It does not necessarily ask Codex to act.

```json
{
  "items": [
    {
      "type": "message",
      "role": "user",
      "content": [
        { "type": "input_text", "text": "Remember this project note." }
      ],
      "metadata": { "person": "alice", "source": "api" }
    }
  ]
}
```

Responses use the list envelope:

```json
{
  "object": "list",
  "data": [
    {
      "id": "item-example",
      "object": "conversation.item",
      "type": "message",
      "role": "user",
      "content": [
        { "type": "input_text", "text": "Remember this project note." }
      ],
      "metadata": { "person": "alice" },
      "sequence": 1
    }
  ],
  "first_id": "item-example",
  "last_id": "item-example",
  "has_more": false
}
```

## Send a message for processing

```http
POST /threads/{thread_id}/messages?project=<project>
```

This is a thread-native action endpoint layered over item append. It appends the user message and then runs the normal Atelier decision path: answer pending input, queue or surface state, or start/resume Codex work.

The preferred request shape is OpenAI-like:

```json
{
  "role": "user",
  "content": [
    { "type": "input_text", "text": "Do the next step." }
  ],
  "metadata": {
    "person": "alice",
    "source": "api"
  }
}
```

For CLI/simple clients, Atelier may also accept the shorthand shape:

```json
{
  "person": "alice",
  "text": "Do the next step."
}
```

The response is item-facing:

```json
{
  "object": "conversation.item",
  "id": "item-example",
  "sequence": 12,
  "status": "started",
  "metadata": {
    "project": "example-project",
    "thread": "thread-example",
    "person": "alice"
  },
  "debug": {
    "job_id": "job-example"
  }
}
```

Normal message responses must not expose top-level `job_id`, `job_dir`, `prompt_id`, or raw event names.

## Input requests and approvals

Codex approval/input prompts are normalized as conversation items:

- `atelier.input_request` from Atelier/assistant;
- `atelier.input_response` from the user/person.

The user should be able to reply in the same thread without knowing a prompt id:

```bash
atelier thread send example --thread "$THREAD" --as alice approve
```

Internal prompt response files remain debug artifacts. Structured Codex choices and ids are preserved under metadata/debug fields for adapters that need exact rendering.

## Busy and recovery state

Project concurrency and recovery should be expressed as items:

- `atelier.thread_state` for busy/queued/waiting/blocked state;
- `atelier.recovery_notice` for lost worker or recovery context.

A user message should be saved as a conversation item even when it cannot start a Codex turn immediately.

## Gateway model

Gateways are adapters over the same conversation item stream:

- inbound external messages become user `message` items;
- outbound delivery reads undelivered items by `sequence`;
- delivery cursors prevent duplicates;
- user-originated items are not echoed to the same gateway by default;
- raw event names and job/prompt ids are hidden unless explicitly debugging.
