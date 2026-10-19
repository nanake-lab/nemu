# P0-08 progress plan

The Codex queue selected P0-08 after confirming P0-01 is merged, P0-08 has no existing claim, and its `spikes/codex-app-server/` scope does not overlap active P0-07 or P1-02 work.

## Execution boundary

- Keep implementation and reproducible probe inputs under `spikes/codex-app-server/`.
- Keep results and raw evidence under `reports/P0-08/`.
- P0-07 is still active. The local IRL comparison that requires its selected carrier is pending that result; remote app-server measurements can proceed.
- The task card also mentions `config/models.lock.json`, `config/helpers.json`, and a deployment profile, all outside its directory scope. `config/helpers.json` additionally requires an ADR and user approval. No such config will be changed in this claim commit.
- Use a dedicated `CODEX_HOME`; never copy or print credentials into the repository.

## Evidence status

No P0-08 measurements have run yet. This file only records the claim and execution boundaries.

## Progress update — 2026-10-19

- Re-read `AGENTS.md` section 9 in both the primary checkout and this worktree. The GPU phase remains gated on a `Runner.Listener` / `Runner.Worker` check and an announced quiet window; no GPU work has started.
- Generated app-server TypeScript with Codex CLI 0.162.1 (888 files) and recorded a deterministic candidate manifest under `spikes/codex-app-server/drafts/`.
- Generated and parsed the 200-case synthetic multilingual fixture set (zh 67, en 67, ja 66). No user rehearsal or formal storage was opened.
- Built the measurement commands and candidate configuration templates. Remote model/usage access remains blocked by isolated-profile sign-in; local carrier measurement remains blocked on P0-07.
- Configuration writes remain limited to candidate drafts because the card directory excludes `config/`; see `tasks/requests/P0-08-config-scope.md`.
