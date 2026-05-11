# Decision 0012: Threads expose OpenAI-style conversation items

## Status

Accepted.

## Context

Atelier's current alpha implementation exposes too much of the runtime as the user interface: jobs, prompt ids, event kinds, worker recovery, and delivery internals. Those artifacts are useful for audit and debugging, but they are not how people naturally work in a project.

The intended product model is:

```text
project -> thread -> messages in and out
```

A person should keep sending messages to a project thread and receive messages from that same thread. Approval requests, user approvals, progress, final answers, stuck-work notices, and recovery actions must flow through that thread. Jobs and Codex prompt files remain internal artifacts linked from the thread when debugging is needed.

OpenAI's Conversations API already defines a durable conversation object with ordered items. Conversation items include messages and model/tool-related artifacts. This is close to Atelier's desired thread model and gives clients a familiar shape.

## Decision

Atelier threads are project-scoped, local OpenAI-style conversations.

The product-facing interface is a conversation item stream:

```text
AtelierThread == project-scoped conversation
ThreadItem    == OpenAI-style conversation item
```

All normal interaction with Atelier must use thread items:

- user messages;
- assistant messages;
- approval requests;
- approval responses;
- input requests;
- progress updates;
- final answers;
- stuck/recovery notices.

Jobs, prompts, Codex protocol logs, worker metadata, and raw events remain internal/debug artifacts. They may be referenced from item metadata, but they are not the primary UX.

## API shape

Preferred daemon API:

```http
GET  /threads/{thread_id}?project=<project>
POST /threads/{thread_id}/items?project=<project>
GET  /threads/{thread_id}/items?project=<project>&after=<cursor>
GET  /threads/{thread_id}/items/{item_id}?project=<project>
```

The item envelope follows OpenAI conventions where practical:

```json
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
```

Atelier-specific item types must use an `atelier.` namespace, for example:

```text
atelier.approval_request
atelier.approval_response
atelier.thread_state
atelier.recovery_notice
atelier.debug_event
```

Plain conversation should use standard-style message items:

```text
type = message
role = user | assistant | system
content[] = input_text | output_text
```

## Approval semantics

When Codex app-server emits an approval or input request, Atelier must append a thread item such as:

```json
{
  "object": "conversation.item",
  "type": "atelier.approval_request",
  "role": "assistant",
  "content": [
    {
      "type": "output_text",
      "text": "Codex wants to run: cargo test --workspace. Reply approve, decline, or cancel."
    }
  ],
  "metadata": {
    "choices": "approve,decline,cancel",
    "job_id": "job-internal",
    "prompt_id": "prompt-internal",
    "method": "item/commandExecution/requestApproval"
  }
}
```

The next inbound user message to the same thread is interpreted against pending thread state. A user reply such as `approve` should resolve the internal Codex prompt, append an acknowledgement item, and let the worker continue. The user should not need the job id or prompt id for ordinary approvals.

## Storage model

Project-local thread state should include a product-facing item log:

```text
.atelier/threads/<thread-id>/
  thread.toml
  items.jsonl
  pending.json
  delivery-cursors/
  events.jsonl          # debug/runtime compatibility
  codex-sessions.jsonl
```

`items.jsonl` is the primary stream for CLI, API, and gateways. `events.jsonl` may remain as a debug/runtime stream during migration.

Internal jobs stay under:

```text
.atelier/jobs/<job-id>/
```

Those jobs may link back to thread items in metadata.

## Compatibility

During migration:

- `POST /events/message` should become a compatibility alias for appending a user message item.
- `GET /events` should remain available as a debug/runtime endpoint.
- CLI `thread follow` should render conversation items by default and expose raw events only with an explicit debug flag or separate command.
- Prompt and job commands may remain as operator/debug tools, but docs must not present them as the normal user flow.

## Consequences

Positive:

- Atelier's UX becomes thread-centric instead of job-centric.
- Approvals work like normal replies in the thread.
- Gateways can send and receive through one item stream.
- The interface matches a known OpenAI API shape while remaining local and project-scoped.
- Jobs and prompts remain auditable without leaking into the primary UX.

Negative:

- Existing event-based code needs migration.
- The daemon must translate Codex app-server protocol into product-facing items.
- Gateway delivery cursors need to move from raw events to items.
- Tests and docs must distinguish product items from debug/runtime events.

## Related

- GitHub issue: https://github.com/smarzola/atelier/issues/18
- Supersedes the product-facing parts of `docs/plans/0006-thread-native-interaction.md` that expose raw thread events as the main interface.
- Builds on Decision 0003: threads remain the durable project workstream.
