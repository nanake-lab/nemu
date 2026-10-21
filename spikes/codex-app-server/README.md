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
node spikes/codex-app-server/src/probe.mjs smoke --code-home $probeHome
node spikes/codex-app-server/src/probe.mjs speech-all --code-home $probeHome --model <catalog-model-id>
node spikes/codex-app-server/src/probe.mjs irl --code-home $probeHome --model <catalog-model-id> --runs 200
node spikes/codex-app-server/src/probe.mjs controls --code-home $probeHome --model <catalog-model-id>
```

`speech-all` performs exactly 50 short responses in each language (150 total) and records account quota snapshots before the batch, after the first 100, and after all 150. `irl` performs 200 structured-output calls and records the same quota boundaries after calls 100 and 200. These snapshots are account-level observations; they are not a billing estimate.

`smoke` initializes app-server and starts an ephemeral thread without starting a model turn. It verifies local protocol setup only; it does not replace authenticated inference measurements.

Raw JSONL records and summaries are written to `reports/P0-08/`. Records retain synthetic case identifiers, latency, status, output length, schema validity, and token usage when available. Response text, account identifiers, authentication data, and raw stderr are not written.

Speech and IRL threads use an empty temporary working directory, `dynamicTools: []`, disabled shell/app/plugin/web-search/multi-agent features, a read-only sandbox, and no network. The approval-control threads enable shell execution only for the isolated temporary working directory; they remain read-only, have no network, and the client declines every approval request. Runtime behavior still needs confirmation from the authenticated control probe.

## Current blockers

See `reports/P0-08/BLOCKED.md`. No remote measurements are recorded until the isolated profile is authenticated. No local carrier measurement is recorded until P0-07 selects the carrier and the runner/GPU preflight passes.

## Second phase (2026-10-22)

PR #50 continues the partially complete PR #29 from a new branch. The isolated profile has now authenticated with ChatGPT. The current model catalog includes all three ADR 0004 tiers. The user explicitly selected `gpt-6-luna` with `max` reasoning for this phase, superseding the card's low-effort setting. Preflight calls remain separate from the formal 150 speech and 200 IRL calls.

Second-phase records are under `reports/P0-08/part2/`. A source snapshot and hashes bind the speech run to its actual code. JSONL records retain failures; structured candidates from synthetic inputs are separate evidence artifacts. Nothing reads rehearsal or life storage.

The original schema used `oneOf`/`allOf`, which the strict output-schema endpoint rejected. `src/prepare-wire-schema.mjs` produces an endpoint-compatible wire schema by flattening object composition and using a disjoint `anyOf` for node kinds. Its array items remain integers; the original tuple, uniqueness and source-binding constraints are still enforced after decoding. Both local and remote use the same wire schema, and the original schema remains the validation target. Schema validity, source binding and empty graphs are reported separately. Schema validity alone does not establish semantic accuracy.

Local reproduction (only during the user-arranged GPU quiet window):

```powershell
uv sync --project spikes/codex-app-server --locked
# Reuse the pinned P0-07 CUDA environment read-only through a local .pth file.
# The .pth is ignored and contains only the path to that environment's site-packages.
$carrierSite = 'D:\Dev\nemu-p007\workers\model-adapter\probe\.venv\Lib\site-packages'
$carrierSite | Set-Content -Encoding utf8 spikes/codex-app-server/.venv/Lib/site-packages/p007-carrier.pth
spikes/codex-app-server/.venv/Scripts/python.exe spikes/codex-app-server/src/local_irl.py --model-root <pinned-model-cache> --output <new-evidence-directory> --quiet-window-confirmed
```

`local_irl.py` refuses an existing output directory, checks the process inventory before loading the model and before every sample, uses the selected carrier's pinned numerical versions, and generates with `m=0` without a hook (the zero reference in ADR 0012 section 2.1). It performs one excluded warmup followed by 200 cases, greedy schema-constrained decoding, and a 2048-token cap. Completion timing includes construction of the per-request constraint parser and generation; tokenizer-data construction and model loading are outside request timing. The constraint decoder is `lm-format-enforcer 0.11.3` (MIT); JSON Schema validation uses `jsonschema 4.26.0` (MIT). Existing P0-07 Torch/Transformers/bitsandbytes versions are reused without changing that environment.

Focused CPU checks:

```powershell
node --test spikes/codex-app-server/src/validation.test.mjs
spikes/codex-app-server/.venv/Scripts/python.exe -m unittest discover -s spikes/codex-app-server/src -p test_local_irl.py
spikes/codex-app-server/.venv/Scripts/python.exe -m ruff check spikes/codex-app-server/src/local_irl.py spikes/codex-app-server/src/test_local_irl.py
spikes/codex-app-server/.venv/Scripts/python.exe -m mypy --strict --follow-imports silent spikes/codex-app-server/src/local_irl.py spikes/codex-app-server/src/test_local_irl.py
```

## Scope

Generated protocol types, synthetic fixtures, and candidate drafts here are spike artifacts. `config/`, `packages/contracts/`, `core/contracts/`, and canonical Echo Core types are outside this task's authorized directory and remain untouched.

## Connection failure and continuation

Each completed ephemeral measurement thread is unsubscribed after its latency is captured. Connection-level RPC or turn timeouts stop a batch and preserve a partial summary. `speech` and `irl` accept a one-based `--start-index` to continue later fixtures; the raw row index remains the original fixture index. Supplemental attempts use unique run IDs and never overwrite failed raw records. The Japanese interruption and supplemental segment manifest in `reports/P0-08/part2/remote-speech-all-summary.json` must be included in analysis. Completed-only latency percentiles exclude censored timeouts; failure counts and supplemental counts remain visible.

The first connection stalled after 127 successful replies; the cause is unconfirmed. Unsubscribing completed threads bounds live server resources but does not establish the cause. Speech source snapshots preserve the version actually loaded by each batch. Formal IRL source and hash are separate. No model/effort fallback is configured or accepted as equivalent evidence.
