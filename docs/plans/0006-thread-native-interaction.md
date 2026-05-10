# Thread-Native Interaction Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Resolve issue #12 by replacing the current job-start-oriented UX with one shared thread-native interaction model used by the CLI, local API, and gateway adapters.

**Architecture:** Introduce a small core interaction layer that accepts a normalized message addressed to a person/project/thread, decides whether to answer a pending prompt, queue/continue running work, resume an existing Codex session, or start a new job, and emits durable thread/job events. CLI, API, and Telegram should become thin surfaces over that same layer. Output delivery should read from the same project-local event stream, with per-surface delivery cursors for idempotency.

**Tech Stack:** Rust workspace, `atelier-core` file-first models, `atelier-cli` daemon/API/CLI commands, Codex app-server, project-local `.atelier/threads/` and `.atelier/jobs/` artifacts, existing JSON-over-HTTP daemon surface.

---

## Product correction

The current implementation treats most user input as:

```text
message -> create a job -> acknowledge the job id
```

The corrected model is:

```text
message -> resolve person/project/thread -> interact with the thread -> deliver events/results
```

A thread interaction must be independent of entrypoint. The same decision path must be used by:

- CLI input, for example `atelier thread send ...` or a later alias around `atelier work`;
- local API input, for example `POST /threads/message`;
- gateway adapter input, for example `POST /adapters/telegram/update`.

Telegram is an adapter, not the core model.

## Design principles

1. **Threads are the user-facing continuity object.** Jobs are implementation artifacts attached to a thread.
2. **One ingress primitive.** All surfaces normalize input into the same `ThreadMessage` request.
3. **One prompt-response primitive.** Prompt answers from CLI, API, or gateway replies resolve through the same code path.
4. **One event stream.** Codex output, job status, prompt events, and final results are recorded as durable thread/job events that every surface can read.
5. **File-first, restart-safe delivery.** Subscriber cursors live in project/global state so final messages and gateway notifications are not duplicated after daemon restart.
6. **No duplicate agent runtime.** Atelier still delegates reasoning/tool use to Codex; this plan only improves orchestration, routing, and delivery.
7. **Public hygiene.** Tests/docs use generic identities and fake ids only.

## Target file model

Add a thread-level event stream:

```text
example-project/
  .atelier/
    threads/
      thread-abc/
        events.jsonl
        delivery-cursors/
          cli-follow.json
          api-client-example.json
          telegram-chat-1000-topic-77.json
        active-job.json
        queued-messages.jsonl
        codex-sessions.jsonl
    jobs/
      job-xyz/
        status.json
        request.md
        context.md
        protocol.jsonl
        result.md
        prompts/
        responses/
        gateway-origin.json
```

`events.jsonl` is append-only. Each event has a stable id or monotonic sequence number.

Suggested event shape:

```json
{
  "sequence": 1,
  "timestamp_unix_seconds": 1770000000,
  "thread_id": "thread-example",
  "job_id": "job-example",
  "kind": "job_started",
  "payload": { "status": "running" }
}
```

Initial event kinds:

- `message_received`
- `prompt_required`
- `prompt_answered`
- `job_started`
- `job_status_changed`
- `agent_message_delta` or `agent_message_snapshot`
- `job_succeeded`
- `job_failed`
- `final_result`

Prefer `agent_message_snapshot` first if Codex app-server output does not provide reliable token deltas. Add true deltas only when tests prove they are stable.

## Core types

Add these to `atelier-core` unless implementation discovery shows a better module split:

```rust
pub struct ThreadMessage {
    pub project: String,
    pub project_path: PathBuf,
    pub thread_id: String,
    pub person: String,
    pub text: String,
    pub source: MessageSource,
}

pub enum MessageSource {
    Cli,
    Api { client_id: Option<String> },
    Gateway { gateway: String, external_thread: String, external_user: String },
}

pub enum ThreadInteractionDecision {
    AnswerPrompt { prompt_id: String },
    QueueForRunningJob { job_id: String },
    ContinueSession { codex_session_id: String },
    StartJob,
}

pub struct ThreadInteractionResult {
    pub status: String,
    pub thread_id: String,
    pub job_id: Option<String>,
    pub queued_message_id: Option<String>,
    pub prompt_id: Option<String>,
}
```

The exact fields can change, but the API must keep the same conceptual boundary: normalized input in, thread interaction result out.

---

## Slice 1: Document and test the durable thread event stream

**Objective:** Add the event stream model before changing runtime behavior.

**Files:**

