# Atelier Agent Instructions

This repository is public-oriented. Never add real personal names, live identifiers, private paths, phone numbers, emails, addresses, tokens, or family details to docs, examples, tests, fixtures, logs, or code comments. Use generic identities such as Alice, Bob, Carol, Example User, Example Family, and Example Project.

## Project mission

Atelier is a Rust, project-native agent runtime around Codex CLI.

The durable unit of work is a project folder. Codex remains the only agentic execution backend. Atelier provides identity resolution, gateway routing, project discovery, job lifecycle, and resume affordances.

## Non-negotiable principles

- Project knowledge lives in the project folder.
- Global memory describes people only, never projects.
- Multiple people are first-class; each person has separate memory while shared project state remains unified.
- Do not rewrite `~/.codex`, project `.codex/config.toml`, or project `AGENTS.md` merely to inject person context.
- Prefer invocation-time context injection over hidden config mutation.
- Do not build overlapping tools when Codex already supports the capability.
- Use Codex-native mechanisms first: `AGENTS.md`, `.agents/skills`, `.codex/config.toml`, MCP, and Codex resume.
- Raw `cd project && codex` must remain a valid workflow.
- Keep state file-first and inspectable unless a database is clearly justified for indexing, locks, gateway bookkeeping, or performance.

## Documentation expectations

When changing behavior, update relevant docs in the same change. This project should be self-rehydrating: future contributors and agents should be able to resume from repository files without prior chat context.

Recommended bootstrap order for future agents:

1. Read `README.md`.
2. Read this `AGENTS.md`.
3. Read `docs/principles.md`.
4. Read `docs/architecture.md`.
5. Read `docs/decisions/`.
6. Read `docs/roadmap.md`.

## Coding expectations

The implementation language is Rust unless a document explicitly says otherwise.

When Rust code is added:

- Use small crates/modules with clear responsibilities.
- Add tests for core routing, identity, project discovery, and job behavior.
- Prefer explicit data structures over stringly typed state.
- Keep Codex invocation logic isolated behind a small adapter.

## Public hygiene check

Before committing, scan docs and examples for accidental private data. Generic examples only.
