# Architecture

Atelier is a project-native runtime around Codex CLI.

## System boundary

Atelier owns:

- project registry and discovery;
- identity resolution for gateways;
- person-memory retrieval;
- project routing;
- Codex process invocation;
- job lifecycle and notifications;
- resume affordances;
- file-first project runtime artifacts.

Codex owns:

- model interaction;
- autonomous reasoning;
- command execution;
- file edits;
- web search when enabled;
- MCP tool use;
- skill activation;
- session transcripts and native resume.

Atelier should not become a second model runtime unless a future decision record explicitly changes this boundary.

## High-level flow

Atelier work flows through the daemon. The daemon is the always-alive orchestration layer; gateways are hosted interfaces inside it, and CLI work-starting commands submit to it. Raw `cd project && codex` remains valid, but it is not an Atelier-managed run.

```text
Gateway / CLI / API
        |
        v
atelier daemon
        |
        v
 Incoming event
        |
        v
 Identity resolver -----> Person memory
        |
        v
 Project router --------> Project folder
        |
        v
 Job manager -----------> .atelier/jobs/<job-id>/
        |
        v
 Managed Codex worker --> codex app-server inside the project root
```

## Project folder contract

A project is a folder that can be used directly with Codex.

Recommended layout:

```text
example-project/
  AGENTS.md
  .agents/
    skills/
  .codex/
    config.toml
  .atelier/
    inbox/
    journal.md
    memory/
    threads/
    jobs/
    sessions/
    artifacts/
```

### Codex-native files

`AGENTS.md`, `.agents/skills`, and `.codex/config.toml` are owned by the project and interpreted by Codex. Atelier may help create or manage them through explicit commands, but it must not secretly rewrite them for runtime context injection.

### Atelier files

`.atelier/` stores project-local Atelier runtime state:

- `inbox/` — captured notes or requests that need later organization.
- `journal.md` — optional human-readable project log.
- `memory/` — project-scoped durable facts or summaries, never person preferences.
- `threads/` — one folder per ongoing Atelier workstream, with gateway bindings and Codex session lineage.
- `jobs/` — one folder per Atelier-launched job.
- `sessions/` — optional indexes or pointers to Codex sessions.
- `artifacts/` — generated files not naturally belonging elsewhere.

## Global state contract

Atelier's global state belongs outside projects, for example:

```text
~/.atelier/
  config.toml
  people/
    alice/
      memory.md
    bob/
      memory.md
  gateways/
  registry.toml
```

Global memory is person memory only. It is safe to inject into any project session for that person because it contains no project facts. The initial CLI stores person memory under `$ATELIER_HOME/people/<person>/memory.md`, falling back to `~/.atelier/people/<person>/memory.md` when `ATELIER_HOME` is unset. The global project registry is stored separately in `$ATELIER_HOME/registry.toml` and records project names and paths, not project knowledge.

## Runtime context injection

Atelier should not swap `CODEX_HOME` or rewrite Codex config as the default way to inject person context.

Preferred invocation model for non-interactive gateway work:

```bash
codex exec --cd /path/to/project "<atelier runtime context + user task>"
```

When a run needs choices such as approvals, sandbox, model, or web search, Atelier should pass them as explicit invocation-time Codex flags/config overrides rather than mutating `~/.codex/config.toml` or project `.codex/config.toml`:

```bash
atelier work /path/to/project \
  --thread thread-example \
  --as alice \
  --approval-policy on-request \
  --sandbox workspace-write \
  --search \
  "Do the next careful implementation step."
```

This currently maps to Codex-native arguments such as `-c approval_policy=\"on-request\"`, `--sandbox workspace-write`, and `--search`. Atelier should not set a model by default; omitting `--model` lets Codex select its configured/default model. A model override remains available only as an explicit per-run choice. The sandbox flag is only available for fresh `codex exec` runs; resume uses Codex's supported resume options and can still pass approval-policy and model overrides.

For local interactive use, a user can attach Codex directly to the current terminal:

```bash
atelier work /path/to/project \
  --thread thread-example \
  --as alice \
  --interactive \
  "Do a task that may require approval."
```

