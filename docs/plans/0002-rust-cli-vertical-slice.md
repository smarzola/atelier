# Rust CLI Vertical Slice Implementation Plan

**Goal:** Build the first executable Atelier vertical slice: initialize a project, create/list project threads, and construct dry-run Codex invocations without mutating Codex configuration.

**Architecture:** Use a Rust workspace with a small core library and CLI binary. `atelier-core` owns file-first project/thread/job models and Codex command construction. `atelier-cli` exposes commands through `clap`. The first slice uses dry-run Codex invocation only; real Codex execution comes after the file model and tests are stable.

**Tech Stack:** Rust 1.95, Cargo workspace, `clap`, `serde`, `toml`, `serde_json`, `anyhow`, `thiserror`, `uuid`, `time`, `camino`, `assert_cmd`, `predicates`, `tempfile`.

---

## Task 1: Bootstrap Rust workspace and CLI test harness

**Objective:** Create a compiling Rust workspace with CLI integration tests that initially fail because commands do not exist.

**Files:**

- Create: `Cargo.toml`
- Create: `crates/atelier-core/Cargo.toml`
- Create: `crates/atelier-core/src/lib.rs`
- Create: `crates/atelier-cli/Cargo.toml`
- Create: `crates/atelier-cli/src/main.rs`
- Create: `crates/atelier-cli/tests/cli_project.rs`
- Modify: `.gitignore`

**Step 1: Write failing CLI test**

Add a test that runs:

```bash
atelier --help
```

Expected output contains:

- `project`
- `thread`
- `work`

**Step 2: Run test to verify RED**

Run:

```bash
. "$HOME/.cargo/env"
cargo test -p atelier-cli cli_help_mentions_core_commands -- --nocapture
```

Expected: FAIL because workspace/CLI is not implemented yet.

**Step 3: Add minimal workspace and CLI**

Implement enough `clap` structure to show the command names.

**Step 4: Run test to verify GREEN**

Run:

```bash
. "$HOME/.cargo/env"
cargo test -p atelier-cli cli_help_mentions_core_commands -- --nocapture
```

Expected: PASS.

---

## Task 2: Implement project initialization

**Objective:** `atelier project init <path> --name <name>` creates a Codex-compatible project scaffold with `.atelier/` state.

**Files:**

- Create: `crates/atelier-core/src/project.rs`
- Modify: `crates/atelier-core/src/lib.rs`
- Modify: `crates/atelier-cli/src/main.rs`
- Modify: `crates/atelier-cli/tests/cli_project.rs`

**Step 1: Write failing test**

Test command:

```bash
atelier project init /tmp/example --name example-project
```

Assert files/directories exist:

```text
AGENTS.md
.atelier/project.toml
.atelier/inbox/
.atelier/threads/
.atelier/jobs/
.atelier/memory/
.atelier/artifacts/
```

Assert `project.toml` contains:

```toml
name = "example-project"
```

**Step 2: Run test to verify RED**

Expected: FAIL because `project init` is not implemented.

**Step 3: Implement minimal project init**

Rules:

- Create target directory if missing.
- Refuse to overwrite an existing `AGENTS.md` unless future `--force` exists.
- Write generic `AGENTS.md`; no private identifiers.
- Write project metadata to `.atelier/project.toml`.

**Step 4: Run test to verify GREEN**

Run specific test, then full test suite.

---

## Task 3: Implement thread creation and listing

**Objective:** `atelier thread new <project-path> "Title"` creates a durable Atelier thread; `atelier threads list <project-path>` lists it.

**Files:**

- Create: `crates/atelier-core/src/thread.rs`
- Modify: `crates/atelier-core/src/lib.rs`
- Modify: `crates/atelier-cli/src/main.rs`
- Create: `crates/atelier-cli/tests/cli_thread.rs`

**Step 1: Write failing test**

Flow:

```bash
atelier project init /tmp/example --name example-project
atelier thread new /tmp/example "Release preparation"
atelier threads list /tmp/example
```

Assert:

- `.atelier/threads/<thread-id>/thread.toml` exists.
- `summary.md` exists.
- `gateway-bindings.toml` exists.
- `codex-sessions.jsonl` exists.
- `jobs/` exists.
- list output contains `Release preparation`.

**Step 2: Run test to verify RED**

Expected: FAIL because thread commands are missing.

**Step 3: Implement minimal thread model**

Use UUID-based thread ids:

```text
thread-<uuid>
```

Write `thread.toml` with:

```toml
id = "thread-..."
title = "Release preparation"
status = "active"
```

**Step 4: Run test to verify GREEN**

Run specific test, then full test suite.

---

## Task 4: Implement dry-run Codex work command

**Objective:** `atelier work <project-path> --thread <thread-id> --as <person> --dry-run "Prompt"` writes a job folder and prints the Codex command that would run.

**Files:**

- Create: `crates/atelier-core/src/job.rs`
- Create: `crates/atelier-core/src/codex.rs`
- Modify: `crates/atelier-core/src/lib.rs`
- Modify: `crates/atelier-cli/src/main.rs`
- Create: `crates/atelier-cli/tests/cli_work.rs`

**Step 1: Write failing test**

Flow:

```bash
atelier project init /tmp/example --name example-project
THREAD_ID=$(atelier thread new /tmp/example "Design" --porcelain)
atelier work /tmp/example --thread "$THREAD_ID" --as alice --dry-run "Summarize the project"
```

Assert:

- output contains `codex exec`;
- output contains `--cd` and the project path;
- output contains `<atelier-context>`;
- output contains `Current person: alice`;
- output contains the user prompt;
- a `.atelier/jobs/<job-id>/` folder exists with `request.md`, `context.md`, `status.json`.

**Step 2: Run test to verify RED**

Expected: FAIL because `work` is not implemented.

**Step 3: Implement dry-run only**

Do not execute Codex yet. Build command representation and write job files.

**Step 4: Run test to verify GREEN**

Run specific test, then full test suite.

---

## Task 5: Add hygiene and CI skeleton

**Objective:** Add basic CI and public hygiene checks without requiring live Codex authentication.

**Files:**

- Create: `.github/workflows/ci.yml`
- Create: `scripts/hygiene-scan.sh`
- Modify: `README.md`

**Step 1: Write hygiene script**

Scan public docs/source for private identifiers and fail on matches.

**Step 2: Add CI workflow**

Run:

```bash
cargo test --workspace
./scripts/hygiene-scan.sh
```

**Step 3: Verify locally**

Run:

```bash
. "$HOME/.cargo/env"
cargo test --workspace
./scripts/hygiene-scan.sh
```

Expected: PASS.

---

## Verification checklist

- [ ] TDD RED/GREEN was followed for each behavior.
- [ ] `cargo test --workspace` passes.
- [ ] `scripts/hygiene-scan.sh` passes.
- [ ] No real personal names or private identifiers in public repo files.
- [ ] CLI scaffold preserves raw Codex equivalence and does not mutate Codex config.
- [ ] Dry-run command is inspectable and does not require live Codex auth.
