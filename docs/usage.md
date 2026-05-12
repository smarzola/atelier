# Atelier Usage Guide

This guide shows the current alpha workflow by walking through one small project. It also documents the CLI and daemon API surfaces you are likely to touch while dogfooding Atelier.

## Mental model

Atelier has three operating modes:

1. **Atelier setup** — home initialization, person memory, project creation, thread creation, and session listing can run from the CLI.
2. **Atelier thread work** — ordinary managed work is sent as messages into project threads and requires `atelier daemon run`.
3. **Raw Codex** — still possible in a project folder, but outside Atelier orchestration and not the primary Atelier user flow.

The daemon-first split is intentional. Atelier is an orchestrator. Managed work goes through the always-alive daemon so gateway messages, API calls, CLI submissions, approvals, recovery, and runtime state all share the same thread item path.

Key concepts:

- **Home workspace:** global Atelier state. It stores person memory, project registry, gateway person bindings, and gateway audit logs.
- **Person:** a human identity. Person memory is global and describes the person, not projects.
- **Project:** a durable working folder. Project-specific state belongs in the project folder under `.atelier/` and Codex-native files such as `AGENTS.md`, `.agents/skills`, and `.codex/config.toml`.
- **Thread:** one Atelier workstream inside a project. Gateway threads, Codex session lineage, and durable conversation items bind to Atelier threads.
- **Thread item:** the primary user-facing message stream in `.atelier/threads/<thread-id>/items.jsonl`. CLI, API, and gateways send and receive these OpenAI-style conversation items.
- **Job:** one internal Atelier-launched Codex run. Jobs live under `.atelier/jobs/` in the project and are debug/audit artifacts, not the normal interaction surface.
- **Thread event:** an internal/debug event stream in `.atelier/threads/<thread-id>/events.jsonl` for lifecycle and recovery diagnostics.

## Walkthrough: create a hello-world project

### Install Codex and Atelier

Codex is the only agentic worker Atelier uses:

```bash
npm i -g @openai/codex
codex login
```

Install Atelier from a GitHub Release archive. Release tags currently build only:

- `aarch64-apple-darwin` for macOS Apple Silicon;
- `aarch64-unknown-linux-gnu` for Linux ARM64;
- `x86_64-unknown-linux-gnu` for Linux x86_64.

Linux x86_64 archive example:

```bash
curl -L https://github.com/smarzola/atelier/releases/download/v0.1.0-alpha.1/atelier-x86_64-unknown-linux-gnu.tar.gz -o atelier.tar.gz
tar -xzf atelier.tar.gz
sudo install -m 0755 atelier-x86_64-unknown-linux-gnu/atelier /usr/local/bin/atelier
```

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

### Create a thread

```bash
THREAD=$(atelier thread new hello-world "Build a friendly greeting" --porcelain)
```

Threads are workstreams. You keep sending messages to the thread and reading conversation items back from it.

### Send the first real thread message from the CLI

```bash
atelier thread send hello-world \
  --thread "$THREAD" \
  --as alice \
  "Create HELLO.md with a friendly one-paragraph greeting for this project."
```

`atelier thread send` submits the message to the daemon-managed thread interaction path. If the project is busy, Atelier preserves the user message in the thread and surfaces state there instead of teaching job ids as the normal workflow.

The shared thread item stream is intentionally bounded. Atelier records user messages, assistant results, input requests, input replies, and project state as conversation items; it does not send token-by-token gateway spam.

Follow the conversation item stream:

```bash
atelier thread follow hello-world --thread "$THREAD" --after 0
```

If Codex asks for approval, reply in the same thread:

```bash
atelier thread send hello-world --thread "$THREAD" --as alice approve
```

Internal job, prompt, and raw-event artifacts are still inspectable as debug surfaces when needed:

```bash
atelier debug jobs list hello-world
atelier debug jobs show hello-world <job-id>
atelier debug prompts inbox
atelier debug events follow hello-world --thread "$THREAD"
```

