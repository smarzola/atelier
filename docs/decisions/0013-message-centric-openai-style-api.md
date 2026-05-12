# ADR 0013: Message-centric OpenAI-style conversation API

## Status

Accepted

## Context

Atelier started with visible job, prompt, and event concepts because those were necessary to make Codex execution durable and inspectable. Dogfooding #21 showed that these runtime details still leak into the product model too easily.

The intended product model is:

```text
project -> thread/conversation -> messages/items
```

OpenAI's conversation-style API shapes are a good fit for this model: stable conversation objects, ordered conversation items, list envelopes, typed content parts, and metadata/debug side channels.

## Decision

Atelier will treat project threads as conversation objects and conversation items/messages as the primary API and UX surface.

Public APIs and CLI output should be message/item centric:

- `GET /threads/{thread_id}?project=<project>` returns a conversation-like object.
- `GET /threads/{thread_id}/items?project=<project>&after=<sequence>` returns an OpenAI-style list envelope.
- `POST /threads/{thread_id}/items?project=<project>` appends durable conversation items.
- `POST /threads/{thread_id}/messages?project=<project>` appends user input and runs Atelier's message-processing path.

Runtime internals remain durable but explicit debug/operator surfaces:

- jobs;
- prompts;
- raw events;
- worker metadata;
- Codex protocol logs.

Normal responses must not expose top-level `job_id`, `job_dir`, `prompt_id`, or raw event names. Those values may remain in `debug` metadata and file artifacts.

## Item taxonomy

Use OpenAI-style `message` items where possible:

- `message` with `role = user` and `input_text` content;
- `message` with `role = assistant` and `output_text` content.

Use Atelier-specific item types when the concept is not a plain message:

- `atelier.input_request`;
- `atelier.input_response`;
- `atelier.thread_state`;
- `atelier.recovery_notice`.

Previous `atelier.approval_request` / `atelier.approval_response` items should migrate to the generalized input request/response names.

## Consequences

- CLI and gateway implementations should read/write the same item stream.
- Gateway delivery cursors advance by item sequence, not raw event sequence.
- Approvals and recovery must be understandable from the thread, without debug prompt/job commands.
- Debug commands remain available, but normal docs should teach project/thread/message flows first.

## Non-goals

- Removing internal job/prompt/event artifacts entirely.
- Hiding all debug data from operators.
- Implementing an OpenAI-compatible HTTP server byte-for-byte. Atelier should follow the shape and vocabulary where useful, while preserving project-native requirements.
