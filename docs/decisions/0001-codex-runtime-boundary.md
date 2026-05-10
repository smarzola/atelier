# Decision 0001: Codex is the agentic execution backend

## Status

Accepted for initial design.

## Context

Atelier could become a general agent framework with multiple model providers, tool systems, and execution loops. That would overlap with existing systems and dilute the project-native idea.

Codex CLI already provides a maintained agentic runtime with:

- interactive and non-interactive modes;
- project instruction loading through `AGENTS.md`;
- skills;
- MCP;
- web search support;
- local session transcripts and resume;
- command execution and file editing.

## Decision

Atelier delegates autonomous work to Codex CLI. Atelier does not implement a competing model loop in the initial architecture.

Atelier owns identity, routing, project lifecycle, job records, gateways, and resume UX. Codex owns agentic execution.

## Consequences

Positive:

- Smaller implementation surface.
- Strong opinionated product shape.
- Immediate leverage from Codex features.
- Raw Codex remains a valid workflow inside project folders.

Negative:

- Atelier depends on Codex CLI behavior and command-line stability.
- Some desired runtime hooks may require upstream Codex support.
- Non-Codex models are out of scope unless a future decision changes this.

## Revisit when

- Codex lacks an essential capability that cannot be added through prompting, skills, MCP, or upstream contribution.
- Atelier's project-native runtime becomes useful enough to justify additional execution backends without weakening the core model.
