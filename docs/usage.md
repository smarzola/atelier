# Atelier Usage Guide

This guide shows the current alpha workflow by walking through one small project. It also documents the CLI and daemon API surfaces you are likely to touch while dogfooding Atelier.

## Mental model

Atelier has three operating modes:

1. **Raw Codex** — `cd project && codex`. This is outside Atelier orchestration and should always remain valid.
2. **Atelier setup and inspection** — initialization, registry, person memory, thread creation, dry-runs, job listing, prompt inspection, and session listing can run from the CLI without the daemon.
3. **Atelier work** — ordinary `atelier work` is daemon-managed and requires `atelier daemon run`.

The split is intentional. Atelier is an orchestrator. Managed work goes through the always-alive daemon so gateway messages, API calls, CLI submissions, prompt relay, recovery, and job state all share the same runtime path.

Key concepts:

- **Home workspace:** global Atelier state. It stores person memory, project registry, gateway person bindings, and gateway audit logs.
- **Project:** a durable working folder. Project-specific state belongs in the project folder under `.atelier/` and Codex-native files such as `AGENTS.md`, `.agents/skills`, and `.codex/config.toml`.
- **Person:** a human identity. Person memory is global and describes the person, not projects.
- **Thread:** one Atelier workstream inside a project. Gateway threads and Codex session lineage bind to Atelier threads.
- **Job:** one Atelier-launched Codex run. Jobs live under `.atelier/jobs/` in the project.

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

### Create home and project folders

```bash
atelier home init ~/atelier-home
mkdir -p ~/hello-world
atelier project init ~/hello-world --name hello-world
atelier projects add hello-world ~/hello-world
```

`atelier home init` creates the global home workspace and registers the `home` alias. `atelier project init` creates starter `AGENTS.md` and `.atelier/project.toml` files in the project.

Now add normal project content:

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

You can still use raw Codex directly in the folder:

```bash
cd ~/hello-world
codex
```

Raw Codex sees the project instructions and files. Atelier-managed Codex additionally receives person context, runs as a tracked job, exposes prompts through the daemon, and records session lineage.

### Add a person

```bash
atelier people add alice
atelier people memory set alice "Prefers short, practical examples."
```

Person memory is global and injected into Atelier-launched work for that person. Do not store project facts there; put project facts in project files.

### Create a thread and dry-run the first task

```bash
THREAD=$(atelier thread new hello-world "Build a friendly greeting" --porcelain)

atelier work hello-world \
  --thread "$THREAD" \
  --as alice \
  --dry-run \
  "Create a tiny hello-world note"
```

The dry-run does not require a daemon. It writes a dry-run job artifact and prints the Codex command plus explicit context injection, including the current person and thread.

### Start the daemon

Open another terminal and run:

```bash
atelier daemon run --listen 127.0.0.1:8787
```

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

### Run the first real task from the CLI

Back in the original terminal:

```bash
atelier work hello-world \
  --thread "$THREAD" \
  --as alice \
  "Create HELLO.md with a friendly one-paragraph greeting for this project."
```

Atelier submits the job to the daemon. Inspect it:

```bash
atelier jobs list hello-world
atelier jobs show hello-world <job-id>
```

If Codex asks for approval, the job becomes `waiting-for-prompt`:

```bash
atelier prompts inbox
atelier prompts show hello-world <prompt-id>
atelier prompts respond hello-world <prompt-id> accept
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

A running daemon reads the registry from disk on each request. Projects added with `atelier projects add` or `POST /projects` are visible without restarting the daemon.

### Start work through the API

```bash
curl -s http://127.0.0.1:8787/work \
  -H 'Content-Type: application/json' \
  -d "{\"project\":\"hello-world\",\"thread\":\"$THREAD\",\"person\":\"alice\",\"text\":\"Append one more friendly sentence to HELLO.md.\"}"
```

### Jobs and prompts through the API

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
  -d '{"message":{"message_id":10,"message_thread_id":77,"chat":{"id":1000},"from":{"id":2000},"text":"Run this task"}}'
```

Atelier maps Telegram thread ids to `chat:<chat-id>` or `chat:<chat-id>:topic:<topic-id>`.

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

List registered projects:

```bash
atelier projects list
```

List threads:

```bash
atelier threads list hello-world
```

Show global runtime state:

```bash
atelier status
```

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
