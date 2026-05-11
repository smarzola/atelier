# Atelier Usage Guide

This guide shows the current alpha workflow by walking through one small project. It also documents the CLI and daemon API surfaces you are likely to touch while dogfooding Atelier.

## Mental model

Atelier has three operating modes:

1. **Atelier setup and inspection** — home initialization, person memory, project creation, thread creation, dry-runs, job listing, prompt inspection, and session listing can run from the CLI.
2. **Atelier work** — ordinary `atelier work` is daemon-managed and requires `atelier daemon run`.
3. **Raw Codex** — still possible in a project folder, but outside Atelier orchestration and not the primary Atelier user flow.

The daemon-first split is intentional. Atelier is an orchestrator. Managed work goes through the always-alive daemon so gateway messages, API calls, CLI submissions, prompt relay, recovery, and job state all share the same runtime path.

Key concepts:

- **Home workspace:** global Atelier state. It stores person memory, project registry, gateway person bindings, and gateway audit logs.
- **Person:** a human identity. Person memory is global and describes the person, not projects.
- **Project:** a durable working folder. Project-specific state belongs in the project folder under `.atelier/` and Codex-native files such as `AGENTS.md`, `.agents/skills`, and `.codex/config.toml`.
- **Thread:** one Atelier workstream inside a project. Gateway threads, Codex session lineage, and durable output events bind to Atelier threads.
- **Job:** one Atelier-launched Codex run. Jobs live under `.atelier/jobs/` in the project.
- **Thread event:** an append-only event in `.atelier/threads/<thread-id>/events.jsonl`. CLI, API, and gateways should read this shared event stream instead of inventing separate delivery state.

## Walkthrough: create a hello-world project

### Install Codex and Atelier

Codex is the only agentic worker Atelier uses:

```bash
npm i -g @openai/codex
codex login
```

Install Atelier from a GitHub Release archive when available. Release tags currently build only:

- `aarch64-apple-darwin` for macOS Apple Silicon;
- `aarch64-unknown-linux-gnu` for Linux ARM64;
- `x86_64-unknown-linux-gnu` for Linux x86_64.

To build from source:

```bash
git clone https://github.com/smarzola/atelier.git
cd atelier
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

### Create home and person state

```bash
atelier home init ~/atelier-home
atelier people add alice
atelier people memory set alice "Prefers short, practical examples."
```

`atelier home init` creates the global home workspace and registers the `home` alias. Person memory is global and injected into Atelier-launched work for that person. Do not store project facts there; put project facts in project files.

### Start the daemon

Open another terminal and run:

```bash
atelier daemon run --listen 127.0.0.1:8787
```

Keep it running. The daemon is the long-lived orchestrator. It hosts the local API, supervises workers, handles prompt relay, and is the surface gateway adapters use.

For adapter or reverse-proxy use, require bearer auth:

```bash
ATELIER_GATEWAY_TOKEN='replace-with-secret' atelier daemon run \
  --listen 127.0.0.1:8787 \
  --auth-token-env ATELIER_GATEWAY_TOKEN
```

The daemon refuses non-loopback addresses unless explicitly allowed:

```bash
atelier daemon run --listen 0.0.0.0:8787 --allow-non-loopback
```

### Create the project

```bash
mkdir -p ~/hello-world
atelier project init ~/hello-world --name hello-world
```

Project initialization creates the starter `AGENTS.md` and `.atelier/project.toml` files and registers the `hello-world` alias. You do not need a separate `atelier projects add` for a newly initialized project.

Add normal project content:

```bash
cat > ~/hello-world/README.md <<'EOF'
# Hello World

This is a tiny project for trying Atelier.
EOF

cat >> ~/hello-world/AGENTS.md <<'EOF'

## Project instructions

Keep outputs small and beginner friendly. Use files in this folder as source of truth.
EOF
```

### Create a thread and dry-run the first task

```bash
THREAD=$(atelier thread new hello-world "Build a friendly greeting" --porcelain)

atelier work hello-world \
  --thread "$THREAD" \
  --as alice \
  --dry-run \
  "Create a tiny hello-world note"
```

The dry-run does not require the daemon. It writes a dry-run job artifact and prints the Codex command plus explicit context injection, including the current person and thread.

### Send the first real thread message from the CLI

```bash
atelier thread send hello-world \
  --thread "$THREAD" \
  --as alice \
  "Create HELLO.md with a friendly one-paragraph greeting for this project."
