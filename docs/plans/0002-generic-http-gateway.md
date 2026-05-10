# Generic HTTP Gateway Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build the first end-to-end generic HTTP gateway for Atelier so external systems can route messages, inspect runtime state, and answer prompts without platform-specific assumptions.

**Architecture:** The gateway is a local HTTP server exposed by `atelier gateway serve`. It uses generic JSON event/request types, existing project registry and thread binding files, person memory, managed work jobs, prompt response files, and status/job listing helpers. Platform adapters such as Telegram can be layered on top later.

**Tech Stack:** Rust 2024, standard library TCP HTTP server for the initial local gateway, `serde`/`serde_json`, existing Atelier core/CLI modules, fake Codex in tests.

---

## Slice 1: Gateway core event and identity model

**Objective:** Add project-neutral gateway event/request/response structs and file-backed external-user-to-person bindings.

**Files:**
- Modify: `crates/atelier-core/src/gateway.rs`
- Test: `crates/atelier-core/tests/gateway_identity.rs`

**Verification:**

```bash
cargo test -p atelier-core --test gateway_identity
```

## Slice 2: Gateway HTTP server skeleton

**Objective:** Add `atelier gateway serve --listen <addr>` with `GET /health` returning JSON.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_gateway_server.rs`

**Verification:**

```bash
cargo test -p atelier-cli --test cli_gateway_server health_endpoint_returns_ok
```

## Slice 3: Runtime inspection endpoints

**Objective:** Add `GET /status`, `GET /jobs`, and `GET /prompts` using the same runtime data as CLI status/jobs/prompts inbox.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_gateway_server.rs`

**Verification:**

```bash
cargo test -p atelier-cli --test cli_gateway_server status_jobs_and_prompts_endpoints_return_runtime_state
```

## Slice 4: Prompt response endpoint

**Objective:** Add `POST /prompts/respond` that writes the same durable response object as `atelier prompts respond`.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_gateway_server.rs`

**Verification:**

```bash
cargo test -p atelier-cli --test cli_gateway_server prompt_response_endpoint_records_response
```

## Slice 5: Message event routing endpoint

**Objective:** Add `POST /events/message` that resolves project alias, thread, person, and text into a managed Atelier work job. The initial endpoint should accept explicit `project`, `thread`, and `person` fields and can later grow gateway binding resolution.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_gateway_server.rs`

**Verification:**

```bash
cargo test -p atelier-cli --test cli_gateway_server message_event_starts_managed_work
```

## Slice 6: Documentation and issue cleanup

**Objective:** Document the generic gateway, update roadmap, and create/close GitHub issues for defects discovered during dogfooding.

**Files:**
- Modify: `README.md`
- Modify: `docs/codex-runtime.md`
- Modify: `docs/architecture.md`
- Modify: `docs/roadmap.md`

**Verification:**

```bash
cargo fmt --check
cargo test --workspace
python -m unittest discover -s . -p 'tests_*.py' -v
./scripts/hygiene-scan.sh
git diff --check
```
