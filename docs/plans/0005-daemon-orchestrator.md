# Daemon Orchestrator Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Make `atelier daemon run` the required always-alive orchestration layer for Atelier work, with gateways hosted inside the daemon and jobs submitted to it instead of directly spawned by the CLI.

**Architecture:** Preserve file-first project/job state, but move process ownership into a long-lived daemon. The CLI becomes a client for work while remaining able to inspect files and initialize local state. The existing HTTP gateway implementation is reused as a daemon-hosted service, not a separate orchestration concept.

**Tech Stack:** Rust 2024, existing `atelier-core` and `atelier-cli`, standard library TCP HTTP server initially, file-backed job directories, Codex `app-server` workers.

---

## Task 1: Add daemon command surface

**Status:** Done in `7a0488c feat: add daemon runtime command`.

**Objective:** Introduce `atelier daemon run` as the user-facing long-lived runtime command without changing gateway behavior yet.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_daemon.rs`
- Docs: `README.md`, `docs/usage.md`

**Steps:**

1. Add a failing CLI test that starts `atelier daemon run --listen 127.0.0.1:<port>` and verifies `GET /health` returns `{"status":"ok"}`.
2. Add `Daemon { Run { listen, allow_non_loopback, auth_token_env } }` to the CLI command enum.
3. Route `daemon run` to the existing gateway server function with worker supervision enabled by default.
4. Keep `gateway serve` working for compatibility.
5. Update docs to teach `atelier daemon run` first and describe `gateway serve` as transitional/developer-facing.
6. Run:
   ```bash
   cargo fmt --check
   cargo test -p atelier-cli --test cli_daemon
   cargo test --workspace
   ```
7. Commit:
   ```bash
   git commit -am "feat: add daemon run command"
   ```

## Task 2: Extract daemon runtime options

**Status:** Done in `7a0488c feat: add daemon runtime command`.

**Objective:** Make the daemon a named runtime abstraction rather than a thin command alias.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Consider create: `crates/atelier-cli/src/daemon.rs`
- Test: `crates/atelier-cli/tests/cli_daemon.rs`

**Steps:**

1. Write a failing test that `daemon run` supervises stale worker state without passing `--supervise-workers`.
2. Extract a `DaemonOptions` struct containing listen address, auth token env, non-loopback opt-in, and supervision interval.
3. Make `daemon run` start worker supervision by default.
4. Keep `gateway serve --supervise-workers` behavior unchanged for compatibility.
5. Run targeted and workspace tests.
6. Commit:
   ```bash
   git commit -am "feat: make daemon supervise workers by default"
   ```

## Task 3: Add daemon submit endpoint for work

**Status:** Done in `157ffea feat: add daemon work submission endpoint`.

**Objective:** Create a daemon-owned API endpoint that starts work from a project/thread/person/task request.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_daemon.rs` or existing gateway tests
- Docs: `docs/usage.md`

**Steps:**

1. Add a failing HTTP test for `POST /work` with JSON:
   ```json
   {
     "project": "example-project",
     "thread": "thread-example",
     "person": "alice",
     "text": "Do the task"
   }
   ```
2. Verify the endpoint creates a job directory and starts a worker using the existing fake Codex binary test harness.
3. Implement the endpoint by reusing the existing gateway message/job creation path.
4. Return the created job id and job directory.
5. Audit-log `work_started`.
6. Run tests and commit:
   ```bash
   git commit -am "feat: add daemon work submission endpoint"
   ```

## Task 4: Make CLI work submit to daemon

**Status:** Done in `b4f964d feat: route work through daemon`.

**Objective:** Change `atelier work` so it requires a reachable daemon instead of directly spawning `atelier __managed-worker`.

**Files:**
- Modify: `crates/atelier-cli/src/main.rs`
- Test: `crates/atelier-cli/tests/cli_managed_work.rs` or equivalent
- Docs: `README.md`, `docs/usage.md`

**Steps:**

1. Add a failing test that `atelier work ...` exits with a clear error when no daemon is reachable.
2. Add a failing test that, when a daemon is running, `atelier work ...` submits to the daemon and receives a job id.
3. Add a daemon endpoint configuration option, defaulting to `http://127.0.0.1:8787`, overridable by CLI flag or environment variable.
4. Replace direct worker spawn in the managed CLI path with an HTTP submission to `/work`.
5. Keep `atelier __managed-worker` hidden and daemon-internal.
6. Update docs: Atelier work requires `atelier daemon run`.
7. Run tests and commit:
   ```bash
   git commit -am "feat: require daemon for work"
   ```

## Task 5: Preserve explicit local escape hatches

**Status:** Done in `b4f964d feat: route work through daemon` and the follow-up docs polish slice.

**Objective:** Keep raw Codex and non-managed inspection workflows clear without undermining daemon-required work.

**Files:**
- Modify: `docs/architecture.md`, `docs/usage.md`, `README.md`
- Modify tests if needed

**Steps:**

1. Document three modes:
   - raw Codex: `cd project && codex`, outside Atelier orchestration;
   - Atelier inspection/setup: CLI can read/write file-backed state without daemon where safe;
   - Atelier work: requires daemon.
2. Ensure `atelier work --dry-run` remains daemon-free because it creates no worker.
3. Ensure `atelier jobs list`, `atelier prompts inbox`, `atelier projects add`, and initialization commands remain daemon-free.
4. Run docs checks and commit:
   ```bash
   git commit -am "docs: clarify daemon-required work"
   ```

## Task 6: Deprecate gateway-as-runtime wording

**Status:** Done in the follow-up docs polish slice.

**Objective:** Remove product wording that suggests the gateway is the daemon.

**Files:**
- Modify: `README.md`, `docs/usage.md`, `docs/architecture.md`, `docs/roadmap.md`, relevant ADR follow-ups
- Test: optional documentation grep test

**Steps:**

1. Search docs for `gateway serve` and `daemon`.
2. Keep command references where accurate, but frame `gateway serve` as compatibility/developer-facing.
3. Add or update a lightweight documentation test if useful.
4. Run:
   ```bash
   ./scripts/hygiene-scan.sh
   git diff --check
   ```
5. Commit:
   ```bash
   git commit -am "docs: frame daemon as atelier runtime"
   ```

## Acceptance criteria

- `atelier daemon run` exists and hosts the current HTTP gateway endpoints.
- Worker supervision is daemon-owned and on by default for the daemon.
- `atelier work` requires a running daemon and no longer directly spawns a worker from the user-facing CLI path.
- Existing file-first job directories remain source of truth.
- Raw `cd project && codex` remains valid but documented as outside Atelier work.
- Initialization, registry, prompt inspection, job inspection, and dry-run commands remain usable without the daemon where safe.
- Docs and ADRs clearly say the daemon contains the gateway and is the required orchestration layer for Atelier work.
- Local verification passes:
  ```bash
  cargo fmt --check
  cargo test --workspace
  python -m unittest discover -s . -p 'tests_*.py' -v
  ./scripts/hygiene-scan.sh
  git diff --check
  ```
