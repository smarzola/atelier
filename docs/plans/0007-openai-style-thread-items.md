# OpenAI-Style Thread Items Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Replace job/event/prompt-centric user interaction with one OpenAI Conversations-style item interface for project threads.

**Architecture:** Add a project-local `thread_items` model where each Atelier thread is a conversation and each user/assistant/approval/recovery message is a conversation item. The daemon, CLI, and gateways send and receive these items. Jobs, prompts, raw events, and worker state remain internal/debug artifacts linked from item metadata.

**Tech Stack:** Rust workspace, `atelier-core`, `atelier-cli`, daemon HTTP API, Codex app-server, project-local `.atelier/threads/<thread-id>/items.jsonl`, existing job/prompt artifacts.

---

## Current product correction

Atelier must not require users to manage jobs, prompt ids, or raw event streams during normal project work.

The primary interface is:

```text
project -> thread -> conversation items
```

Every normal interaction is a message/item to a thread. Approvals are also items in the same thread stream.

## References

- Decision: `docs/decisions/0012-openai-style-thread-items.md`
- Design: `docs/thread-conversation-items.md`
- GitHub issue: https://github.com/smarzola/atelier/issues/18

## Target file model

```text
.atelier/threads/<thread-id>/
  thread.toml
  items.jsonl
  pending.json
  delivery-cursors/
  events.jsonl          # debug/compatibility
  codex-sessions.jsonl
```

## Target API

```http
GET  /threads/{thread_id}?project=<project>
POST /threads/{thread_id}/items?project=<project>
GET  /threads/{thread_id}/items?project=<project>&after=<sequence>
GET  /threads/{thread_id}/items/{item_id}?project=<project>
```

Compatibility endpoints:

```http
POST /events/message   # alias to item creation
GET  /events           # debug/runtime events
```

---

## Slice 1: Core thread item store

**Objective:** Add project-local OpenAI-style conversation item storage.

**Files:**

- Create: `crates/atelier-core/src/thread_items.rs`
- Modify: `crates/atelier-core/src/lib.rs`
- Create: `crates/atelier-core/tests/thread_items.rs`

**Step 1: Write failing tests**

Add tests for:

1. appending a user message item creates `.atelier/threads/<thread>/items.jsonl`;
2. sequence numbers are monotonic;
3. reading after sequence returns later items only;
4. returned objects use `object = "conversation.item"`, `type = "message"`, `role = "user"`, and content arrays.

Suggested command:

```bash
cargo test -p atelier-core thread_items -- --nocapture
```

Expected: fail because module does not exist.

**Step 2: Implement minimal item model**

Define:

```rust
pub struct ThreadItem {
    pub id: String,
    pub object: String,
    pub sequence: u64,
    pub item_type: String,
    pub role: String,
    pub content: Vec<ThreadItemContent>,
    pub metadata: serde_json::Map<String, serde_json::Value>,
    pub created_at: u64,
}

pub struct ThreadItemContent {
    pub content_type: String,
    pub text: String,
}
```

Use serde renames so JSON has:

```json
{ "type": "message", "content": [{ "type": "input_text" }] }
```

Add helpers:

```rust
append_thread_item(project_path, thread_id, item) -> ThreadItem
read_thread_items(project_path, thread_id, after_sequence) -> Vec<ThreadItem>
```

**Step 3: Verify**

```bash
cargo test -p atelier-core thread_items -- --nocapture
cargo test --workspace
```

**Step 4: Commit**

```bash
git add crates/atelier-core

git commit -m "feat: add thread conversation item store"
```

Push and wait for CI.

---

## Slice 2: Daemon item API

**Objective:** Expose OpenAI-style item create/list endpoints through the daemon.

**Files:**

- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests: `crates/atelier-cli/tests/cli_daemon.rs` or `cli_gateway_server.rs`
- Modify docs: `docs/thread-conversation-items.md` if details change

**Step 1: Write failing tests**

Test:

1. `POST /threads/<thread>/items?project=example-project` appends a user message item.
2. `GET /threads/<thread>/items?project=example-project&after=0` returns OpenAI-style list shape:
   - `object = "list"`
   - `data[]`
   - `first_id`
   - `last_id`
   - `has_more`
3. `GET /threads/<thread>?project=example-project` returns conversation object.

Expected: fail because endpoints do not exist.

**Step 2: Implement routes**

Add route parsing for:

```text
GET  /threads/<thread>
POST /threads/<thread>/items
GET  /threads/<thread>/items
```

Keep query parsing simple and consistent with existing daemon endpoints.