```

`atelier thread send` submits the message to the daemon-managed thread interaction path. `atelier work` remains available as a compatibility shorthand for starting managed work, but the thread-native command is preferred for ongoing workstreams. If another job is already running in the project, the message is persisted to the thread's `queued-messages.jsonl` rather than starting an overlapping writer.

The shared thread event stream is intentionally bounded. Atelier records lifecycle events, prompt notifications, queued-message notifications, coalesced `agent_message_snapshot` progress, and `final_result`; it does not send token-by-token gateway spam.

Inspect the job and follow the shared thread event stream:

```bash
atelier jobs list hello-world
atelier jobs show hello-world <job-id>
atelier thread follow hello-world --thread "$THREAD" --after 0
```

If Codex asks for approval, the job becomes `waiting-for-prompt`. You can answer a single pending approval from the thread itself:

```bash
atelier thread send hello-world --thread "$THREAD" --as alice approve
```

Or use the explicit prompt commands when you need to inspect details or send structured input. For iterative approval flows, `respond-latest` avoids copying the newest prompt id from repeated list/show output:

```bash
atelier prompts inbox
atelier prompts show hello-world <prompt-id>
atelier prompts respond hello-world <prompt-id> accept
atelier prompts respond-latest hello-world <job-id> accept
```

During a live dogfood run, Codex asked for file-change approval and then created this file:

```text
HELLO.md
Hello and welcome to this hello-world project! This small space is here to make it easy to learn, try things out, and build confidence one simple step at a time.
```

When the job finishes:

```bash
atelier jobs list hello-world
atelier sessions hello-world --thread "$THREAD"
```

`sessions` shows the Codex session attached to the Atelier thread, so the runtime can resume the workstream later.

## Use the daemon API

The daemon API is the same orchestration surface gateways use.

### Health and status

```bash
curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/status
```

### Projects

List projects:

```bash
curl -s http://127.0.0.1:8787/projects
```

Create and register a project through the API:

```bash
curl -s http://127.0.0.1:8787/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"api-created","path":"/tmp/api-created"}'
```

That creates starter project files, registers the alias, and appends a gateway audit event.

`atelier projects add <name> <path>` still exists for adopting an already-existing folder or repairing/updating a registry alias. It is not part of the normal new-project setup path.

A running daemon reads the registry from disk on each request. Projects created by the CLI or API are visible without restarting the daemon.

### Start work through the API

```bash
curl -s http://127.0.0.1:8787/work \
  -H 'Content-Type: application/json' \
  -d "{\"project\":\"hello-world\",\"thread\":\"$THREAD\",\"person\":\"alice\",\"text\":\"Append one more friendly sentence to HELLO.md.\"}"
```

### Thread events, jobs, and prompts through the API

Read the shared thread event stream with a stateless cursor:

```bash
curl -s "http://127.0.0.1:8787/events?project=hello-world&thread=$THREAD&after=0"
```

The response contains `events` and `last_sequence`. Use `last_sequence` as the next `after` value when polling from a CLI, local UI, or gateway publisher. Durable subscribers can persist their own delivery cursor under `.atelier/threads/<thread-id>/delivery-cursors/` so daemon restarts do not duplicate delivered events.

Inspect current jobs and prompts:

```bash
curl -s http://127.0.0.1:8787/jobs
curl -s http://127.0.0.1:8787/prompts
```

Respond to an approval prompt:

```bash
curl -s http://127.0.0.1:8787/prompts/respond \
  -H 'Content-Type: application/json' \
  -d '{"project":"hello-world","prompt_id":"prompt-0","decision":"accept"}'
```

### Generic gateway message event

Gateway adapters can route external messages into project threads:

```bash
curl -s http://127.0.0.1:8787/events/message \
  -H 'Content-Type: application/json' \
  -d '{"gateway":"example-gateway","project":"hello-world","thread":"thread-example","person":"alice","text":"Run this task"}'
```

## Bind gateway identities

Bind an external thread to an Atelier thread:

```bash
atelier gateway bind hello-world \
  --thread "$THREAD" \
  --gateway example-gateway \
  --external-thread external-thread
```

Bind an external user to a person:

```bash
atelier gateway bind-person \
  --gateway example-gateway \
  --external-user external-user \
  --person alice
```

Then route by external identifiers:

```bash
curl -s http://127.0.0.1:8787/events/message \
  -H 'Content-Type: application/json' \
  -d '{"gateway":"example-gateway","external_thread":"external-thread","external_user":"external-user","text":"Run this task"}'
```

## Telegram adapter

Configure the daemon with a bot token, public webhook URL, and Telegram secret token:

```bash
export ATELIER_TELEGRAM_BOT_TOKEN='replace-with-bot-token'
export ATELIER_TELEGRAM_WEBHOOK_URL='https://example.invalid/atelier/telegram'
export ATELIER_TELEGRAM_WEBHOOK_SECRET='replace-with-secret-token'
atelier daemon run --listen 127.0.0.1:8787
```

By default, Atelier calls the real Telegram Bot API at `https://api.telegram.org`. Tests or local proxies can override that with `ATELIER_TELEGRAM_API_BASE`.

