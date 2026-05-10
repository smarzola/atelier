# Atelier

Atelier is a project-native agent runtime for people who want autonomous agents to work the way humans work: in durable project folders, with inspectable artifacts, resumable sessions, and shared project context.

Atelier is intentionally opinionated. It does not try to be a general model router or a second tool ecosystem. Codex CLI is the agentic worker. Atelier provides the surrounding runtime: identity, project routing, gateways, session discovery, and job lifecycle.

## Status: alpha

Atelier is in alpha. The core local CLI and daemon-hosted gateway are usable for dogfooding, but interfaces may still change before stable releases.

Alpha includes:

- project/home initialization and registry aliases;
- person-scoped memory outside projects;
- thread and Codex session lineage files;
- explicit Codex context injection without hidden Codex config mutation;
- managed Codex app-server workers and prompt relay;
- job inspection, recovery, global status, and prompt inboxes;
- a loopback-first HTTP gateway with optional bearer auth;
- generic gateway message/prompt/project endpoints;
- an initial Telegram webhook adapter;
- worker supervision and gateway audit logs;
- an accepted daemon architecture decision: Atelier-managed work requires an always-alive daemon, and the gateway is hosted inside that daemon.

## Quickstart

### 1. Install prerequisites

Install Rust and Codex CLI, then authenticate Codex:

```bash
npm i -g @openai/codex
codex login
```

Download prebuilt alpha binaries from GitHub Releases when available, or build from source:

```bash
git clone https://github.com/smarzola/atelier.git
cd atelier
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

Release tags build Linux and macOS binary archives through GitHub Actions.

### 2. Create a home workspace and a project

```bash
atelier home init ~/atelier-home
atelier project init ~/example-project --name example-project
atelier projects add example-project ~/example-project
```

### 3. Start the daemon

Atelier-managed work requires an always-alive daemon. The daemon hosts gateway endpoints and supervises managed workers. Keep this running in its own terminal.

```bash
atelier daemon run --listen 127.0.0.1:8787
```

For adapter or reverse-proxy use, add bearer auth:

```bash
ATELIER_GATEWAY_TOKEN='replace-with-secret' atelier daemon run \
  --listen 127.0.0.1:8787 \
  --auth-token-env ATELIER_GATEWAY_TOKEN
```

### 4. Create a thread and run managed work

In another terminal:

```bash
THREAD=$(atelier thread new example-project "First task" --porcelain)
atelier work example-project --thread "$THREAD" --as alice --managed "Summarize this project"
```

Dry-runs and inspection commands can run without the daemon because they do not start managed workers:

```bash
atelier work example-project --thread "$THREAD" --as alice --dry-run "Summarize this project"
```

### 5. Inspect runtime state

```bash
atelier status
atelier jobs list example-project
atelier prompts inbox
atelier sessions example-project --thread "$THREAD"
```

### 6. Use the daemon API

Project registry API:
```bash
curl -s http://127.0.0.1:8787/projects

curl -s http://127.0.0.1:8787/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"api-project","path":"/tmp/api-project"}'
```

Messages can be routed through the generic event endpoint:
```bash
curl -s http://127.0.0.1:8787/events/message \
  -H 'Content-Type: application/json' \
  -d '{"gateway":"example-gateway","project":"example-project","thread":"thread-example","person":"alice","text":"Run this task"}'
```

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

6. **Raw Codex remains valid**
   - `cd project && codex` should remain a valid way to work.
   - Atelier may add identity, gateway routing, job orchestration, and context injection, but it must not hide essential project semantics outside the folder.

7. **No hidden Codex config mutation for context injection**
   - Atelier should not rewrite a person's `~/.codex` files or project `.codex/config.toml` merely to inject runtime context.
   - Person context should be injected as runtime task context or another explicit invocation-time mechanism.

8. **Do not duplicate Codex tools**
   - If Codex already supports a capability, Atelier should not build an overlapping tool.
   - Users can add capabilities through Codex-native mechanisms such as MCP, skills, and project instructions.

9. **Threads bind gateways to workstreams**
   - An Atelier thread is one ongoing workstream in a project or home workspace.
   - Telegram topics, reply roots, synthetic chat selections, CLI sessions, and Codex session lineage attach to Atelier threads.
   - Multiple threads may exist in one project, with conservative write concurrency by default.

10. **File-first and inspectable**
    - Prefer readable files and folders over opaque databases for project state.
    - Use databases only for indexes, locks, gateway bookkeeping, or performance.

11. **Public examples use generic identities**
    - Documentation, tests, examples, and fixtures use generic names such as Alice, Bob, and Carol.
    - Do not include real personal names, live identifiers, private paths, tokens, or family details in the public repository.

## Relationship to Codex

Atelier is not a replacement for Codex. Atelier is a project and gateway runtime around Codex.

Codex already provides important primitives Atelier should use rather than duplicate:

- `AGENTS.md` for global and project instructions.
- `.agents/skills` for reusable workflows.
- `.codex/config.toml` for project-scoped configuration, including MCP servers in trusted projects.
- `codex resume` and `codex exec resume` for interactive and non-interactive session continuation.
- Built-in web search support, with user-controlled behavior.

Atelier's job is to make these project-native and multi-person.

## Project layout

A project may look like this:

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

The `.atelier` folder stores Atelier's project-local runtime artifacts. Codex-native files remain Codex-native.

## Runtime sketch

```text
Gateway / CLI / API
        |
        v
 Atelier runtime
        |
        +--> identity resolver --> person memory
        |
        +--> project router ----> project folder
        |
        +--> thread resolver ---> .atelier/threads/<thread-id>/
        |
        +--> job manager -------> .atelier/jobs/<job-id>/
        |
        v
 Codex app-server / codex exec inside the project root
```

## Documentation

- [Usage Guide](docs/usage.md)
- [Codex Runtime](docs/codex-runtime.md)
- [Architecture](docs/architecture.md)
- [Principles](docs/principles.md)
- [Roadmap](docs/roadmap.md)
