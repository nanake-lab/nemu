# P0-01 pause checkpoint — 2026-10-14 (Australia/Perth)

Paused at the user's explicit request due to the usage limit. Do not resume work,
start validation, or merge until the user instructs continuation.

## Git and review

- Workspace: D:\dev\nemu
- Branch: task/P0-01-monorepo
- Implementation HEAD before this checkpoint: 23148c105ee2fa137f169594fe2c9404ea537eb1
- Initial implementation commit: 73778b8
- Working tree was clean before creating this checkpoint; all implementation and evidence were already committed and pushed.
- PR: https://github.com/nanake-lab/nemu/pull/1 — OPEN, not draft, not merged; merge status UNSTABLE at pause.
- This checkpoint is committed with [skip ci] to avoid starting new validation.

## Completed

- pnpm workspace, strict TS, ESLint, Prettier, exact dependencies, scripts, LFS rules.
- Twelve Cargo crate skeletons, dependency direction, pinned Rust, IDL and acceptance placeholders.
- Four independently locked uv projects, Python example tests/strict checks; .NET 10 connector skeleton.
- Electron transparent stage + normal panel, secure preload, browser Vite entry points, React 19.
- Seven public package skeletons, codex-bridge, content template, profiles/schemas, Codex version lock, W00 templates.
- Legacy Composer adapted from the user-confirmed candidate prototype; source inventory/hashes in migration.md.
  No legacy agent/store/IPC/Live2D assets or personal content copied.
- CI, PR template, setup instructions, budget warning, test-only transport and Low IL probes.
- All 34 local validation commands passed: final-validation/validation.json and logs.
  TypeScript: 13 tests; Python: one per project; Rust: skeleton builds/tests (zero domain tests).
- Electron transparency is checked with native captured alpha; frame readiness fixed a capture race.
  Parent-owned temporary userData cleanup occurs after exit; smoke passed three consecutive runs.
- Local SwiftShader white/black visual comparison passed; images visually inspected.
- pnpm dev started successfully and exited normally after closing both windows; no remaining scaffold temp directory was seen at the last check.
- pnpm format:check passed. Budget warning observed at 1680/2000 estimated minutes.

## CI at pause

- First run https://github.com/nanake-lab/nemu/actions/runs/38439189975:
  Ubuntu passed; Windows failed because setup-dotnet tried writing Program Files.
- Commit 23148c1 fixes Windows by detecting the installed pinned SDK, otherwise installing under runner.temp.
  It also removes duplicate task-branch push runs; PR CI and main pushes remain enabled.
- Duplicate old PR run 38439330771 was cancelled.
- Latest run https://github.com/nanake-lab/nemu/actions/runs/38439576599:
  Ubuntu SUCCESS. Windows SDK detection, named pipe, Low IL, and Electron e2e SUCCESS;
  visual step was IN_PROGRESS when pause arrived. Cancellation was requested and accepted.
  Do not claim the complete Windows job or overall run passed; inspect final status on resume.

## Local runner / environment

- Repository-scoped ephemeral runner: nemu-p0-01-ephemeral, labels self-hosted/Windows/X64/nemu.
  It was busy/online at cancellation request and should unregister after the job ends.
- Runner 2.338.0: .runner/runtime; portable PowerShell 7.6.6: .runner/pwsh.
  Archives were SHA-256 verified. Entire .runner/ is gitignored; never stage its credentials or logs wholesale.
- Listener launched hidden; latest logs: .runner/listener-2.log and listener-error-2.log.
  No Windows service or permanent startup task was installed.
- Persistent runner registration belongs to P0-05 part 2.
- Node 24.21.0; pnpm 12.11.2; Rust 1.99.0; Python 3.12.14;
  uv 0.12.15; .NET 10.0.401; Codex CLI 0.162.1.

## Remaining / exact resume sequence

1. Read this checkpoint, AGENTS.md, tasks/README.md, P0-01 in tasks/P0-foundation.md,
   docs/ARCHITECTURE.md, docs/CONTRACTS.md, docs/echo-core/README.md, and the relevant local skills.
2. Inspect git status/HEAD and PR #1. Preserve any new changes. Fetch main and check divergence.
3. Inspect run 38439576599 and runner registration/process status. Confirm cancellation/automatic unregister.
4. Only after resume authorization: if no runner is online, re-register .runner/runtime with a fresh
   repository registration token using config.cmd --unattended --url https://github.com/nanake-lab/nemu
   --name nemu-p0-01-ephemeral --labels nemu --work _work --ephemeral --disableupdate.
   Obtain the token via gh api POST repos/nanake-lab/nemu/actions/runners/registration-token;
   hold it in a local variable, never print or commit it. Add .runner/pwsh to the listener PATH.
   Start bin/Runner.Listener.exe run --startuptype manual hidden, with fresh redirected log names.
5. Rerun cancelled Windows CI via gh run rerun 38439576599 --failed, or trigger appropriate PR
   validation for the final HEAD. Do not count cancellation/skips as acceptance. Fix any real failure
   without relaxing assertions or modifying existing visual baselines without approval.
6. Remaining optional robustness review: smoke creates its temporary directory before electron.launch;
   launch failure happens before its current try/finally. Consider ensuring cleanup on launch failure
   and when application.close throws. This is a known follow-up, not an implemented fix.
7. Append final acceptance/CI links and actual limitations to REPORT.md. The report's initial
   pending-CI wording is historical; no final completion has been claimed. Update PR description.
   All checks for current implementation are recorded in final-validation; don't repeat unaffected
   suites without a change, failure, or unresolved concern.
8. Commit/push final evidence; verify applicable CI on the final head and the AGENTS.md merge
   conditions. Do not merge during this pause. Stay within P0-01 unless separately directed.

P0-01 remains incomplete until the full remote Windows job is verified green and final acceptance
is recorded. No active architectural decision, missing prototype, or credential blocker remains.