When the work finishes:

```bash
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

### Send and read thread items through the API

Create a product-facing thread item:

```bash
curl -s "http://127.0.0.1:8787/threads/$THREAD/items?project=hello-world" \
  -H 'Content-Type: application/json' \
  -d '{"items":[{"type":"message","role":"user","content":[{"type":"input_text","text":"Hello through the item API"}],"metadata":{"person":"alice","source":"api"}}]}'
```

When you want Codex to act on a message, use the thread-native message endpoint. The preferred request shape is an OpenAI-like user message:

```bash
curl -s "http://127.0.0.1:8787/threads/$THREAD/messages?project=hello-world" \
  -H 'Content-Type: application/json' \
  -d '{"role":"user","content":[{"type":"input_text","text":"Append one more friendly sentence to HELLO.md."}],"metadata":{"person":"alice","source":"api"}}'
```

The response is the created `conversation.item` plus a `status` such as `started`, `input-answered`, or `blocked`. Normal responses do not expose `job_id`, `job_dir`, `prompt_id`, or raw event names at top level; debug identifiers live under nested `debug` fields. The shorthand shape remains available for simple clients:

```json
{"person":"alice","text":"Append one more friendly sentence to HELLO.md."}
```

Read the product-facing thread item stream with a stateless cursor:

```bash
curl -s "http://127.0.0.1:8787/threads/$THREAD/items?project=hello-world&after=0"
```

The response is OpenAI-style: `object = "list"`, `data[]`, `first_id`, `last_id`, and `has_more`. Use each item's numeric `sequence` as the next `after` value when polling from a CLI, local UI, or gateway publisher. Durable subscribers persist delivery cursors under `.atelier/threads/<thread-id>/delivery-cursors/` so daemon restarts do not duplicate delivered items.

Internal runtime endpoints for raw events, jobs, prompts, and compatibility message ingestion are debug/operator surfaces. They remain useful for audits and adapter development, but normal clients should build on project/thread items.

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

Gateway adapters route by external identifiers through the same project/thread item model used by CLI and local API clients.

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

Atelier maps Telegram thread ids to `chat:<chat-id>` or `chat:<chat-id>:topic:<topic-id>`. Telegram delivery reads the shared thread item stream, publishes user-facing `atelier.input_request`, `atelier.thread_state`, and final assistant output back to the same chat/topic, and advances delivery cursors to avoid duplicate sends. Internal job ids and raw event names are not part of normal Telegram delivery.

Send a Telegram message through the Bot API:

```bash
curl -s http://127.0.0.1:8787/adapters/telegram/send-message \
  -H 'Content-Type: application/json' \
  -d '{"chat_id":"1000","message_thread_id":"77","text":"Example notification"}'
```

## Dogfood verification

The thread-native flow was dogfooded against a temporary mock project with a fake Codex app-server on `PATH`. A direct `POST /threads/<thread>/items` call and an `atelier thread send` call produced one shared conversation item stream:

```text
1	message	user	hello item api
2	message	user	run dogfood task
3	message	assistant	dogfood done
```

This verifies that CLI send/follow, daemon item APIs, Codex app-server output translation, and project-local artifacts operate through the same thread item model. Normal `atelier thread send` output is item-facing (`Status`, `Item`, `Sequence`) and does not print internal job ids or job directories unless explicit debug output is requested. The message endpoint has also been exercised with the OpenAI-like `role`/`content`/`metadata` request shape and returns a `conversation.item` rather than a job-centric envelope.

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

Inspect or recover idle/lost internal jobs only from the debug surface:

```bash
atelier debug jobs recover hello-world <job-id>
atelier debug jobs recover hello-world --all-idle
atelier debug jobs recover hello-world --all-worker-lost
```

Writer-slot checks reconcile persisted worker metadata before refusing new work. If stale runtime state blocks a project, Atelier should surface recovery/busy state in the thread; debug job commands are operator escape hatches, not the normal product workflow.

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