**Step 3: Verify**

```bash
cargo test -p atelier-cli daemon_thread_items --test cli_daemon -- --nocapture
cargo test --workspace
```

**Step 4: Commit, push, watch CI**

```bash
git add crates/atelier-cli docs/thread-conversation-items.md

git commit -m "feat: expose thread item API"

git push
```

---

## Slice 3: Route `/events/message` through item creation

**Objective:** Keep compatibility while making item creation the shared ingress primitive.

**Files:**

- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests: existing gateway message tests

**Step 1: Write failing test**

Update an existing `/events/message` test to assert that a user message item is appended to `items.jsonl` before work starts or queues.

Expected: fail because `/events/message` currently creates jobs/events but no item.

**Step 2: Implement**

`handle_gateway_message_event` should normalize to the same internal `ThreadItemCreate` path used by `POST /threads/<thread>/items`.

Response should be thread/item shaped, with debug job info optional:

```json
{
  "status": "accepted",
  "thread": "thread-example",
  "item_id": "item-example",
  "sequence": 1,
  "debug": { "job_id": "job-example" }
}
```

During compatibility, preserve existing fields if tests/users rely on them.

**Step 3: Verify and commit**

```bash
cargo test -p atelier-cli gateway_message_event --test cli_gateway_server -- --nocapture
cargo test --workspace
```

Commit:

```bash
git commit -m "feat: route gateway messages through thread items"
```

Push and wait for CI.

---

## Slice 4: Convert Codex prompts into approval request items

**Objective:** When Codex asks for approval/input, append a user-facing thread item and persist pending state.

**Files:**

- Modify: managed worker prompt handling in `crates/atelier-cli/src/main.rs`
- Modify/add tests under `crates/atelier-cli/tests/`
- Possibly add helpers in `atelier-core/src/thread_items.rs`

**Step 1: Write failing test**

Use fake Codex app-server that emits a prompt request. Assert:

- job prompt file is still created;
- thread `items.jsonl` contains `atelier.approval_request`;
- thread `pending.json` points to item id, job id, prompt id, and choices;
- `/threads/<thread>/items` returns the approval request item.

**Step 2: Implement**

When prompt is stored:

1. create internal prompt artifact as today;
2. append approval/input item to thread;
3. write `pending.json`.

**Step 3: Verify and commit**

```bash
cargo test --workspace
```

Commit:

```bash
git commit -m "feat: surface codex prompts as thread items"
```

Push and wait for CI.

---

## Slice 5: Resolve approval replies as normal thread messages

**Objective:** A user message to a thread resolves pending approval without prompt id or job id.

**Files:**

- Modify: thread item ingress logic
- Modify: CLI `thread send`
- Modify tests: `cli_thread.rs`, `cli_gateway_server.rs`, `cli_prompts.rs`

**Step 1: Write failing tests**

Cases:

1. Pending approval exists; `POST /threads/<thread>/items` with user text `approve` writes the internal prompt response and appends `atelier.approval_response`.
2. Pending approval exists; `/events/message` with text `approve` does the same.
3. Invalid approval reply appends an assistant message explaining valid choices and does not clear pending state.
4. `atelier thread send ... approve` uses the item path and does not require prompt id/job id.

**Step 2: Implement**

On inbound user message:

1. append user item;
2. if `pending.json` exists, interpret text against pending choices;
3. write prompt response;
4. clear pending;
5. append approval response item;
6. do not start a new job.

**Step 3: Verify and commit**

```bash
cargo test -p atelier-cli thread_send_approval -- --nocapture
cargo test --workspace
```

Commit:

```bash
git commit -m "feat: answer approvals through thread messages"
```

Push and wait for CI.

---

## Slice 6: Render item stream in CLI

**Objective:** Make `atelier thread follow` show conversation items by default instead of raw event kinds.

**Files:**

- Modify: `crates/atelier-cli/src/main.rs`
- Modify tests: `crates/atelier-cli/tests/cli_thread.rs`
- Modify docs: `docs/usage.md`

**Step 1: Write failing test**

Given items:

- user message;
- approval request;
- approval response;
- assistant final message;

`atelier thread follow` should print:

```text
[1] alice: update README
[2] assistant: Codex wants approval...
[3] alice: approve
[4] assistant: Approved. Continuing.
```

Raw `job_started`/`final_result` event names should not appear unless using debug mode.

**Step 2: Implement**

Change `thread follow` to read `items.jsonl` by default.

Optionally add:

```bash
atelier thread follow --debug-events
```

for existing event stream.

