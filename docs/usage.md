# Atelier Usage Guide

This guide documents the current alpha CLI and local gateway workflows.

## Concepts

- **Home workspace:** the global Atelier workspace. It stores person memory, project registry, gateway person bindings, and gateway audit logs.
- **Project:** a durable working folder. Project-specific state belongs in the project folder under `.atelier/` and Codex-native files such as `AGENTS.md`, `.agents/skills`, and `.codex/config.toml`.
- **Person:** a human identity. Person memory is global and describes the person, not projects.
- **Thread:** an Atelier workstream inside a project. Gateway threads and Codex session lineage bind to Atelier threads.
- **Job:** one Atelier-launched Codex run. Jobs live under `.atelier/jobs/` in the project.

## Install Atelier

Download a prebuilt alpha archive from GitHub Releases when available, or build from source:

```bash
git clone https://github.com/smarzola/atelier.git
cd atelier
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

Release tags build archives for Linux and macOS.

## Initialize Atelier

```bash
atelier home init ~/atelier-home
```

This creates a home project, starter Codex-native skills, and registers the `home` project alias.

## Projects

Initialize a project folder:

```bash
atelier project init ~/example-project --name example-project
```

Register or update a project alias:

```bash
atelier projects add example-project ~/example-project
```

List registered projects:

```bash
atelier projects list
```

A running gateway reads the project registry from disk on each request. Projects added with `atelier projects add` are visible to the gateway immediately; no gateway restart is required.

## People and person memory

Create a person:

```bash
atelier people add alice
```

Set person memory:

```bash
atelier people memory set alice "Prefers concise status updates."
```

Person memory is injected into Atelier-launched Codex work for the selected person. It must not store project facts.

## Threads and sessions

Create a thread:

```bash
THREAD=$(atelier thread new example-project "Release preparation" --porcelain)
```

List threads:

```bash
atelier threads list example-project
```

Show Codex session lineage for a thread:

```bash
atelier sessions example-project --thread "$THREAD"
```

## Work

Dry-run a Codex invocation without executing it:

```bash
atelier work example-project --thread "$THREAD" --as alice --dry-run "Summarize this project"
```

Run managed Codex app-server work:

```bash
atelier work example-project --thread "$THREAD" --as alice --managed "Summarize this project"
```

Continue a Codex session through Atelier:

```bash
atelier continue example-project --thread "$THREAD" --as alice --last "Continue the previous task"
```

Atelier does not rewrite Codex home or project config to inject runtime context. It passes explicit context into the invocation.

## Jobs and prompts

Show global runtime state:

```bash
atelier status
```

List jobs in a project:

```bash
atelier jobs list example-project
```

Show job details and log previews:

```bash
atelier jobs show example-project <job-id>
```

List pending prompts across projects:

```bash
atelier prompts inbox
```

List prompts in a project:

```bash
atelier prompts list example-project
```

Show a prompt:

```bash
atelier prompts show example-project <prompt-id>
```

Respond to a prompt:

```bash
atelier prompts respond example-project <prompt-id> accept
atelier prompts respond example-project <prompt-id> answer --text "Example answer"
atelier prompts respond example-project <prompt-id> accept --json '{"decision":"accept"}'
```

Recover idle or lost managed jobs:

```bash
atelier jobs recover example-project <job-id>
atelier jobs recover example-project --all-idle
atelier jobs recover example-project --all-worker-lost
```

## Daemon and hosted gateway

Start the always-alive Atelier daemon:

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

`atelier gateway serve` remains available as a compatibility/developer command for the same HTTP surface, but product usage should run the daemon.

### Gateway endpoints

Health:

```bash
curl -s http://127.0.0.1:8787/health
```

Status:

```bash
curl -s http://127.0.0.1:8787/status
```

Projects:

```bash
curl -s http://127.0.0.1:8787/projects
curl -s http://127.0.0.1:8787/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"api-project","path":"/tmp/api-project"}'
```

Jobs and prompts:

```bash
curl -s http://127.0.0.1:8787/jobs
curl -s http://127.0.0.1:8787/prompts
curl -s http://127.0.0.1:8787/prompts/respond \
  -H 'Content-Type: application/json' \
  -d '{"project":"example-project","prompt_id":"prompt-example","decision":"answer","text":"Example answer"}'
```

Generic message event:

```bash
curl -s http://127.0.0.1:8787/events/message \
  -H 'Content-Type: application/json' \
  -d '{"gateway":"example-gateway","project":"example-project","thread":"thread-example","person":"alice","text":"Run this task"}'
```

### Gateway bindings

Bind an external thread to an Atelier thread:

```bash
atelier gateway bind example-project \
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

### Telegram adapter

Bind a Telegram topic-style thread and user:

```bash
atelier gateway bind example-project \
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

## Audit logs

Gateway-originated actions append JSON Lines events to:

```text
~/.atelier/gateway/audit.jsonl
```

or:

```text
$ATELIER_HOME/gateway/audit.jsonl
```

Audit events currently cover project creation, prompt responses, and message-start actions.

## Codex-native capabilities

Add a project-local Codex skill:

```bash
atelier skill add project example-project /path/to/skill-folder
```

Add a project-local MCP server to `.codex/config.toml`:

```bash
atelier mcp add project example-project example-server -- command arg1 arg2
```

Atelier writes Codex-native capability files only when explicitly asked.
