# Atelier Home Skills Pack Plan

**Goal:** Design the first set of Codex-native skills that let Atelier understand, operate, maintain, and improve itself from the home workspace and project folders.

**Architecture:** Atelier skills are Codex skills stored in project-local `.agents/skills/` folders. The home workspace contains the global Atelier operations skills. Individual projects may add local skills for project-specific workflows. Atelier should manage skill placement explicitly but should not create a parallel skill runtime.

**Tech Stack:** Codex CLI skills (`SKILL.md`), `AGENTS.md`, project-local `.agents/skills/`, optional MCP through Codex `.codex/config.toml`, file-first state under `.atelier/`.

---

## Design principles for skills

1. **Codex-native first**
   - Skills are normal Codex skills.
   - Atelier does not invent a separate skill format.
   - Skills live in `.agents/skills/<skill-name>/SKILL.md`.

2. **Home skills are about Atelier operations**
   - Home skills teach Codex how to operate Atelier itself: project routing, thread binding, session resume, memory boundaries, job lifecycle, and public hygiene.
   - They must not contain private person facts.

3. **Project skills are about project work**
   - Project-specific workflows live in the project folder.
   - Examples: release process, documentation cleanup, research synthesis, email triage.

4. **Skills should be small and composable**
   - Prefer a few focused skills over one giant operating manual.
   - Each skill should have a clear trigger description.

5. **Skills should preserve raw Codex equivalence**
   - Running `cd project && codex` should still load relevant project skills.
   - Atelier may install or scaffold skills, but Codex executes them.

6. **Public examples only**
   - Use Alice, Bob, Carol, Example Project, and `example.com`.
   - Do not include private names, live identifiers, local private paths, tokens, or production chat IDs.

---

## Proposed home workspace layout

```text
atelier-home/
  AGENTS.md
  .agents/
    skills/
      atelier-self-orientation/
        SKILL.md
      atelier-project-router/
        SKILL.md
      atelier-thread-operator/
        SKILL.md
      atelier-codex-session-resume/
        SKILL.md
      atelier-person-memory-boundaries/
        SKILL.md
      atelier-job-operator/
        SKILL.md
      atelier-public-hygiene/
        SKILL.md
      atelier-skill-authoring/
        SKILL.md
  .atelier/
    inbox/
    threads/
    jobs/
    memory/
    artifacts/
```

The repository may also ship templates for generating a home workspace:

```text
templates/
  home/
    AGENTS.md
    .agents/skills/...
```

---

## Skill 1: `atelier-self-orientation`

**Purpose:** Help Codex orient itself inside an Atelier home or project workspace.

**Trigger description:** Use when starting work in an Atelier home or project folder, when asked to explain Atelier, or when resuming without enough context.

**Responsibilities:**

- Read `AGENTS.md`, `README.md`, `docs/principles.md`, `docs/architecture.md`, and relevant ADRs.
- Determine whether the current folder is home, a project, or a repository checkout.
- Identify `.atelier/`, `.agents/skills/`, and `.codex/config.toml` if present.
- Summarize the safe working context without inventing missing facts.

**Must not:**

- Store project facts in person memory.
- Rewrite Codex config.
- Assume a gateway identity if none was provided.

---

## Skill 2: `atelier-project-router`

**Purpose:** Route user requests to the correct project or home workspace.

**Trigger description:** Use when a request names or implies a project, asks to switch projects, creates a project, or drops information into an inbox.

**Responsibilities:**

- Inspect the Atelier project registry when available.
- Resolve explicit project names first.
- Fall back to current thread binding, chat default, or home.
- If ambiguous, produce a short set of candidate projects rather than guessing.
- Record routing decisions in `.atelier/threads/<thread-id>/` or job context when running through Atelier.

**Must not:**

- Move project knowledge into global person memory.
- Create new projects implicitly when a likely existing project exists.

---

## Skill 3: `atelier-thread-operator`

**Purpose:** Manage Atelier threads as durable workstreams.

**Trigger description:** Use when creating, binding, listing, resuming, summarizing, or closing an Atelier thread; use for Telegram topic/thread work.

**Responsibilities:**

- Understand that gateway topics bind to Atelier threads.
- Maintain thread metadata under `.atelier/threads/<thread-id>/`.
- Keep `thread.toml`, `summary.md`, `gateway-bindings.toml`, and `codex-sessions.jsonl` coherent.
- Support Telegram native topics, reply-root fallbacks, and synthetic threads as bindings.
- Keep thread identity separate from person identity.

**Must not:**

- Treat a whole chat as a single session if a thread binding exists.
- Inject private memory for non-speaking participants.

---

## Skill 4: `atelier-codex-session-resume`

**Purpose:** Resume Codex sessions safely through Atelier threads.

**Trigger description:** Use when asked to resume, continue, list previous sessions, or bind a Codex session to an Atelier thread.

**Responsibilities:**

- Prefer Codex native resume commands:
  - `codex resume`
  - `codex resume --last`
  - `codex resume --all`
  - `codex resume <SESSION_ID>`
  - `codex exec resume --last "..."`
  - `codex exec resume <SESSION_ID> "..."`
- Always run/resume from the intended project root.
- Store known session IDs as thread lineage.
- Use thread summaries to reduce confusion across long-running workstreams.

**Must not:**

- Reimplement Codex transcripts as the source of truth.
- Rely on current working directory accidentally when a project root is known.