- Modify: `crates/atelier-core/src/lib.rs`
- Create: `crates/atelier-core/src/thread_events.rs`
- Create or modify tests under `crates/atelier-core/tests/`
- Modify docs: `docs/architecture.md`, `docs/usage.md`

**Step 1: Write failing tests**

Add tests for:

- appending two events creates `.atelier/threads/<thread>/events.jsonl`;
- event sequence numbers are monotonic;
- reading from `after_sequence = 0` returns all events;
- reading from `after_sequence = 1` returns later events only.

**Step 2: Run focused tests**

```bash
cargo test -p atelier-core thread_events -- --nocapture
```

Expected: fail because the module does not exist.

**Step 3: Implement minimal event API**

Provide functions equivalent to:

```rust
append_thread_event(project_path, thread_id, kind, payload) -> ThreadEvent
read_thread_events(project_path, thread_id, after_sequence) -> Vec<ThreadEvent>
```

Use project-local files only.

**Step 4: Verify**

```bash
cargo test -p atelier-core thread_events -- --nocapture
cargo test --workspace
```

**Step 5: Docs and commit**

Document the event stream in architecture docs and commit:

```bash
git add crates/atelier-core docs
git commit -m "feat: add thread event stream"
```

---

## Slice 2: Record job lifecycle and final-result events from managed workers

**Objective:** Make existing worker behavior visible through the new thread event stream.

**Files:**

- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests: `crates/atelier-cli/tests/cli_daemon.rs`, `crates/atelier-cli/tests/cli_gateway_server.rs`

**Step 1: Write failing tests**

Add integration tests that start a fake or short managed job and assert events include:

- `job_started`;
- at least one `job_status_changed`;
- `final_result` when `result.md` is written or a successful completion occurs.

Where real Codex would make this brittle, use the existing test seams/fake worker patterns. If no seam exists, add one explicitly rather than shelling out to production Codex in tests.

**Step 2: Run focused tests**

```bash
cargo test -p atelier-cli --test cli_daemon thread_events -- --nocapture
```

Expected: fail because worker paths do not append events yet.

**Step 3: Append lifecycle events**

Update paths that currently write:

- `status.json`
- `result.md`
- prompt files

so they also append thread events. Keep the file writes as source artifacts; events are an index/delivery stream, not a replacement.

**Step 4: Verify**

```bash
cargo test -p atelier-cli --test cli_daemon thread_events -- --nocapture
cargo test --workspace
```

**Step 5: Commit**

```bash
git add crates/atelier-cli crates/atelier-core
git commit -m "feat: emit managed job thread events"
```

---

## Slice 3: Introduce the shared thread interaction service

**Objective:** Centralize the decision path currently duplicated across `/work`, `/events/message`, and adapter code.

**Files:**

- Create: `crates/atelier-core/src/thread_interaction.rs` or `crates/atelier-cli/src/thread_interaction.rs` depending on dependency boundaries
- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests: CLI/API/gateway tests

**Step 1: Write decision tests**

Test the decision function in isolation:

1. If the thread has a pending prompt, incoming text becomes `AnswerPrompt`.
2. If the thread has a running job and no prompt, incoming text becomes `QueueForRunningJob`.
3. If the thread is idle and has a recent Codex session lineage, incoming text becomes `ContinueSession`.
4. If the thread is idle without lineage, incoming text becomes `StartJob`.

Do not implement queuing/resume behavior fully in this slice; test only the decision output and minimal result wiring.

**Step 2: Run tests**

```bash
cargo test -p atelier-core thread_interaction -- --nocapture
```

or, if placed in CLI:

```bash
cargo test -p atelier-cli thread_interaction -- --nocapture
```

**Step 3: Implement minimal decision layer**

Use existing project files:

- prompt files under jobs;
- `status.json` for active jobs;
- thread `codex-sessions.jsonl` for session lineage.

**Step 4: Route existing entrypoints through it**

Update:

- `POST /work`;
- `POST /events/message`;
- `POST /adapters/telegram/update`;
- CLI `atelier work` submission path.

At the end of this slice, behavior may still start jobs in most cases, but it must do so through the shared interaction service.

**Step 5: Verify and commit**

```bash
cargo test --workspace
git add crates
git commit -m "feat: route work through thread interaction service"
```

---

## Slice 4: Add API event reading and polling

**Objective:** Give non-gateway clients a stable way to observe thread/job output before adding push streaming.

**Files:**

- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests: `crates/atelier-cli/tests/cli_daemon.rs`
- Modify docs: `docs/usage.md`

**Step 1: Write failing API tests**

