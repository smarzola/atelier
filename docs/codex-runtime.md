# Codex Runtime

Atelier uses Codex CLI as an external runtime. It does not bundle Codex initially.

## Local installation

Install Codex using an upstream-supported method:

```bash
npm i -g @openai/codex
# or
brew install --cask codex
```

Verify:

```bash
codex --version
codex --help
```

## Authentication

Codex supports ChatGPT login and OpenAI API key login.

Interactive login:

```bash
codex login
```

Device-code login, useful on headless machines:

```bash
codex login --device-auth
```

API key login, useful for programmatic environments:

```bash
printenv OPENAI_API_KEY | codex login --with-api-key
```

Codex caches login details in its own supported storage, such as `~/.codex/auth.json` or the operating system credential store. Treat those credentials like secrets.

## Atelier configuration

Atelier should discover Codex from `PATH` by default:

```toml
[codex]
binary = "codex"
minimum_version = "0.130.0"
```

A local override may point at a custom binary:

```bash
ATELIER_CODEX=/path/to/codex atelier work example-project "..."
```

Atelier creates a job directory for every real Codex run. The initial implementation invokes:

```bash
codex exec --cd <project> "<atelier-context>...<user-task>..."
```

and records:

- `request.md` — original user task;
- `context.md` — explicit Atelier context passed to Codex;
- `status.json` — current job status;
- `result.md` — Codex stdout;
- `stderr.log` — Codex stderr.

Atelier records Codex invocation metadata and exit code in `status.json`. Future versions should also record richer Codex metadata, such as Codex version and native Codex session identifiers when available.

## Managed runs and prompt relay

`codex exec` is appropriate for non-interactive one-shot work. It can emit JSON events with `--json`, but it is not the right substrate for gateway prompt relay because the gateway needs to answer Codex while the turn is running.

Managed Atelier jobs should use Codex's app-server protocol:

```bash
codex app-server
```

The app-server is a bidirectional JSON-RPC interface over stdio, websocket, or unix socket. Atelier should start with stdio. The client flow is:

1. send `initialize` with `experimentalApi: true`;
2. send `initialized`;
3. send `thread/start` or `thread/resume` with project `cwd`, approval policy, sandbox/permissions, and no model override unless explicitly requested;
4. send `turn/start` with the explicit Atelier context preamble plus the user's task;
5. persist raw JSON-RPC traffic and derived job state.

When Codex needs user input, it sends a server-initiated JSON-RPC request. Atelier must store this as a pending prompt and route it to the bound person/gateway. Important request methods include:

- `item/commandExecution/requestApproval`;
- `item/fileChange/requestApproval`;
- `item/permissions/requestApproval`;
- `item/tool/requestUserInput`;
- `mcpServer/elicitation/request`.

The gateway response becomes the JSON-RPC response to that exact request id. `serverRequest/resolved` and final item/turn notifications close the pending prompt.

Current managed work mode starts a background Atelier worker for each managed job. The worker owns one Codex app-server process, records `protocol.jsonl`, writes prompt records under `prompts/`, waits for response files under `responses/`, forwards those responses back to Codex, captures the final assistant message in `result.md`, and updates `status.json`. The launcher also records worker stdout/stderr as `worker-stdout.log` and `worker-stderr.log` so failed worker bootstraps are inspectable.

Useful commands:

```bash
atelier work <project> --thread <thread> --as <person> --managed "task"
atelier jobs list <project>
atelier jobs show <project> <job-id>
atelier prompts list <project>
atelier prompts show <project> <prompt-id>
atelier prompts respond <project> <prompt-id> accept
atelier prompts respond <project> <prompt-id> answer --text "example answer"
atelier prompts respond <project> <prompt-id> accept --json '{"decision":"accept"}'
atelier jobs recover <project> <job-id>
```

Managed workers support `--idle-timeout-seconds`. If a worker reaches idle timeout before a response or completion, it marks the job `idle-timeout`. `atelier jobs recover` restarts the managed worker from the saved job context. `atelier jobs list` reconciles `running` or `waiting-for-prompt` jobs with worker metadata and marks jobs `worker-lost` when their worker process is gone.

Atelier records managed app-server thread metadata in the Atelier thread's `codex-sessions.jsonl` file. That lineage stores the Codex app-server thread id, the job id, and the session path when Codex reports one. This keeps recovery and future resume UX grounded in Codex-native state rather than a parallel transcript store.

The default project concurrency policy is conservative single-writer: a new managed job refuses to start while another managed job in the same project is `running` or `waiting-for-prompt`. Parallel reads and future worktree-based write strategies can be added explicitly, but the default protects shared project folders from overlapping writes.

A terminal passthrough mode can remain useful for local human work, but it is not the managed prompt-relay architecture.

## Doctor checks

`atelier doctor` checks the local Codex runtime:

- Codex binary exists on `PATH`;
- `codex --version` succeeds;
- `codex exec --help` succeeds;
- `codex resume --help` succeeds.

`atelier doctor --project <path>` also checks the project scaffold:

- project root exists;
- `.atelier/project.toml` exists;
- `AGENTS.md` exists;
- `.atelier/threads/` exists.

Future versions may add explicit version-range checks and an opt-in live Codex smoke test.
Doctor must not silently write Codex config or credentials.

## CI testing strategy

Normal CI must not require live Codex authentication or spend model/API credits.

Default tests should use a fake Codex executable placed first on `PATH` or configured through `ATELIER_CODEX`.

The fake Codex should support enough behavior for deterministic tests:

```text
fake-codex --version
fake-codex exec ...
fake-codex exec resume --last ...
fake-codex exec resume <session-id> ...
fake-codex resume --last
```

It should record:

- argv;
- working directory;
- stdin when used;
- selected environment variables safe for tests.

It should return deterministic output, including a fake session id.

Tests should verify:

- command construction;
- project root selection;
- prompt/context injection;
- resume command construction;
- job metadata recording;
- error handling when Codex is missing;
- error handling when Codex exits non-zero.

## Optional real Codex integration test

A separate workflow may run real Codex only when secrets are configured and the workflow is manually triggered.

Example policy:

- not required for pull requests;
- not required for normal pushes;
- manual `workflow_dispatch` only;
- uses an OpenAI API key secret or preconfigured runner credentials;
- runs a tiny read-only smoke prompt in a disposable repository;
- records no private data.

Example smoke prompt:

```bash
codex exec --sandbox read-only -c approval_policy='"never"' \
  "Reply with exactly: ATELIER_CODEX_OK"
```

The smoke test should fail closed if authentication is missing, but it should not block the normal CI suite.