Interactive mode deliberately streams Codex stdout/stderr/stdin through the attached terminal so Codex prompts, approval requests, and other questions can be seen and answered by the human. This is the local CLI shape of the same capability a future gateway/daemon needs: surfaced pending prompts plus a response path back to the Codex process.

Where the prompt contains clearly delimited runtime context:

```markdown
<atelier-context>
Current person: Alice
Gateway: Telegram DM
Person memory:
- Alice prefers concise progress updates.
- Alice wants public examples to use generic names.

Boundary:
- This context is about the person only.
- Do not store project facts in person memory.
- Record durable project facts in project files.
</atelier-context>

<user-task>
Organize the inbox notes into the appropriate project documents.
</user-task>
```

For interactive local use, Atelier may provide:

```bash
atelier codex
```

This should be an explicit wrapper that launches Codex with an initial context message or another supported runtime mechanism. It should not silently mutate Codex home or project files.

## Thread and resume model

Atelier has its own durable thread model. An Atelier thread represents one ongoing workstream in a project or the home workspace. It is the user-facing continuity object that gateway topics, CLI sessions, and Codex sessions attach to.

Current design direction: an Atelier thread is a project-scoped OpenAI-style conversation. The product-facing interface is an ordered stream of conversation messages/items in `.atelier/threads/<thread-id>/items.jsonl`. Jobs, Codex prompts, protocol logs, and raw `events.jsonl` entries are internal/debug artifacts linked from item metadata. See `docs/decisions/0012-openai-style-thread-items.md`, `docs/decisions/0013-message-centric-openai-style-api.md`, `docs/thread-conversation-items.md`, `docs/design/message-centric-conversation-api.md`, and `docs/plans/0008-message-centric-conversation-api.md`.

Codex stores local transcripts and supports native resume commands:

```bash
codex resume
codex resume --last
codex resume --all
codex resume <SESSION_ID>
codex exec resume --last "Continue with the next step"
codex exec resume <SESSION_ID> "Continue with the next step"
```

Atelier should wrap these capabilities rather than reimplementing transcripts. The initial CLI supports `atelier continue <project> --thread <thread-id> --as <person> --last "prompt"` and `--session <session-id>`, implemented through `codex exec resume`. It stores Codex session IDs as a lineage attached to an Atelier thread when available.

Initial commands might be:

```bash
atelier threads list [--project <name>] [--home]
atelier thread new [--project <name>] "Release preparation"
atelier thread status [<thread-id>]
atelier sessions list [--thread <thread-id>] [--project <name>] [--home]
atelier resume [--thread <thread-id>] [--last | <session-id>]
atelier continue [--thread <thread-id>] [--last | <session-id>] "prompt"
```

Thread folders store bindings, summaries, events, jobs, and Codex session lineage:

```text
.atelier/threads/thread-abc/
  thread.toml
  summary.md
  gateway-bindings.toml
  events.jsonl
  delivery-cursors/
  codex-sessions.jsonl
  jobs/
```

`events.jsonl` is the durable thread event stream used by CLI, API, and gateway surfaces. It is append-only and sequence-numbered so clients can read from an `after` cursor and avoid replaying old output. Job folders keep source artifacts such as `status.json`, `protocol.jsonl`, and `result.md`; thread events are the shared delivery/index layer over those artifacts.

Atelier job folders should store pointers to Codex session IDs when available:

```text
.atelier/jobs/2026-01-01T120000Z-example/
  request.md
  context.md
  codex-session-id.txt
  status.json
  result.md
```

A project may have multiple Atelier threads in parallel, but write-capable jobs need a concurrency policy. The initial default is `single-writer` per project for work: parallel threads are allowed, but a new job refuses to start while another job in the same project is `running` or `waiting-for-prompt`. Jobs whose worker process disappeared are reconciled to `worker-lost` before the writer slot is evaluated. Future project configuration may opt into safer parallel strategies such as git worktrees.

## Gateway model

A gateway message is resolved into four layers:

1. person identity;
2. gateway thread/topic binding;
3. project or home target;
4. task or command.

Example:

```text
atelier thread send example-project --thread thread-abc --as alice "summarize the latest research notes"
```