Register the webhook with Telegram:

```bash
curl -s http://127.0.0.1:8787/adapters/telegram/webhook/setup \
  -H 'Content-Type: application/json' \
  -d '{}'
```

Telegram update requests to `/adapters/telegram/update` must include `X-Telegram-Bot-Api-Secret-Token` matching `ATELIER_TELEGRAM_WEBHOOK_SECRET` when that variable is set.

Bind a Telegram topic-style thread and user:

```bash
atelier gateway bind hello-world \
  --thread "$THREAD" \
  --gateway telegram \
  --external-thread chat:1000:topic:77

atelier gateway bind-person \
  --gateway telegram \
  --external-user 2000 \
  --person alice
```

Send a Telegram update payload:

```bash
curl -s http://127.0.0.1:8787/adapters/telegram/update \
  -H 'Content-Type: application/json' \
  -H "X-Telegram-Bot-Api-Secret-Token: $ATELIER_TELEGRAM_WEBHOOK_SECRET" \
  -d '{"message":{"message_id":10,"message_thread_id":77,"chat":{"id":1000},"from":{"id":2000},"text":"Run this task"}}'
```

Atelier maps Telegram thread ids to `chat:<chat-id>` or `chat:<chat-id>:topic:<topic-id>`. When a Telegram update starts a job, Atelier acknowledges the update by sending a Bot API `sendMessage` back to the same chat/topic with the started job id. When the job completes, Atelier reads the shared thread event stream, coalesces progress to a bounded set of useful messages, publishes prompt/progress/final output back to the same Telegram chat/topic, and advances delivery cursors to avoid duplicate sends.

Send a Telegram message through the Bot API:

```bash
curl -s http://127.0.0.1:8787/adapters/telegram/send-message \
  -H 'Content-Type: application/json' \
  -d '{"chat_id":"1000","message_thread_id":"77","text":"Example notification"}'
```

## Dogfood verification

The thread-native flow was dogfooded against a temporary mock project with a fake Codex app-server on `PATH`. Two `atelier thread send` interactions in the same thread produced the shared CLI/API event stream (`job_started`, `agent_message_snapshot`, `final_result`, `job_succeeded`), recorded Codex session lineage, and wrote a project artifact. This verifies that CLI send/follow, daemon work submission, API event polling, Codex app-server lineage, and project-local artifacts all operate through the same thread model.

## Codex-native skills and MCP

Atelier does not build a separate tool ecosystem when Codex already has one. Use project-local Codex-native files.

Add a project-local Codex skill by copying a skill folder into `.agents/skills`:

```bash
atelier skill add project hello-world /path/to/skill-folder
```

Add a project-local MCP server to `.codex/config.toml`:

```bash
atelier mcp add project hello-world example-server -- command arg1 arg2
```

Atelier writes Codex-native capability files only when explicitly asked. It does not rewrite Codex home or project config merely to inject person context.

## Other useful commands

Register or update an existing project folder alias:

```bash
atelier projects add existing-project /path/to/existing-project
```

List registered projects:

```bash
atelier projects list
```

List threads:

```bash
atelier threads list hello-world
```

Show global runtime state from the CLI or daemon:

```bash
atelier status
curl -s http://127.0.0.1:8787/status
```

The daemon `/status` response includes a `daemon` object with the running Atelier executable path, version, and worker command. Check this after rebuilding during dogfood; if the path or version is not the binary you expect, restart the daemon before submitting more gateway work.

Continue a Codex session through Atelier:

```bash
atelier continue hello-world --thread "$THREAD" --as alice --last "Continue the previous task"
```

Recover idle or lost jobs:

```bash
atelier jobs recover hello-world <job-id>
atelier jobs recover hello-world --all-idle
atelier jobs recover hello-world --all-worker-lost
```

Writer-slot checks reconcile persisted worker metadata before refusing new work. If a previous `running` or `waiting-for-prompt` job has a dead worker process, the daemon marks it `worker-lost` and lets the next `/events/message` or `/work` request start normally instead of queueing behind a stale owner. If a live job still owns the writer slot, the error points you to `/jobs` or `atelier jobs list <project>` so you can inspect the active job before recovering or waiting.

## Audit logs

Gateway-originated actions append JSON Lines events to:

```text
~/.atelier/gateway/audit.jsonl
```

or, when `ATELIER_HOME` is set:

```text
$ATELIER_HOME/gateway/audit.jsonl
```

Audit events currently cover project creation, prompt responses, and message-start actions.
