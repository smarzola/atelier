# Atelier

Atelier is a project-native runtime around Codex CLI.

The idea is simple: people work in projects, so agents should work in projects too. Atelier adds an always-alive daemon, gateway/API surface, person identity, project routing, threads, jobs, prompt relay, and session lineage around Codex. Codex remains the only agentic worker.

## Status: alpha

Atelier is usable for dogfooding, but interfaces may change before stable releases.

The alpha currently supports:

- home workspace initialization;
- person-scoped memory that is injected into Atelier-launched Codex work;
- project initialization with automatic registry aliases;
- daemon-managed `atelier work`;
- dry-runs without a daemon;
- threads, jobs, prompts, recovery, and session lineage;
- a loopback-first daemon HTTP API;
- generic gateway events;
- a Telegram daemon gateway with webhook setup, secret validation, inbound update routing, outbound Bot API `sendMessage`, and job-start acknowledgements.

Release archives are built only for:

- macOS Apple Silicon: `aarch64-apple-darwin`
- Linux ARM64: `aarch64-unknown-linux-gnu`
- Linux x86_64: `x86_64-unknown-linux-gnu`

## A small usage example

This walkthrough creates a tiny `hello-world` project, asks Atelier/Codex to write a file, handles a Codex approval prompt through the daemon API, and then inspects the result.

### 1. Install Codex and Atelier

Install and authenticate Codex first:

```bash
npm i -g @openai/codex
codex login
```

Install Atelier from a release archive when available, or build it from source:

```bash
git clone https://github.com/smarzola/atelier.git
cd atelier
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

### 2. Create the home workspace and person profile

Atelier's home workspace stores global runtime state and person memory. Person memory describes people only. Project facts belong in project folders.

```bash
atelier home init ~/atelier-home
atelier people add alice
atelier people memory set alice "Prefers short, practical examples."
```

### 3. Start the daemon

Atelier is daemon-first. Ordinary `atelier work` is daemon-managed by default. If the daemon is not running, work fails instead of silently starting unmanaged local work.

Run this in a separate terminal and leave it running:

```bash
atelier daemon run --listen 127.0.0.1:8787
```

The daemon hosts the local API, supervises Codex workers, and is the surface that gateways use.

### 4. Create a project

Project initialization now does the obvious thing: it creates the project scaffold and registers the project alias in one command.

```bash
mkdir -p ~/hello-world
atelier project init ~/hello-world --name hello-world
```

Add project instructions and content:

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

### 5. Create a thread and dry-run the first task

Threads are workstreams inside projects. A dry-run is useful before the first real task because it shows the exact context Atelier will inject into Codex.

```bash
THREAD=$(atelier thread new hello-world "Build a friendly greeting" --porcelain)

atelier work hello-world \
  --thread "$THREAD" \
  --as alice \
  --dry-run \
  "Create a tiny hello-world note"
```

The dry-run is intentionally daemon-free. It records a dry-run job and prints the Codex invocation and injected person/thread context.

### 6. Send work into the thread from the CLI

```bash
atelier thread send hello-world \
  --thread "$THREAD" \
  --as alice \
  "Create HELLO.md with a friendly one-paragraph greeting for this project."
```

`atelier thread send` submits the message to the daemon-managed thread interaction path. `atelier work` remains available as a compatibility shorthand, but thread-native send/follow is the preferred ongoing workflow. If another job is already running in the project, the message is persisted to the thread's `queued-messages.jsonl` rather than starting an overlapping writer. Inspect it from the CLI:

```bash
atelier jobs list hello-world
atelier jobs show hello-world <job-id>
atelier prompts inbox
atelier thread follow hello-world --thread "$THREAD" --after 0
```

If Codex needs approval, the job becomes `waiting-for-prompt`. You can answer a single pending approval through the thread:

```bash
atelier thread send hello-world --thread "$THREAD" --as alice approve
```

Use `atelier prompts inbox/show/respond` when you need to inspect details or send structured prompt input. While dogfooding this flow, Codex asked for file-change approval before writing `HELLO.md`.

### 7. Use the API for the same workflow

Check the daemon:

```bash
curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/projects
```

Start work through the API instead of the CLI:

```bash
curl -s http://127.0.0.1:8787/work \
  -H 'Content-Type: application/json' \
  -d "{\"project\":\"hello-world\",\"thread\":\"$THREAD\",\"person\":\"alice\",\"text\":\"Append one more friendly sentence to HELLO.md.\"}"
