# Roadmap

## Phase 0: Design scaffold

- [x] Define project thesis.
- [x] Record core principles.
- [x] Record Codex runtime boundary decision.
- [x] Record context injection decision.
- [x] Create initial public GitHub repository.
- [x] Plan the Atelier home skills pack: `docs/plans/0001-home-skills-pack.md`.
- [x] Document external Codex runtime and CI strategy: `docs/codex-runtime.md`.
- [x] Record Codex external runtime decision: `docs/decisions/0004-codex-external-runtime.md`.

## Phase 1: Minimal local CLI

Goal: prove project-native local workflows without a daemon.

Possible commands:

```bash
atelier project init [path] --name <name>
atelier projects list
atelier projects add <name> <path>
atelier work <project> --thread <thread-id> --as <person> "prompt"
atelier threads list <project>
atelier thread new <project> "Release preparation"
atelier sessions <project> --thread <thread-id>
atelier continue <project> --thread <thread-id> --as <person> --last "prompt"
atelier continue <project> --thread <thread-id> --as <person> --session <session-id> "prompt"
```

Expected behavior:

- project registry stored in `~/.atelier/registry.toml`;
- project-local `.atelier/` created explicitly;
- Codex invoked in the project root via `codex exec --cd <project> <context>`;
- thread folder created for each ongoing workstream;
- job folder created for each Atelier-launched run with request, context, status, stdout result, and stderr log;
- Codex session IDs recorded as thread lineage when available;
- no hidden Codex config mutation;
- `atelier doctor` reports Codex runtime readiness and optional project scaffold health.

## Phase 2: Identity and person memory

Goal: support multiple people locally before gateways.

Possible commands:

```bash
atelier people add alice
atelier people memory set alice "Prefers concise progress updates."
atelier work <project> --as alice --thread <thread-id> "prompt"
```

Expected behavior:

- person memory stored outside projects;
- person memory injected into Codex invocation context;
- project facts recorded only in project files.

## Phase 3: Gateway daemon

Goal: long-lived Atelier process with message routing.

Possible capabilities:

- Telegram adapter first, or generic webhook first;
- map gateway identities to people;
- bind Telegram topics, reply roots, or synthetic selections to Atelier threads;
- route messages to home or named projects;
- create background jobs;
- send completion notifications;
- expose thread, session list, and resume commands;
- default to single-writer concurrency per project.

## Phase 4: Codex-native capability management

Goal: provide explicit management UX for Codex-native capabilities without duplicating execution.

Possible commands:

```bash
atelier mcp add project <project> <name> -- <command> [args...]
atelier skill add project <project> <skill-folder>
atelier doctor --project <project>
```

Expected behavior:

- writes Codex-native config only through explicit management commands;
- never creates overlapping tools when Codex already supports the capability.

## Phase 5: Hardening

- Access control for shared projects.
- Better session indexing.
- Audit logs.
- Cross-platform packaging.
- Documentation site.
- CI and release automation.
