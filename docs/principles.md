# Atelier Principles

This document records the design constraints Atelier should preserve as it evolves.

## 1. Project-native agency

The durable unit of work is a folder. A project can be a software repository, documentation workspace, wiki, research folder, email operations folder, or any other durable workspace.

Atelier should optimize for the way people actually work:

- enter a project;
- read local instructions;
- inspect notes and artifacts;
- run tools;
- leave durable output behind;
- return later and resume.

## 2. Codex-only execution

Atelier delegates autonomous agentic work to Codex CLI.

Atelier should not begin as a provider-agnostic agent framework. That would blur the product. The first version should be super opinionated:

- Codex runs the work.
- Atelier routes, contextualizes, records, resumes, and notifies.

If this constraint becomes harmful later, revisit it with a written decision record.

## 3. Person memory and project knowledge are separate

Atelier supports global memory, but global memory is about people only.

Good person-memory examples:

- Alice prefers concise status updates.
- Bob wants risky destructive actions confirmed.
- Carol prefers public examples to use generic names.

Bad person-memory examples:

- Project Alpha uses a Python test command.
- The documentation project has a `Drafts/` directory.
- The email project uses a specific MCP server.

Those are project facts and belong in project files.

## 4. Multiple people are first-class

Atelier is not single-user by assumption. It should map gateway identities to people:

```text
telegram:123 -> person:alice
slack:U123   -> person:bob
```

Each person has separate global memory. Projects are shared and unified because project knowledge is stored in the project folder.

## 5. Raw Codex equivalence

A user should be able to run:

```bash
cd example-project
codex
```

and get the project semantics from Codex-native files:

- `AGENTS.md`
- `.agents/skills`
- `.codex/config.toml`

Atelier-enhanced entrypoints may add identity, gateway metadata, job tracking, and person context, but must not hide core project behavior in Atelier-only state.

## 6. No hidden Codex config rewriting

Atelier should not rewrite a user's `~/.codex` files, project `.codex/config.toml`, or project `AGENTS.md` merely to inject person context.

Preferred approaches:

- pass person context in the initial prompt for `codex exec`;
- pass person context as explicit runtime context through a supported Codex mechanism if one exists;
- provide `atelier codex` as an explicit wrapper, not invisible mutation.

Any future mechanism that modifies Codex files must be opt-in, visible, and reversible.

## 7. Do not duplicate Codex capabilities

If Codex already supports a capability, Atelier should not build an overlapping tool.

Examples:

- Codex has web search support, so Atelier should not ship its own web crawler by default.
- Codex supports MCP, so external tools should be added through MCP rather than an Atelier-specific tool interface.
- Codex supports skills, so reusable workflows should use Codex skills first.

Atelier may build management UX around these features, but Codex remains the execution layer.

## 8. Session resume is a product primitive

Resuming previous work is not an advanced feature. Atelier should expose resume affordances from the beginning.

It should support:

- resume latest session in current project;
- list resumable sessions for a project;
- resume by session id;
- resume home sessions;
- resume non-interactive Codex runs where possible.

Atelier should lean on Codex's native `resume` and `exec resume` functionality rather than reimplementing transcripts.

## 9. File-first, inspectable state

Project-local runtime state should be readable and auditable:

```text
.atelier/
  inbox/
  jobs/
  sessions/
  memory/
  artifacts/
  journal.md
```

Databases may be used for indexes, locks, gateway bookkeeping, or performance, but they should not be the only place where durable project meaning lives.

## 10. Public hygiene

Atelier is open source. Public docs, examples, tests, and fixtures must use generic names and identifiers.

Use:

- Alice, Bob, Carol
- Example User
- Example Project
- Example Family
- `example.com`
- `/home/example/...`

Avoid:

- real family names;
- live account identifiers;
- private local paths;
- tokens or secrets;
- production chat IDs or phone numbers.
