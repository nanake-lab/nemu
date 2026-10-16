# P0-07 carrier probe (in progress)

This directory is isolated from the production model-adapter environment.
`manifest.json` lists candidates, not a selected carrier. No measurement or
acceptance result is implied by installing dependencies or downloading assets.

## Preparation

From the repository root in PowerShell:

```powershell
uv sync --project workers/model-adapter/probe --locked
uv run --project workers/model-adapter/probe --locked python workers/model-adapter/probe/prepare.py
```

The second command downloads configuration, tokenizer and license files at the
manifest's immutable revisions. Add `--weights` to download safetensors too;
`--model Qwen/Qwen3-4B` limits preparation to one manifest entry. Downloads and
SHA-256 inventories live under ignored `.cache/`. Download preparation does not
import torch, start a renderer, or allocate GPU memory.

## Measurement status

The explicit forward wrapper and path checks are implemented and checked on a
small random CPU Qwen3 fixture. Hardware stage rendering has been checked separately.
Pretrained-model checks, the quantization/concurrent-load matrix, and three-language
direction checks remain pending. These commands are not the final one-command
measurement suite.

The pinned Transformers 4.57.3 `Qwen3DecoderLayer.forward` performs the attention
residual addition immediately before `post_attention_layernorm`. The probe will
use that explicit location, not a dynamic module hook. Layer, rank, mask, U and
numeric settings must be recorded before measurements. Model parameters remain
frozen. Cross-request KV reuse must never survive a modulation change.

Do not run local Electron, Playwright, visual tests or concurrent GPU/render
measurements while the Windows CI runner is online. Coordinate the runner
before measurements. Every PR triggers a Windows CI job and needs the runner;
main also triggers CI after merge.

## Verified preparation commands

```powershell
uv run --project workers/model-adapter/probe --locked pytest workers/model-adapter/probe/test_injection.py -q
uv run --project workers/model-adapter/probe --locked python workers/model-adapter/probe/path_probe.py --tiny-cpu --output <new-result.json>
```

After complete weight verification, omit `--tiny-cpu` and specify `--model` plus
`--precision bf16|int8|int4` for the real GPU path measurement. The script refuses
GPU execution while the Windows runner is present. It does not itself establish
concurrent-load performance, language quality, or production numerical tolerance.

If the Hugging Face client encounters TLS stream resets, use the system-curl
fallback in `download_weights.py`. It uses immutable revision metadata, bounded
8 MiB ranges, four workers, Content-Range/length checks and final LFS SHA-256.
Set HTTP_PROXY/HTTPS_PROXY only in the calling process if needed. No TLS checks
are disabled. Chunk files and response headers remain in ignored cache, including
partial downloads; never stage that directory in Git.

`render_load.mjs <new-ready-json> <stop-file>` launches the existing stage on port
5174, checks the physical RTX 3080 renderer and advancing frames, then stays alive
until the stop-file exists. The caller must use fresh output paths and create the
stop-file in a finally block. It is a load generator, not a visual-regression tool.
The probe does not modify the stage, visual goldens or appearance.

The forward ordering in injection.py is adapted from Transformers 4.57.3 under
Apache-2.0; copyright attribution is in that file and the license is retained in
licenses/transformers-Apache-2.0.txt.