Add tests for:

```text
GET /threads/<thread-id>/events?project=example&after=0
GET /jobs/<job-id>/events?project=example&after=0
```

If route parsing is too limited for path params, use query/body routes such as:

```text
GET /events?project=example&thread=thread-example&after=0
```

Prefer the simplest route compatible with the current hand-written HTTP server.

**Step 2: Implement polling endpoint**

Return JSON:

```json
{
  "events": [...],
  "last_sequence": 12
}
```

**Step 3: Verify and commit**

```bash
cargo test -p atelier-cli --test cli_daemon events_endpoint -- --nocapture
cargo test --workspace
git add crates docs
git commit -m "feat: expose thread events through daemon API"
```

---

## Slice 5: Add CLI thread send/follow UX

**Objective:** Make CLI dogfooding thread-native instead of repeated job-oriented commands.

**Files:**

- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests under `crates/atelier-cli/tests/`
- Modify docs: `docs/usage.md`, `README.md`

**Step 1: Add failing CLI tests**

Target UX:

```bash
atelier thread send example-project --thread thread-example --as alice "Continue from here"
atelier thread follow example-project --thread thread-example --after 0
```

Possible aliases can come later. Keep `atelier work` as a compatibility command that submits to the same thread interaction service.

Test that:

- `thread send` requires a daemon and posts to the shared API;
- `thread follow` reads thread events and prints final results/status changes;
- `atelier work` and `thread send` produce equivalent daemon requests.

**Step 2: Implement minimal CLI commands**

Do not remove `atelier work`. Implement `thread send` as the clearer thread-native command while keeping `work` as existing shorthand.

**Step 3: Verify and commit**

```bash
cargo test -p atelier-cli --test cli_thread -- --nocapture
cargo test --workspace
git add crates docs README.md
git commit -m "feat: add thread send and follow commands"
```

---

## Slice 6: Add shared prompt response routing

**Objective:** Let incoming messages answer pending prompts through the same primitive used by explicit prompt commands.

**Files:**

- Modify interaction service from Slice 3
- Modify prompt response handlers in `crates/atelier-cli/src/main.rs`
- Modify tests: prompt CLI/API/gateway tests

**Step 1: Write tests**

Cases:

1. Thread has a single pending prompt; `thread send` with an approval-shaped response records a prompt response instead of starting a new job.
2. API message to the same thread answers the prompt.
3. Telegram update in a bound topic answers the prompt through the same interaction service.
4. Ambiguous free text with multiple pending prompts returns an actionable error rather than guessing.

**Step 2: Implement minimal prompt-answer detection**

Start conservative:

- exact commands such as `approve`, `accept`, `yes`, `decline`, `deny`, `no`;
- if prompt type requires structured input, require explicit prompt command for now.

Do not ask Codex to infer approval intent in Atelier; this is orchestration, not agent reasoning.

**Step 3: Verify and commit**

```bash
cargo test --workspace
git add crates docs
git commit -m "feat: route prompt replies through thread messages"
```

---

## Slice 7: Implement queued messages for running jobs

**Objective:** Avoid spawning overlapping jobs when a user replies while a job is already running.

**Files:**

- Modify interaction service
- Add queue file handling under thread folder
- Modify worker completion path to drain or expose queued messages
- Tests under core/CLI

**Step 1: Write tests**

Cases:

- running job + incoming thread message appends to `queued-messages.jsonl`;
- result says message queued, not new job started;
- when current job completes, the next queued message starts or is marked ready to start according to chosen minimal policy.

**Step 2: Choose minimal drain policy**

For the first implementation, prefer safety:

- queue messages;
- after job completion, emit `queued_message_ready`;
- do not auto-start the next job until there is an explicit daemon supervisor loop that can do so safely.

A later slice can auto-drain.

**Step 3: Verify and commit**

```bash
cargo test --workspace
git add crates docs
git commit -m "feat: queue thread messages while jobs run"
```

---

## Slice 8: Add gateway/API/CLI delivery cursors

**Objective:** Make event delivery idempotent for every surface.

**Files:**

- Add delivery cursor helpers in `atelier-core`
- Modify CLI follow/API/gateway tests
- Modify docs

**Step 1: Write tests**

Test:

- reading events with a named subscriber returns only undelivered events;
- advancing a cursor persists it under `delivery-cursors/`;
- restarting/re-reading does not duplicate delivered events.

**Step 2: Implement cursor helpers**

Suggested API:

