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

Global memory is person memory only. It is safe to inject into any project session for that person because it contains no project facts.

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

## Resume model

Codex stores local transcripts and supports native resume commands:

```bash
codex resume
codex resume --last
codex resume --all
codex resume <SESSION_ID>
codex exec resume --last "Continue with the next step"
codex exec resume <SESSION_ID> "Continue with the next step"
```

Atelier should wrap these capabilities rather than reimplementing transcripts.

Initial commands might be:

```bash
atelier sessions list [--project <name>] [--home]
atelier resume [--project <name>] [--last | <session-id>]
atelier continue [--project <name>] [--last | <session-id>] "prompt"
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

## Gateway model

A gateway message is resolved into three layers:

1. person identity;
2. project target;
3. task or command.

Example:

```text
/work example-project summarize the latest research notes
```

Atelier resolves the sender to a person, loads only that person's memory, routes to `example-project`, creates a job folder, and invokes Codex in the project root.

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