Atelier resolves the sender to a person, loads only that person's memory, routes to `example-project`, resolves the Atelier thread, and submits the message through the shared thread interaction service. That service may answer a pending prompt, queue the message behind a running job, or create a new job folder and invoke Codex in the project root.

### Gateway thread bindings

Gateway threads are bindings to Atelier threads, not the core object.

For Telegram, the preferred binding is a forum topic or topic-enabled private chat thread using `message_thread_id`:

```text
telegram:<chat_id>:<message_thread_id> -> atelier-thread:<thread-id>
```

If native topics are unavailable or unreliable, the Telegram adapter may fall back to reply-root threads or synthetic command-selected threads. The same Atelier thread model should also support future platforms such as Discord, Slack, CLI, and API clients.

The initial CLI provides file-backed gateway binding primitives, while the always-alive daemon hosts the generic local HTTP gateway:

```bash
atelier gateway bind example-project --thread thread-abc --gateway example-gateway --external-thread external-thread
atelier gateway bind-person --gateway example-gateway --external-user external-user --person alice
atelier gateway resolve example-project --gateway example-gateway --external-thread external-thread
atelier daemon run --listen 127.0.0.1:8787
```

Thread bindings are stored in the Atelier thread folder's `gateway-bindings.toml` file. Person bindings are stored in global Atelier state because they describe external identities, not project knowledge. Platform adapters should build on this gateway-neutral binding layer rather than placing Telegram assumptions into the core thread model.

The daemon-hosted HTTP gateway exposes JSON endpoints:

- `GET /health`
- `GET /status`
- `GET /jobs`
- `GET /prompts`
- `GET /projects`
- `GET /events?project=<name>&thread=<thread-id>&after=<sequence>`
- `POST /projects`
- `POST /work`
- `POST /prompts/respond`
- `POST /events/message`
- `POST /adapters/telegram/update`
- `POST /adapters/telegram/webhook/setup`
- `POST /adapters/telegram/send-message`

The daemon listens on localhost by default and refuses non-loopback addresses unless explicitly allowed. `atelier gateway serve` remains available as a compatibility/developer command for the same HTTP surface, but product usage should run `atelier daemon run`.

The Telegram adapter uses `ATELIER_TELEGRAM_BOT_TOKEN` for outbound Bot API calls, defaults to `https://api.telegram.org`, and accepts `ATELIER_TELEGRAM_API_BASE` for local test servers or proxies. Webhook setup posts `url` from `ATELIER_TELEGRAM_WEBHOOK_URL` and includes `secret_token` when `ATELIER_TELEGRAM_WEBHOOK_SECRET` is set. Incoming Telegram updates validate `X-Telegram-Bot-Api-Secret-Token` against that secret before they are translated into the generic gateway message event. When an update starts a job, the adapter sends a Bot API acknowledgement to the originating chat/topic with the job id. Job output is delivered from the shared thread event stream using a bounded policy: prompt notifications and queued-message notices are deliverable immediately, agent-message snapshots are coalesced, adjacent duplicate text is collapsed, and the final result is always preserved. Delivery cursors prevent duplicate sends after restart.

A message resolves in this order:

1. explicit command target;
2. existing gateway thread binding;
3. chat-level default thread;
4. home fallback.

### Shared threads

In shared group threads, Atelier resolves the thread and the current speaker independently:

```text
chat/thread -> AtelierThread -> project/home
sender      -> Person        -> person memory
```

Person memory injection must be scoped to the current speaker. Atelier must not inject other participants' private person memory unless that memory is explicitly marked shared or the person has granted permission. Project state remains shared because it lives in the project folder.

## Tool model

Atelier should avoid native tools that duplicate Codex features.

- Web search: use Codex built-in search or MCP.
- External systems: use MCP.
- Reusable workflows: use Codex skills.
- Project instructions: use `AGENTS.md`.

Atelier provides management commands that write Codex-native configuration explicitly:

```bash
atelier mcp add project example-project context7 -- npx -y @upstash/context7-mcp
atelier skill add project example-project ./skills/triage-inbox
```

These commands create or update project-local Codex files such as `.codex/config.toml` and `.agents/skills/`. They are explicit setup operations, not hidden runtime mutation.

These are management operations, not separate execution paths.