---

## Skill 5: `atelier-person-memory-boundaries`

**Purpose:** Enforce the boundary between global person memory and project knowledge.

**Trigger description:** Use when writing memory, summarizing conversations, handling multi-person shared threads, or deciding where durable information belongs.

**Responsibilities:**

- Person memory stores stable facts about people only.
- Project facts go into project files.
- In shared threads, inject only the current speaker's person memory unless other memory is explicitly marked shared or permissioned.
- Prefer project-local notes, ADRs, task files, or `.atelier/memory/` for project knowledge.

**Must not:**

- Save project state as person memory.
- Leak one participant's private memory into another participant's prompt.

---

## Skill 6: `atelier-job-operator`

**Purpose:** Operate Atelier jobs from creation through completion.

**Trigger description:** Use when launching background work, checking status, recording results, or handling queued work.

**Responsibilities:**

- Create job folders under `.atelier/jobs/<job-id>/`.
- Record `request.md`, `context.md`, `status.json`, `result.md`, and Codex session pointer when available.
- Respect project concurrency policy.
- Default to single-writer per project.
- Send concise completion summaries through the gateway when applicable.

**Must not:**

- Start parallel write jobs in the same project unless a safe policy is configured.
- Hide failures; write them to job status and report them clearly.

---

## Skill 7: `atelier-public-hygiene`

**Purpose:** Protect the open-source repository from private identifiers.

**Trigger description:** Use before committing docs, examples, tests, fixtures, logs, or public repository content.

**Responsibilities:**

- Scan for real names, private local paths, chat IDs, account IDs, phone numbers, emails, tokens, and production identifiers.
- Replace examples with Alice, Bob, Carol, Example Project, Example Family, `example.com`, and `/home/example/...`.
- Keep public docs generic unless explicitly authorized otherwise.

**Must not:**

- Commit private memory context.
- Use real family names in public examples.

---

## Skill 8: `atelier-skill-authoring`

**Purpose:** Create and maintain Atelier skills themselves.

**Trigger description:** Use when adding, updating, validating, or organizing Atelier skills in home or project `.agents/skills/` folders.

**Responsibilities:**

- Follow Codex skill structure:
  - `.agents/skills/<skill-name>/SKILL.md`
  - YAML frontmatter with `name` and `description`
  - clear trigger-focused description
  - actionable instructions
  - common pitfalls
  - verification checklist
- Keep skills small and composable.
- Decide whether a skill belongs in home or in a project.
- Avoid duplicating existing Codex capabilities.

**Must not:**

- Create a parallel non-Codex skill format.
- Put project-specific workflows in the home skills pack unless they are Atelier operations workflows.

---

## Implementation tasks

### Task 1: Add Codex skill templates to the repository

**Objective:** Create reusable templates for Atelier skills.

**Files:**

- Create: `templates/skills/basic/SKILL.md`
- Create: `templates/skills/README.md`

**Steps:**

1. Write a minimal valid Codex skill template.
2. Include placeholders for name, description, overview, when to use, instructions, pitfalls, and verification.
3. Add notes about generic public examples.
4. Verify the template contains no private identifiers.

### Task 2: Add home workspace template

**Objective:** Create a file scaffold for an Atelier home workspace.

**Files:**

- Create: `templates/home/AGENTS.md`
- Create: `templates/home/README.md`
- Create directories documented in `templates/home/.atelier/README.md`

**Steps:**

1. Write home `AGENTS.md` that teaches Codex the home workspace role.
2. Reference the home skills pack.
3. Explain person-memory boundaries.
4. Explain project routing and thread binding.
5. Keep all examples generic.

### Task 3: Draft the first home skills

**Objective:** Add the initial skills under `templates/home/.agents/skills/`.

**Files:**

- Create one `SKILL.md` per skill listed above.

**Steps:**

1. Start with `atelier-self-orientation`.
2. Add `atelier-person-memory-boundaries`.
3. Add `atelier-thread-operator`.
4. Add `atelier-codex-session-resume`.
5. Add remaining skills.
6. Ensure descriptions are concise and trigger-focused.

### Task 4: Add validation script

**Objective:** Validate Atelier skill files for basic shape and hygiene.

**Files:**

- Create: `scripts/validate-skills.py`

**Steps:**

1. Walk `templates/**/.agents/skills/*/SKILL.md`.
2. Ensure frontmatter starts at byte zero.
3. Ensure `name` and `description` exist.
4. Ensure skill names are lowercase hyphenated.
5. Ensure public hygiene scan passes for configured patterns.
6. Exit non-zero on failure.

### Task 5: Document installation and update flow

**Objective:** Explain how skills are copied or managed into home/project workspaces.

**Files:**

- Create: `docs/skills.md`
- Modify: `README.md`
- Modify: `docs/roadmap.md`

**Steps:**

1. Explain home skills versus project skills.
2. Explain raw Codex behavior.
3. Explain how Atelier will eventually install/update skills explicitly.
4. State that skills are Codex-native.

---

## Verification checklist

- [ ] Skills use Codex-native `.agents/skills/<name>/SKILL.md` layout.
- [ ] Home skills contain only Atelier operations knowledge, not private person facts.
- [ ] Project-specific workflows are reserved for project-local skills.
- [ ] No hidden Codex config mutation is introduced.
- [ ] Public hygiene scan passes.
- [ ] README and roadmap link to the skills plan.
