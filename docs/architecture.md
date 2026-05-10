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

```text
Gateway / CLI / API
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
 Codex adapter ---------> codex exec / codex resume
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

Thread folders store bindings, summaries, jobs, and Codex session lineage:

```text
.atelier/threads/thread-abc/
  thread.toml
  summary.md
  gateway-bindings.toml
  codex-sessions.jsonl
  jobs/
```

Atelier job folders should store pointers to Codex session IDs when available:

```text
.atelier/jobs/2026-01-01T120000Z-example/
  request.md
  context.md
  codex-session-id.txt
  status.json
  result.md
```

A project may have multiple Atelier threads in parallel, but write-capable jobs need a concurrency policy. The initial default should be `single-writer` per project: parallel threads are allowed, but writes to the same project are serialized or queued unless a project opts into a safer parallel strategy such as git worktrees.

## Gateway model

A gateway message is resolved into four layers:

1. person identity;
2. gateway thread/topic binding;
3. project or home target;
4. task or command.

Example:

```text
/work example-project summarize the latest research notes
```

Atelier resolves the sender to a person, loads only that person's memory, routes to `example-project`, resolves or creates an Atelier thread, creates a job folder, and invokes Codex in the project root.

### Gateway thread bindings

Gateway threads are bindings to Atelier threads, not the core object.

For Telegram, the preferred binding is a forum topic or topic-enabled private chat thread using `message_thread_id`:

```text
telegram:<chat_id>:<message_thread_id> -> atelier-thread:<thread-id>
```

If native topics are unavailable or unreliable, the Telegram adapter may fall back to reply-root threads or synthetic command-selected threads. The same Atelier thread model should also support future platforms such as Discord, Slack, CLI, and API clients.

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

Atelier may later provide management commands that write Codex-native configuration explicitly, for example:

```bash
atelier mcp add project example-project context7 -- npx -y @upstash/context7-mcp
atelier skill add project example-project ./skills/triage-inbox
```

These are management operations, not separate execution paths.