```rust
read_undelivered_events(project_path, thread_id, subscriber_id) -> Vec<ThreadEvent>
advance_delivery_cursor(project_path, thread_id, subscriber_id, sequence)
```

**Step 3: Wire CLI/API**

- `thread follow --subscriber cli-follow` can use cursor mode optionally.
- API can accept `subscriber_id` for durable cursor behavior, while still supporting stateless `after` polling.

**Step 4: Verify and commit**

```bash
cargo test --workspace
git add crates docs
git commit -m "feat: track thread event delivery cursors"
```

---

## Slice 9: Publish final results back to Telegram using the shared event stream

**Objective:** Solve the immediate Telegram-visible gap without making Telegram special in the core.

**Files:**

- Modify daemon/gateway adapter code in `crates/atelier-cli/src/main.rs`
- Modify Telegram tests in `crates/atelier-cli/tests/cli_daemon.rs`
- Modify docs: `docs/usage.md`, `docs/architecture.md`

**Step 1: Write failing test**

A Telegram-originated job should:

1. receive update;
2. start job and ack job id;
3. when `final_result` event appears, call fake Telegram Bot API `sendMessage` with same `chat_id` and `message_thread_id`;
4. not send the same final result twice after cursor advancement.

**Step 2: Persist gateway origin**

When a gateway starts or interacts with a thread, record enough target metadata to deliver later:

```json
{
  "gateway": "telegram",
  "chat_id": "1000",
  "message_thread_id": "77"
}
```

Keep this tied to the job or thread delivery cursor, not person memory.

**Step 3: Implement publisher loop**

Start simple:

- scan active/recent gateway-origin jobs periodically in daemon loop;
- read undelivered events;
- send `final_result` and prompt notifications only;
- leave high-frequency progress streaming for a later slice.

**Step 4: Verify and commit**

```bash
cargo test -p atelier-cli --test cli_daemon telegram -- --nocapture
cargo test --workspace
git add crates docs
git commit -m "feat: publish thread results to telegram"
```

---

## Slice 10: Add bounded progress streaming

**Objective:** Add progress delivery after final-result delivery is reliable.

**Files:**

- Worker event emission
- CLI follow formatting
- API polling/streaming docs
- Telegram publisher throttling

**Step 1: Define progress policy**

Do not send token-by-token messages to Telegram. Use bounded updates:

- prompt required immediately;
- final result always;
- queued-message notices when a running job finishes;
- coalesced progress snapshots rather than every intermediate `agent_message_snapshot`;
- adjacent duplicate text collapsed so a final result does not repeat an identical snapshot.

**Step 2: Add tests for throttling**

Use fake event streams and fake Telegram API; assert only expected messages are sent.

**Step 3: Implement progress publisher**

Read from thread events, coalesce, and deliver via each surface's formatter.

**Step 4: Verify and commit**

```bash
cargo test --workspace
python -m unittest discover -s . -p 'tests_*.py' -v
./scripts/hygiene-scan.sh
git diff --check
git add crates docs README.md
git commit -m "feat: stream bounded thread progress"
```

---

## Final verification for the whole correction

Run:

```bash
cargo fmt --check
cargo test --workspace
python -m unittest discover -s . -p 'tests_*.py' -v
./scripts/hygiene-scan.sh
git diff --check
```

Then push and watch CI:

```bash
git push
gh run list --limit 5
gh run watch <run-id> --exit-status
```

## Open design questions

1. Should `atelier work` remain the main public command, or should docs transition to `atelier thread send` with `work` as shorthand?
2. Should queued messages auto-drain after the current job succeeds, or require explicit confirmation in early alpha?
3. How much of Codex app-server message output is stable enough to expose as progress events versus only final snapshots?
4. Should API streaming use server-sent events later, or is polling enough for the alpha?
5. How should gateway prompt replies handle ambiguous human text beyond conservative `approve`/`decline` commands?

## Documentation updates required after implementation

- `README.md`: quickstart should introduce thread-native send/follow before adapter-specific gateway behavior.
- `docs/architecture.md`: show the corrected flow with thread interaction and event delivery.
- `docs/usage.md`: teach CLI/API/gateway as equivalent surfaces over the same thread model.
- `docs/codex-runtime.md`: explain how Codex session lineage is used for thread continuation without mutating Codex config.

## Issue closure criteria

Close #12 only when:

- CLI, API, and Telegram adapter all call the shared thread interaction path;
- thread events are persisted and readable;
- CLI can follow output;
- API can read or stream output;
- Telegram gets final results through the same event stream;
- prompt responses share one primitive;
- docs describe the unified model.