```

List and answer prompts:

```bash
curl -s http://127.0.0.1:8787/prompts

curl -s http://127.0.0.1:8787/prompts/respond \
  -H 'Content-Type: application/json' \
  -d '{"project":"hello-world","prompt_id":"prompt-0","decision":"accept"}'
```

Then check job status and the file Codex created:

```bash
curl -s http://127.0.0.1:8787/jobs
cat ~/hello-world/HELLO.md
atelier sessions hello-world --thread "$THREAD"
```

The session command shows the Codex session lineage attached to this Atelier thread.

### 8. Create a project through the API

The daemon can also initialize and register projects:

```bash
curl -s http://127.0.0.1:8787/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"api-created","path":"/tmp/api-created"}'
```

This creates the project folder, writes the starter Atelier/Codex files, registers the project alias, and records a gateway audit event.

## Core principles

1. **Project-native by default**
   - A project is any durable working folder: software, documentation, wiki, research, household administration, email workflows, or anything else that benefits from local state.

2. **Codex is the only agentic execution backend**
   - Atelier delegates autonomous work to Codex CLI.
   - No provider abstraction is planned until the Codex-only constraint clearly stops making sense.

3. **Project knowledge lives in the project**
   - Project instructions, skills, MCP configuration, notes, memory, jobs, and artifacts belong inside the project folder whenever possible.
   - A person should be able to enter the folder and understand the project without reading prior chats.

4. **Person memory is global and separate**
   - Global memory describes people: preferences, collaboration style, stable personal context, and identity.
   - Global memory must not accumulate project facts.
   - Project facts must be recorded in project files.

5. **Multiple people are first-class**
   - Atelier resolves each gateway identity to a person.
   - Each person has separate global memory.
   - Shared projects remain unified because project knowledge lives in the project folder.

6. **Raw Codex remains valid, but it is not the Atelier user flow**
   - Atelier should not make project folders unusable without Atelier.
   - The primary product path is daemon-managed Atelier work.

7. **No hidden Codex config mutation for context injection**
   - Atelier should not rewrite a person's `~/.codex` files or project `.codex/config.toml` merely to inject runtime context.
   - Person context should be injected as runtime task context or another explicit invocation-time mechanism.

8. **Do not duplicate Codex tools**
   - If Codex already supports a capability, Atelier should not build an overlapping tool.
   - Users can add capabilities through Codex-native mechanisms such as MCP, skills, and project instructions.

9. **Threads bind gateways to workstreams**
   - An Atelier thread is one ongoing workstream in a project or home workspace.
   - Telegram topics, reply roots, synthetic chat selections, CLI sessions, and Codex session lineage attach to Atelier threads.

10. **File-first and inspectable**
    - Prefer readable files and folders over opaque databases for project state.
    - Use databases only for indexes, locks, gateway bookkeeping, or performance.

11. **Public examples use generic identities**
    - Documentation, tests, examples, and fixtures use generic names such as Alice, Bob, and Carol.
    - Do not include real personal names, live identifiers, private paths, tokens, or family details in the public repository.

## Documentation

- [Usage Guide](docs/usage.md)
- [Codex Runtime](docs/codex-runtime.md)
- [Architecture](docs/architecture.md)
- [Principles](docs/principles.md)
- [Roadmap](docs/roadmap.md)
