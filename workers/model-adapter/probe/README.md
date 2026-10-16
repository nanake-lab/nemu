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

The explicit forward wrapper, path checks, quantization/concurrent-load matrix,
and three-language direction checks are not yet implemented or measured.
These preparation commands are not the final one-command measurement suite.

The pinned Transformers 4.57.3 `Qwen3DecoderLayer.forward` performs the attention
residual addition immediately before `post_attention_layernorm`. The probe will
use that explicit location, not a dynamic module hook. Layer, rank, mask, U and
numeric settings must be recorded before measurements. Model parameters remain
frozen. Cross-request KV reuse must never survive a modulation change.

Do not run local Electron, Playwright, visual tests or concurrent GPU/render
measurements while the Windows CI runner is online. Coordinate the runner
before measurements. Every PR triggers a Windows CI job and needs the runner;
main also triggers CI after merge.
