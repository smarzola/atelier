# Roadmap

## Alpha milestone

Atelier has entered alpha. The core local CLI, managed Codex app-server path, gateway, Telegram adapter, project API, supervision, audit logging, and user-facing usage docs are available for dogfooding.

Next alpha work focuses on packaging and release readiness:

- [ ] Cross-platform binary builds.
- [ ] GitHub Release automation.
- [ ] Installation docs for released binaries.
- [ ] Better session indexing.
- [ ] Shared-project access controls.

## Phase 0: Design scaffold

- [x] Define project thesis.
- [x] Record core principles.
- [x] Record Codex runtime boundary decision.
- [x] Record context injection decision.
- [x] Create initial public GitHub repository.
- [x] Plan the Atelier home skills pack: `docs/plans/0001-home-skills-pack.md`.
- [x] Plan the Rust CLI vertical slice: `docs/plans/0002-rust-cli-vertical-slice.md`.
- [x] Plan Codex app-server prompt relay: `docs/plans/0003-codex-app-server-prompt-relay.md`.
- [x] Plan the generic HTTP gateway: `docs/plans/0004-generic-http-gateway.md`.
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

- gateway binding scaffold via `atelier gateway bind <project> --thread <thread-id> --gateway <name> --external-thread <id>`;
- gateway resolution scaffold via `atelier gateway resolve <project> --gateway <name> --external-thread <id>`;
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

- [x] Preserve Codex exit codes when work fails.
- [x] Record Codex invocation metadata in job status.
- [x] Keep hygiene scanner generic and cover it with a unit test.
- [x] Pass Codex run policy choices explicitly at invocation time: approval policy, sandbox, model, and search.
- [x] Support local interactive Codex runs so prompts and approvals are visible/respondable in the attached terminal.
- [x] Identify Codex app-server as the structured prompt-relay substrate for managed/gateway runs.
- [x] Implement initial Codex app-server managed runs and pending prompt state.
- [x] Add local prompt list/show/respond commands for managed-run prompt records.
- [x] Keep Codex app-server workers alive so prompt responses resume the running Codex turn.
- [x] List multiple managed jobs in one project.
- [x] Recover idle managed jobs from saved job context.
- [x] Reconcile running managed jobs whose worker process has disappeared.
- [x] Record managed Codex app-server thread/session metadata in Atelier thread lineage.
- [x] Enforce a conservative single-writer managed-work policy per project.
- [x] Validate prompt response decisions and support text/JSON payload responses.
- [x] Add job show inspection and durable managed-worker stdout/stderr logs.
- [x] Resolve registered project aliases for work/jobs/prompts/sessions.
- [x] Add global status dashboard across registered projects.
- [x] Add cross-project prompt inbox.
- [x] Add project-scoped bulk recovery for idle-timeout and worker-lost jobs.
- [x] Bootstrap an Atelier home workspace with starter Codex-native skills.
- [x] Add generic local HTTP gateway with health/status/jobs/prompts/respond/message endpoints.
- [x] Add gateway person bindings and external-thread routing for message events.
- [x] Default the HTTP gateway to loopback-only and support bearer-token authentication for adapter/reverse-proxy use.
- [x] Add an initial Telegram webhook adapter that translates message updates into generic gateway events.
- [x] Add optional gateway worker supervision that periodically reconciles dead managed workers.
- [x] Add file-first audit logs for gateway-originated prompt responses and message-start actions.
- [ ] Access control for shared projects.
- [ ] Better session indexing.
- [ ] Cross-platform packaging.
- [ ] Documentation site.
- [ ] CI and release automation.
- [x] Publish alpha README quickstart and usage guide.
