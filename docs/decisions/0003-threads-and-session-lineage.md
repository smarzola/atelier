# Decision 0003: Threads bind gateways to Codex session lineage

## Status

Accepted for initial design.

## Context

Atelier needs to support multiple parallel workstreams in the same project, home workspace, Telegram DM, or group chat. Binding a whole gateway chat directly to one Codex session is too coarse.

For example, one project may need simultaneous threads for design, implementation, debugging, release preparation, and documentation. A shared group may also contain multiple people participating in the same project thread.

Telegram supports forum topics through `message_thread_id` in capable chats. Other platforms have different thread models, and some chats may not support native threads at all. Atelier needs a platform-neutral abstraction that can use native threads when available but still work without them.

## Decision

Atelier introduces a first-class `AtelierThread` concept.

An Atelier thread:

- belongs to a project or the home workspace;
- represents one ongoing workstream;
- may be bound to one or more gateway threads/topics;
- stores a lineage of Codex sessions rather than assuming exactly one Codex session forever;
- owns jobs and summaries for that workstream.

Gateway platform threads are bindings, not the core object.

```text
Telegram topic/reply/synthetic thread
        |
        v
GatewayThreadBinding
        |
        v
AtelierThread
        |
        v
CodexSessionLineage
```

## Binding model

A gateway message resolves in this order:

1. explicit command target;
2. existing gateway thread binding;
3. chat-level default thread;
4. home fallback.

For Telegram, the best binding key is:

```text
telegram:<chat_id>:<message_thread_id>
```

when `message_thread_id` is present. If native topics are not available, a Telegram adapter may use reply-root messages or synthetic command-selected threads.

## Storage model

Project threads are stored in the project folder:

```text
example-project/
  .atelier/
    threads/
      thread-abc/
        thread.toml
        summary.md
        codex-sessions.jsonl
        gateway-bindings.toml
        jobs/
```

Home threads are stored under the home workspace using the same layout.

`thread.toml` records human-facing thread identity:

```toml
id = "thread-abc"
title = "Release preparation"
project = "example-project"
status = "active"
created_by = "person:alice"
created_at = "2026-01-01T12:00:00Z"

[active_codex]
session_id = "7f9f9a2e-1b3c-4c7a-9b0e-example"
```

`gateway-bindings.toml` records platform bindings:

```toml
[[bindings]]
gateway = "telegram"
chat_id = "example-chat"
message_thread_id = "42"
```

`codex-sessions.jsonl` records the Codex session lineage for the thread. A thread may have more than one Codex session over time.

## Telegram topics

Telegram forum topics are the preferred UX when available. Atelier may create or bind topics such as:

- `Home`;
- `Example Project`;
- `Example Project / Design`;
- `Example Project / Implementation`.

The adapter should treat native topics as an optimization over the core thread model. If Telegram topics are unavailable or unreliable in a particular chat type, Atelier should fall back to reply-root or synthetic threads without changing the core data model.

## Multi-person shared threads

In a shared thread, each message has both a thread binding and a sender identity.

Atelier resolves:

```text
chat/thread -> AtelierThread -> project/home
sender      -> Person        -> person memory
```

Person memory injection must be scoped to the current speaker. Atelier must not inject other participants' private person memory unless that memory is explicitly marked shared or the person has granted permission.

Project state remains shared because it lives in the project folder.

## Concurrency policy

Multiple Atelier threads may exist for a project, but concurrent write-capable Codex jobs can conflict when they edit the same files.

The initial default is `single-writer` per project:

- multiple threads may exist;
- read-only/background analysis jobs may run in parallel when safe;
- write-capable jobs for the same project are serialized or queued.

Future project policies may allow:

- `git-worktree` for software projects, where each write-capable thread runs in a separate worktree;
- `unsafe-parallel` for users who explicitly accept conflict risk.

## Consequences

Positive:

- Parallel workstreams become natural.
- Telegram topics can map cleanly to Atelier threads.
- The same model can support Discord, Slack, CLI, API, and other platforms later.
- Codex sessions remain an implementation detail behind Atelier's durable thread model.
- Project-local thread state stays inspectable.

Negative:

- Atelier must maintain a thread index and binding resolver.
- Session resume requires mapping from Atelier threads to Codex session IDs where available.
- Concurrency requires conservative defaults to avoid file conflicts.

## Revisit when

- Codex exposes richer programmatic session metadata or thread APIs.
- Telegram private-chat topic behavior stabilizes or changes.
- A second gateway platform forces changes to the binding abstraction.
- Git worktree support becomes necessary for practical parallel coding work.
