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

## Full probe entry point (2026-10-17)

Once weights have verified successfully and the runner is offline, reserve a quiet
GPU window. The following command implements the matrix followed by the separate
informational direction study; use a fresh output directory:

```powershell
$env:HF_HUB_OFFLINE = '1'
$env:TRANSFORMERS_OFFLINE = '1'
uv run --project workers/model-adapter/probe --locked python workers/model-adapter/probe/run_all.py --output reports/P0-07/run-<new-id>
```

`run_matrix.py` measures three 4B-class candidates in bf16/int8/int4 and the 8B
comparison in int4, sequentially. Actual TTS Base weights stay resident while the
existing stage renders. No audio voice is chosen. Each configuration has a separate
process and evidence directory, including failures. Latency uses one excluded
warmup and twenty fresh-KV requests. `--fixed-twenty` asks the model to reproduce a
specified twenty-Han-character sentence; token generation is still autoregressive,
not teacher-forced. Every actual output and count is retained. This controlled
copying fixture does not establish free-form response quality. Time limits are
checked between forwards, so a single slow forward can overshoot the limit.

`direction_probe.py` uses int4 and three synthetic calm/excited calibration pairs.
Their mean residual difference becomes a fixed direction of norm at most one;
only the first of eight coefficient slots is active, with a predeclared amplitude
of one. Ten matched seeds, each with zero and positive modulation, produce twenty
outputs per language. Generation prompts stay identical between conditions. This
small exploratory study is not a functional certificate or a claim that the
synthetic contrast is a true Echo Core state coordinate. No online optimizer runs.

The path probe's identity U and this exploratory U are distinct recorded fixtures,
not a selected production carrier. Production tolerance, carrier freeze and full
acceptance require the report's remaining checks. See REPORT.md for what actually
ran; availability of this entry point alone is not an acceptance result.

EOS stopping follows the full list in each pinned generation_config.json. Phi v1
omitted one EOS and is retained as defective-stop evidence; v2 reruns only that
direction case. Matrix v1 contains no occurrences of the omitted token, but remains
historical evidence from the earlier script, not a claimed rerun of this version.

## Int4 optimization experiments (2026-10-17)

User authorized int4-only optimization with unchanged acceptance thresholds.
Existing default eager/reference behavior is retained. New explicit switches:

- `--attention sdpa`: use the installed Transformers/PyTorch SDPA integration.
- `--lean`: cache Ua only inside one request and emit only last-position logits.
  Ua is computed on the first measured forward, cleared at end/begin, never
  carried across requests. Zero coefficients still execute the explicit addition.
- `run_matrix.py --precision int4` runs only the three candidates and 8B control.
  Every selected variant runs its own path probe before concurrent measurement.
- `run_all.py --int4-only --attention sdpa --lean` also runs three-language
  observations using those same switches. No acceptance is implied by availability.

Fresh output paths are required. The fixture, TTS Base resident load, rendering,
seed, greedy decoding, one warmup + 20 repeats, max96 tokens/30s and600ms remain
unchanged. Examples (only in a notified quiet GPU window with runner offline):

```powershell
python workers/model-adapter/probe/profile_int4.py --model Qwen/Qwen3-4B --output reports/P0-07/profile-eager-NEW
python workers/model-adapter/probe/run_matrix.py --precision int4 --output reports/P0-07/int4-reference-NEW
python workers/model-adapter/probe/run_matrix.py --precision int4 --attention sdpa --output reports/P0-07/int4-sdpa-NEW
python workers/model-adapter/probe/run_matrix.py --precision int4 --attention sdpa --lean --output reports/P0-07/int4-sdpa-lean-NEW
```

`profile_int4.py` is a separate solo-carrier diagnostic: one warmup then one
prefill plus eight decode steps, raw CPU/CUDA events and Chrome trace. It is not a
sentence benchmark or a concurrent-load acceptance run. Profiler instrumentation
perturbs timing; CPU/device event sums must not be added as if they were elapsed
wall time. Actual fused-kernel use must be established from the trace, not guessed
from `sdpa` alone. No new dependency or compiler backend is introduced.

### 原生 RMSNorm 实验（仅 int4）

`--native-rms` 可与 `run_matrix.py --precision int4`、`run_all.py --int4-only` 组合；默认关闭。仅替换载体的 Qwen3RMSNorm/Phi3RMSNorm，不替换 TTS。调用现有 PyTorch 2.10 的 `torch.nn.functional.rms_norm(..., weight=None, eps=原值)`，结果转回输入 dtype 后再乘原 learned weight，保留原实现的中间舍入位置；无新依赖，无 forward hook，不改残差包装/层号/掩码。

归约内核改变可能带来数值差异，不能假定逐位一致。路径 JSON 同时记录 `native_rms_vs_original_*`（内核替换对原始模型的差异）以及原有零调制、注入误差、缓存失效字段；零调制对照使用相同内核的未包装模型。所有数字为实测，不能自行据此冻结生产容差。正式计时保留同一并载协议，profiler 只用于诊断。

依据：PyTorch v2.10.0 官方源码 `aten/src/ATen/native/layer_norm.cpp` 的 rms_norm_symint 调用 `_fused_rms_norm`，本机已安装 CUDA dispatch；实际是否使用融合内核仍以轨迹核对。
