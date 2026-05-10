# Atelier

Atelier is a project-native agent runtime for people who want autonomous agents to work the way humans work: in durable project folders, with inspectable artifacts, resumable sessions, and shared project context.

Atelier is intentionally opinionated. It does not try to be a general model router or a second tool ecosystem. Codex CLI is the agentic worker. Atelier provides the surrounding runtime: identity, project routing, gateways, session discovery, and job lifecycle.

## Thesis

Autonomous agents should not be organized primarily around ephemeral chat sessions. Human work is organized around projects: folders, notes, instructions, tools, artifacts, and history. Atelier makes the project folder the durable unit of agent work.

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

## Proposed project layout

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
 Atelier daemon
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
 codex exec / codex resume inside the project root
```

## Status

Early design and bootstrap phase.
