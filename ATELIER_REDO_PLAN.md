# Atelier Redo Plan

## Scope

Rollback the non-Atelier implementation of issue #12 and the issue closures that were made from that implementation, then redo the work through Atelier itself.

This plan intentionally keeps commit `73735d8 docs: plan thread-native interaction model`. That commit is the project-native plan for issue #12 and is useful input for the redo. The rollback target is the implementation and closure evidence after that plan.

Current rollback range:

```text
73735d8..HEAD
```

Current `HEAD` at inspection time:

```text
e30f12b feat: harden gateway recovery ergonomics
```

## History Findings

Issue #12 implementation commits after the plan:

```text
0e2437f feat: add thread event stream
1ea1ce8 feat: emit managed job thread events
c47cdd2 test: tolerate transient status writes
fdcacba feat: route work through thread interaction service
32cc448 feat: expose thread events through daemon API
0a6f165 feat: add thread send and follow commands
124de19 feat: route prompt replies through thread messages
ef85095 feat: queue thread messages while jobs run
dc61ad7 feat: track thread event delivery cursors
ff2f54f feat: publish thread results to telegram
818a909 feat: deliver bounded thread progress
bbd3bb0 docs: finalize thread-native interaction rollout
```

Related follow-up commit that closed adjacent dogfood/gateway issues and depends on the same implementation line:

```text
e30f12b feat: harden gateway recovery ergonomics
```

GitHub issue state observed during inspection:

```text
#12 CLOSED as COMPLETED at 2026-05-10T23:26:50Z
#7  CLOSED as COMPLETED at 2026-05-11T20:21:00Z
#8  CLOSED as COMPLETED at 2026-05-11T20:21:03Z
#9  CLOSED as COMPLETED at 2026-05-11T20:21:05Z
#10 CLOSED as COMPLETED at 2026-05-11T20:21:07Z
#11 CLOSED as COMPLETED at 2026-05-11T20:21:10Z
```

Issue #12 closure evidence cites the final implementation commits `818a909` and `bbd3bb0`. Issue #11 closure evidence cites `ff2f54f`, `818a909`, and `e30f12b`. Issues #7 through #10 cite `e30f12b`.

No commit message in the inspected range contains an automatic `Closes #12`, `Fixes #12`, or equivalent trailer. The issue closures appear to have been manual GitHub issue comments and close actions.

## Exact Git Revert Plan

1. Start from a clean working tree on `main`.

   ```bash
   git status --short
   ```

2. Create a rollback branch.

   ```bash
   git switch -c redo/issue-12-atelier-native-rollback
   ```

3. Revert the dependent implementation commits newest-to-oldest in one staged rollback.

   ```bash
   git revert --no-commit \
     e30f12b \
     bbd3bb0 \
     818a909 \
     ff2f54f \
     dc61ad7 \
     ef85095 \
     124de19 \
     0a6f165 \
     32cc448 \
     fdcacba \
     c47cdd2 \
     1ea1ce8 \
     0e2437f
   ```

4. If conflicts occur, resolve them by restoring the repository behavior to commit `73735d8` plus the new planning file only. Do not remove `docs/plans/0006-thread-native-interaction.md`.

5. Inspect the staged rollback before committing.

   ```bash
   git diff --cached --stat
   git diff --cached
   ```

6. Run verification after conflict resolution.

   ```bash
   cargo fmt --check
   cargo test --workspace
   python -m unittest discover -s . -p 'tests_*.py' -v
   ./scripts/hygiene-scan.sh
   git diff --check
   ```

7. Commit the rollback.

   ```bash
   git commit -m "revert: rollback non-atelier issue 12 implementation"
   ```

8. Push the rollback branch and open a PR.

   ```bash
   git push -u origin redo/issue-12-atelier-native-rollback
   gh pr create --fill
   ```

## Issue Tracker Rollback Plan

After the rollback PR exists, reopen the issues whose closures relied on the reverted commits. Add comments that point to the rollback PR and state that the implementation will be redone through Atelier itself.

```bash
gh issue reopen 12 --comment "Reopening because the non-Atelier implementation is being rolled back and the issue will be redone through Atelier itself. Rollback PR: <PR URL>."
gh issue reopen 11 --comment "Reopening because the cited implementation commits are included in the issue #12 rollback. Rollback PR: <PR URL>."
gh issue reopen 10 --comment "Reopening because the closure cited commit e30f12b, which is included in the issue #12 rollback. Rollback PR: <PR URL>."
gh issue reopen 9 --comment "Reopening because the closure cited commit e30f12b, which is included in the issue #12 rollback. Rollback PR: <PR URL>."
gh issue reopen 8 --comment "Reopening because the closure cited commit e30f12b, which is included in the issue #12 rollback. Rollback PR: <PR URL>."
gh issue reopen 7 --comment "Reopening because the closure cited commit e30f12b, which is included in the issue #12 rollback. Rollback PR: <PR URL>."
```

Recommended order is `#12`, then `#11`, then `#10` through `#7`, so the primary issue is visibly reopened before the dependent dogfood/gateway issues.

## Redo Through Atelier

After the rollback PR is merged:

1. Start a new Atelier-managed thread for issue #12 using the retained plan in `docs/plans/0006-thread-native-interaction.md`.
2. Use `atelier work` or the current project-supported Atelier workflow, not direct manual implementation, for each implementation slice.
3. Keep each redo slice small enough to review independently:
   - core thread event model;
   - managed worker event emission;
   - shared interaction routing;
   - daemon event API;
   - CLI thread send/follow;
   - prompt reply unification;
   - queueing and delivery cursors;
   - Telegram bounded delivery;
   - docs and closure verification.
4. Re-close issues only after the Atelier-managed redo commits are merged and verification evidence is attached to the relevant issues.
