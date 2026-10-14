# Development checkpoint — 2026-10-14

## Preserved state

- Workspace: `D:\dev\nemu`.
- Branch: `task/P0-06-echo-baseline` (tracks origin; do not start a replacement branch).
- Baseline implementation commit: `747b94db97ebe64ad3b8a89420aed8c05bc0f92d`. This checkpoint is a subsequent documentation commit; obtain its exact ID with `git log -1 -- reports/P0-06/PAUSE_CHECKPOINT.md`.
- PR: https://github.com/nanake-lab/nemu/pull/3 — OPEN, `needs-user`, not merged.
- Main/base: `ab3c6d175f0c4ea156aa9a8baff429e741d3eb49`; origin/main matched at last fetch. Worktree was clean after the implementation commit; only this checkpoint was then added.
- All completed work/evidence committed. No original specification or runtime code modified; no user's rehearsal/life storage opened.

## Completed prior task

P0-02 merged: https://github.com/nanake-lab/nemu/pull/2. Parser 197 cases; contracts 312 tests; workspace 324 tests; parser line coverage 100% (104/104). Local build/static/Rust/Python validation and both CI jobs passed. Final PR CI: https://github.com/nanake-lab/nemu/actions/runs/38503748940. Post-merge CI: https://github.com/nanake-lab/nemu/actions/runs/38504208518 (success). Do not redo P0-02 implementation.

## P0-06 delivered and validated

All four W00 documents, E1–E14 register, source/acceptance mapping, proposed ADR 0006, report and blocker record exist. Five original hashes match. All W packages, 45 operator IDs, 25 SYS IDs, 22 invariants routed. Locked SQLite build probe uses rusqlite 0.40.2 / libsqlite3-sys 0.38.2 / SQLite 3.53.2; fmt, clippy, WAL/FULL/rollback/integrity and temporary cleanup passed. Evidence: `verification-20261014T031717034Z/` and `sqlite-build.log`.

P0-06 first CI was queued when this checkpoint was authored: https://github.com/nanake-lab/nemu/actions/runs/38505639151. The checkpoint push will supersede that run. **Do not infer final-head CI success from this snapshot**; fetch live checks using the commands below. Existing CI runs repository language/build/desktop checks; the report-only SQLite probe is separately validated locally, not automatically invoked by CI.

## Blocking decisions and exact resume instructions

1. Read AGENTS.md, architecture/contracts, Echo Core README and echo-core-work skill; then this checkpoint, REPORT.md, BLOCKED.md, the diff register and proposed ADR 0006.
2. Obtain explicit user direction for W00-C01: (a) later multi-root-only closure supersedes frozen single-root consolidation, (b) explicitly approve v1.0 baseline while deferring excluded growth conflict to v1.x, or (c) await corrected authoritative text. Until then retain NO-GROWTH and do not merge P0-06.
3. Obtain W00-C02 confirmation that the summary's “24” is a typo; retain all SYS01–SYS25. Do not reopen already approved E5/E8/E13/E14.
4. Verify current state (preserve unrelated changes):

   ```powershell
   git status --short
   git fetch origin
   git branch --show-current
   gh pr view 3 --json state,headRefOid,labels,statusCheckRollup
   gh run list --branch task/P0-06-echo-baseline --limit 5
   ```

5. Append the user's decisions to the register/report and ADR status; do not overwrite originals. Update the task branch from latest main, rerun `pwsh -File reports/P0-06/verify-baseline.ps1` (new timestamped evidence), and run all relevant CI after the update. A temporary repository-scoped Windows runner with label `nemu` is needed if none is online; setup lives in docs/SETUP.md. Runner files/credentials remain ignored in `.runner/`; never commit tokens.
6. Only after decisions, tests and both final-head CI jobs are green, remove `needs-user` and squash merge per AGENTS. If user opts to provide new upstream originals, re-baseline explicitly before considering acceptance complete.
7. Sync main, confirm clean tree, then take P5-01 (P0-02 already merged; P0-06 must be merged). No P5-01 implementation has started. Observe all prerequisite readings and directory ownership.

Stopping reason: required user decisions, not implementation failure or lowered acceptance. Scope outside v1.0 stays not_run. This checkpoint does not authorize any growth activation, model selection, birth or storage migration.
