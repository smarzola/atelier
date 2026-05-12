# Plan 0008: Message-centric conversation API

## Goal

Deliver issue #25 end-to-end: Atelier's normal API, CLI, docs, and gateway model should be message-centric and close to OpenAI conversation-style APIs.

## Slice 1: Design/docs baseline

- Add message-centric API design document.
- Add ADR for OpenAI-style conversation/message API direction.
- Link design from architecture, README, and usage docs.
- Keep debug/runtime artifacts documented only as debug/operator surfaces.

## Slice 2: Thread-native message request/response shape

- Accept OpenAI-like message request bodies on `POST /threads/{thread}/messages`:
  - `role`;
  - `content[]` parts;
  - `metadata.person`;
  - `metadata.source`.
- Preserve shorthand `{person, text}` for CLI/simple clients.
- Return an item-facing object, not a work/job response:
  - `object = conversation.item`;
  - `id`;
  - `sequence`;
  - `status`;
  - `metadata`;
  - optional `debug`.
- Keep job/prompt ids out of top-level normal responses.

## Slice 3: Generalize approvals to input request/response items

- Replace normal-path `atelier.approval_request` with `atelier.input_request`.
- Replace normal-path `atelier.approval_response` with `atelier.input_response`.
- Preserve structured Codex decisions and prompt/job ids under metadata/debug.
- Ensure a normal thread reply resolves pending input without prompt ids.
- Keep old item names only as historical/debug compatibility if needed.

## Slice 4: Busy/recovery as thread items

- When a message cannot start because the project/thread is busy, append `atelier.thread_state` to the same thread.
- When stale/lost worker recovery is relevant, append `atelier.recovery_notice`.
- Avoid normal response text that points users to `/jobs` or prompt commands.

## Slice 5: Gateway and docs finalization

- Verify gateway delivery consumes conversation items and renders input requests/thread state.
- Ensure normal docs and README use project/thread/message vocabulary.
- Move `/work`, `/events/message`, `/jobs`, `/prompts`, and `/prompts/respond` to debug/internal docs only or remove from public docs.
- Dogfood via Atelier itself.

## Verification

- RED/GREEN tests for each slice.
- `cargo fmt --check`.
- Rust test suite or relevant focused tests plus workspace tests before merge.
- `python -m unittest discover -s . -p 'tests_*.py' -v`.
- `./scripts/hygiene-scan.sh`.
- `git diff --check`.
- Push each completed slice and watch CI.
