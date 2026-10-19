# P0-08 Codex app-server spike

This directory contains an experimental, reproducible probe for P0-08. It uses synthetic multilingual inputs only. It does not define a canonical SemanticCandidate contract or modify runtime configuration.

## Requirements

- Windows PowerShell.
- The Codex CLI version recorded by `node spikes/codex-app-server/src/hash-generated.mjs`.
- A dedicated, isolated `CODEX_HOME` for this spike. Do not point the probe at a normal Codex profile or copy credentials into the repository.
- For remote measurements, the owner must complete the account sign-in flow in that isolated profile. The probe never enables account security settings or accepts approval requests.
- For the local carrier comparison, P0-07 must first select and document the carrier. Before any GPU measurement, follow the runner and quiet-window checks in `AGENTS.md` section 9.

## Prepare

From the repository root:

```powershell
$probeHome = Join-Path $env:LOCALAPPDATA 'Nemu\probes\P0-08\CODEX_HOME'
New-Item -ItemType Directory -Force -Path $probeHome | Out-Null
$env:CODEX_HOME = $probeHome
codex app-server generate-ts --experimental --out spikes/codex-app-server/generated
node spikes/codex-app-server/src/hash-generated.mjs
node spikes/codex-app-server/src/prepare-fixtures.mjs
```

`hash-generated.mjs` writes a candidate manifest under `drafts/`. It records the CLI version, generated file count, and a deterministic SHA-256 over sorted relative paths and file hashes. It does not update `config/models.lock.json`.

Sign in only through the normal Codex CLI flow after confirming that account settings permit it. The observed account requires device-code sign-in to be enabled by the account owner; this spike does not change that setting. Check with `codex login status` using the isolated `CODEX_HOME` before running remote measurements.

## Measurements

```powershell
node spikes/codex-app-server/src/probe.mjs catalog --code-home $probeHome
node spikes/codex-app-server/src/probe.mjs speech-all --code-home $probeHome --model <catalog-model-id>
node spikes/codex-app-server/src/probe.mjs irl --code-home $probeHome --model <catalog-model-id> --runs 200
node spikes/codex-app-server/src/probe.mjs controls --code-home $probeHome --model <catalog-model-id>
```

`speech-all` performs exactly 50 short responses in each language (150 total) and records account quota snapshots before the batch, after the first 100, and after all 150. `irl` performs 200 structured-output calls and records the same quota boundaries after calls 100 and 200. These snapshots are account-level observations; they are not a billing estimate.

Raw JSONL records and summaries are written to `reports/P0-08/`. Records retain synthetic case identifiers, latency, status, output length, schema validity, and token usage when available. Response text, account identifiers, authentication data, and raw stderr are not written.

Speech and IRL threads use an empty temporary working directory, `dynamicTools: []`, disabled shell/app/plugin/web-search/multi-agent features, a read-only sandbox, and no network. The approval-control threads enable shell execution only for the isolated temporary working directory; they remain read-only, have no network, and the client declines every approval request. Runtime behavior still needs confirmation from the authenticated control probe.

## Current blockers

See `reports/P0-08/BLOCKED.md`. No remote measurements are recorded until the isolated profile is authenticated. No local carrier measurement is recorded until P0-07 selects the carrier and the runner/GPU preflight passes.

## Scope

Generated protocol types, synthetic fixtures, and candidate drafts here are spike artifacts. `config/`, `packages/contracts/`, `core/contracts/`, and canonical Echo Core types are outside this task's authorized directory and remain untouched.