**Step 3: Verify and commit**

```bash
cargo test -p atelier-cli thread_follow --test cli_thread -- --nocapture
cargo test --workspace
```

Commit:

```bash
git commit -m "feat: render thread items in follow"
```

Push and wait for CI.

---

## Slice 7: Gateway delivery uses items

**Objective:** Gateway adapters deliver product-facing items, not raw runtime events.

**Files:**

- Modify: Telegram publishing/delivery code in `crates/atelier-cli/src/main.rs`
- Modify tests: `cli_daemon.rs` Telegram/fake Bot API tests
- Modify docs: `docs/usage.md`, `docs/thread-conversation-items.md`

**Step 1: Write failing tests**

1. Telegram receives assistant/approval/final item text from `items.jsonl`.
2. Delivery cursor advances by item sequence.
3. Restarting daemon does not redeliver already delivered items.
4. Raw debug events are not sent as user-facing Telegram messages.

**Step 2: Implement**

Move delivery cursor logic to item sequence.

Filter deliverable items:

- `message` with role `assistant`;
- `atelier.approval_request`;
- `atelier.approval_response`;
- `atelier.recovery_notice`.

Do not deliver user-originated items back to the same source unless a future feature explicitly wants echoing.

**Step 3: Verify and commit**

```bash
cargo test -p atelier-cli telegram --test cli_daemon -- --nocapture
cargo test --workspace
```

Commit:

```bash
git commit -m "feat: deliver gateway thread items"
```

Push and wait for CI.

---

## Slice 8: Thread-centric recovery items

**Objective:** Start migrating #10 from job commands to thread-level recovery messages.

**Files:**

- Modify: thread item ingress logic
- Modify/add tests under `crates/atelier-cli/tests/`
- Modify docs

**Step 1: Write failing tests**

Cases:

1. Thread has a stale/lost active job; `POST /threads/<thread>/items` with `continue` creates a recovery item and recovers/restarts internally.
2. Thread has live running work; `continue` appends assistant message saying work is already running.
3. Thread has no stuck work; `continue` behaves as normal user work message.

**Step 2: Implement minimal recovery intent**

Interpret these thread messages only when thread state is stuck:

```text
continue
retry
recover
cancel
```

Append user-facing recovery status items.

**Step 3: Verify and commit**

```bash
cargo test --workspace
```

Commit:

```bash
git commit -m "feat: handle recovery through thread messages"
```

Push and wait for CI.

---

## Slice 9: Documentation and issue migration

**Objective:** Make repository docs self-rehydrating for the new model and link #9, #10, #11 under #18.

**Files:**

- Modify: `README.md`
- Modify: `docs/usage.md`
- Modify: `docs/architecture.md`
- Modify: `docs/roadmap.md`
- Modify: old plan `docs/plans/0006-thread-native-interaction.md` to mark raw event-facing parts superseded by plan 0007.

**Step 1: Docs**

Docs must explain:

- thread items before jobs;
- jobs/prompts as internal/debug artifacts;
- approvals as thread messages;
- OpenAI-style item shape;
- gateway delivery from item stream.

**Step 2: GitHub issue comments**

Comment on #9, #10, #11 that they are now interpreted as slices under #18:

- #9: approvals via thread messages;
- #10: stuck/recovery via thread messages;
- #11: gateway delivery of thread items.

**Step 3: Verify and commit**

```bash
cargo fmt --check
cargo test --workspace
python -m unittest discover -s . -p 'tests_*.py' -v
./scripts/hygiene-scan.sh
git diff --check
```

Commit:

```bash
git commit -m "docs: adopt thread item interaction model"
```

Push and wait for CI.

---

## Final verification

Before closing #18:

1. Create a temporary project and thread.
2. Start daemon.
3. Send user item through `POST /threads/<thread>/items`.
4. Trigger fake Codex approval request.
5. Read approval request from `GET /threads/<thread>/items`.
6. Send `approve` as a normal user item.
7. Verify internal prompt response is written.
8. Verify final assistant item appears in item stream.
9. Verify Telegram/fake gateway delivers the item stream with cursor behavior.
10. Run full local gates.
11. Push and watch CI.
12. Close #18 with dogfood evidence and link related issues.

## Related issue mapping

- #9 becomes: approvals are answered as normal user items in a thread.
- #10 becomes: stuck work/recovery is handled by thread messages and recovery notice items.
- #11 becomes: gateway adapters deliver thread items and accept replies as thread items.

Do not close #9, #10, or #11 with job-centric fixes. Close them only once their thread-item slice has merged and CI is green.
