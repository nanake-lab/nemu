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
