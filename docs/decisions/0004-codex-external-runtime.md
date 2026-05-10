# Decision 0004: Codex is an external runtime

## Status

Accepted for initial design.

## Context

Atelier delegates autonomous work to Codex CLI. One option would be to bundle a Codex binary inside Atelier releases. That would make first-run setup easier, but it would also make Atelier responsible for Codex release tracking, platform binaries, security updates, authentication behavior, sandbox changes, MCP behavior, and session/resume compatibility.

A bundled private Codex binary could also violate Atelier's raw Codex equivalence principle: `cd project && codex` should behave consistently with Atelier-invoked Codex for project semantics.

## Decision

Atelier does not bundle Codex initially.

Atelier treats Codex as an external required runtime, similar to `git` or `docker`:

- users install Codex separately;
- Atelier discovers `codex` on `PATH` by default;
- Atelier allows an explicit Codex binary path override;
- `atelier doctor` validates Codex availability, version, and basic execution readiness;
- Atelier records Codex version and invocation metadata in job folders.

## Installation guidance

Atelier documentation should direct users to install Codex through one of the upstream-supported methods, for example:

```bash
npm i -g @openai/codex
# or
brew install --cask codex
```

Users then authenticate Codex through upstream-supported mechanisms:

```bash
codex login
# or, for programmatic environments:
printenv OPENAI_API_KEY | codex login --with-api-key
```

## Configuration

Atelier should support a configurable Codex binary path:

```toml
[codex]
binary = "codex"
minimum_version = "0.130.0"
```

Environment override:

```bash
ATELIER_CODEX=/path/to/codex atelier work example-project "..."
```

## CI strategy

Atelier CI should not require a live OpenAI or ChatGPT login for normal tests.

Default CI should use a fake Codex executable that records argv/stdin and returns deterministic output. This lets tests verify:

- command construction;
- working directory selection;
- prompt/context injection;
- resume command construction;
- failure handling;
- job metadata recording.

A separate optional smoke workflow may run against real Codex only when repository secrets are configured. That workflow should be manually triggered or clearly marked as integration-only.

## Consequences

Positive:

- Atelier remains small and focused.
- Codex can update independently.
- Raw `cd project && codex` remains the reference behavior.
- CI remains deterministic and does not spend model/API credits by default.

Negative:

- Users must install and authenticate Codex separately.
- Atelier must provide clear doctor/install guidance.
- Integration testing against real Codex must be opt-in.

## Revisit when

- Users need a managed Codex installer for onboarding.
- Codex exposes a stable embeddable library/API that is better than shelling out.
- A future distribution needs an opt-in bundled or managed Codex binary.
