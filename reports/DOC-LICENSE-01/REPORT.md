仓库从此采用 Apache-2.0，别人可以使用、修改，也可以商用，但必须保留版权和许可声明；`content` 不在此列。
本卡新增官方 `LICENSE`、`NOTICE`，在 `README.md` 新增用户确认的英文简介与中英双语「License」一节，并完成全仓 16,439 个已追踪文件的第三方材料与许可排查。
需要用户审查并合并本 PR（含 `README.md` 改动与仓库许可采用的批准），并由 Claude 处理下方标红的 4 项第三方许可登记与再分发边界问题。

档位：轻档（交付线，文档与许可文件）；仅新建 `LICENSE`、`NOTICE`、任务请求与本报告，并在 `README.md` 新增英文简介与「License」一节，不改代码、契约、规格、任务卡、git 历史或 `content` 子模块。

# DOC-LICENSE-01 仓库采用 Apache-2.0 — 报告

## 当前状态

已完成全部文档与排查交付，PR [#254](https://github.com/nanake-lab/nemu/pull/254) 保持 draft 并带 `needs-user` 标签，等待审查、Claude 处理标红项及用户合并。

本轮基线：`origin/main` `8ed77319b2c4bc20a79a38f4b2e15797ebc18e78`。执行期间用户于 2026-11-21 明确确认追加范围：将一段英文简介一并加在 `README.md` 开头（紧接 `# Nemu` 之后），已补充记录于 [tasks/requests/DOC-LICENSE-01.md](../../tasks/requests/DOC-LICENSE-01.md)。

## 交付物

- [LICENSE](../../LICENSE)：Apache License, Version 2.0 官方全文，自 `https://www.apache.org/licenses/LICENSE-2.0.txt` 原样下载（11,358 字节，LF 换行，SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30` 与官方逐字节一致）。
- [NOTICE](../../NOTICE)：按任务卡原文写入版权声明、`content` 子模块边界与第三方资产许可提示（321 字节，LF 换行，SHA-256 `9bf0a28a17e4880226db90440052ec44f37b2fb0bc4e7983322a627dd9cc5c59`）。
- [README.md](../../README.md)：按用户 2026-11-21 确认在 `# Nemu` 后新增英文简介；按卡要求在文末新增中英双语 `## License` 一节，其余既有正文未改。
- [tasks/requests/DOC-LICENSE-01.md](../../tasks/requests/DOC-LICENSE-01.md)：登记本卡原文与 2026-11-21 英文简介追加指令。
- [reports/DOC-LICENSE-01/REPORT.md](REPORT.md)：本报告（含全仓第三方材料与许可排查表）。

## 第三方材料排查：标红项（请 Claude 处理）

> [!CAUTION]
> **🔴 以下 4 项命中任务卡「标红」条件（没有登记许可证的第三方文件 / 许可证不允许再分发或未审查再分发的文件）——请 Claude 处理：**
>
> 1. **🔴 【标红｜请 Claude 处理】没有登记许可证的第三方文件（已入仓）：`spikes/stage-overlay/src/renderer/public/VRM1_Constraint_Twist_Sample.vrm`**
>    - 与 `apps/stage/public/sample.vrm` 逐字节一致（10,776,032 字节，SHA-256 `12c2b97e95e700783a6a550dc0eee2d7880aeedccef9ae67bc4c5a2f0f2631a2`），来源为 `pixiv/three-vrm` 示例模型（`(c) 2022 pixiv Inc.`）。虽内嵌 VRM Public License 1.0（`https://vrm.dev/licenses/1.0/`，`allowRedistribution: true`），但 `spikes/stage-overlay/` 下无 `manifest.json`，全仓任何 `manifest.json` 亦未登记该路径。
> 2. **🔴 【标红｜请 Claude 处理】没有登记许可证的第三方文件（已入仓）：`spikes/codex-app-server/generated/`（888 个 `.ts` 文件）与 `executors/codex-bridge/src/base/generated/`（46 个 `.ts` 文件）**
>    - 均由 OpenAI `codex-cli 0.162.1` 执行 `codex app-server generate-ts --experimental` 生成（文件头标注 `ts-rs` 生成）。`executors/codex-bridge/src/base/generated/MANIFEST.json` 仅记录生成器与文件 SHA-256，缺少 `source`/`license`/`redistributable` 字段；`spikes/codex-app-server/generated/` 无 `manifest.json`（`drafts/cli-protocol-candidate.json` 亦未登记许可证）。上游 `openai/codex` 为 Apache-2.0，但本仓库未核验并登记对应版本 `0.162.1` 生成产物的许可证。
> 3. **🔴 【标红｜请 Claude 处理】许可证不允许再分发的文件（未入仓，仅清单引用）：`packages/voice/src/asr/manifest.json`（及 `config/models.lock.json` `asr` 节）中的 `model.int8.onnx` 与 `tokens.txt`**
>    - 来自 `csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17`（源自 `FunAudioLLM/SenseVoiceSmall`），许可证登记为 `custom:FunASR-Model-1.1`，明确标注 `"redistributable": false`。两文件未提交入仓（仅用户本地下载，备注称 2026-10-21 已决定仅开发使用、P7 正式发布前重评），但因命中「许可证不允许再分发」，在此标红提请确认与跟踪。
> 4. **🔴 【标红｜请 Claude 处理】再分发许可未审查的外部归档（未入仓，仅清单引用）：`spikes/carrier-engine/manifest.json` 中的 `llama.cpp-b11669`（Windows x64 CUDA 12.4 二进制归档）**
>    - 许可证登记为 `MIT (source); LLVM OpenMP and NVIDIA terms also apply to binary runtime bundle`，明确标注 `"redistributionPermittedByLicense": null`、`"productionDistributionApproved": false`，备注完整二进制归档再分发未审查。归档未提交入仓，发布或打包前不可随仓库或应用分发。

### GPL / AGPL 专项核查结果

- **仓库内已追踪源码与文件**：**未发现任何 GPL 或 AGPL 代码文件**（0 处）。
- **提及 GPL/AGPL 的文档与工具说明**：
  - `reports/P0-00/borrow-list.md` 仅记录对 `shinyflvre/Mate-Engine`（AGPL-3.0）的黑盒功能观察，无源码接触或复制。
  - `tools/asset-pipeline/` 是面向外部开发工具 **Blender 5.2.2 LTS（GPL-3.0-or-later）** 的无头 Python 脚本（经 `blender -b --python` 调用）；Blender 本体未入仓、不打包进应用，且 Apache-2.0 与 GPL-3.0 兼容。
  - `apps/panel/src/legacy/Composer.tsx` 头部注明改编自用户本人拥有的 React 原型 `nemu-agent`（非第三方）。
- **各包直接依赖核对**：已检查全仓全部 `package.json`、`core/Cargo.toml`、`core/crates/*/Cargo.toml`、`*/pyproject.toml` 直接依赖清单，未见 GPL/AGPL 应用运行时依赖（注：未逐一展开核对锁文件中全部传递依赖的许可元数据）。

## 第三方材料排查总表

排查方法：对 `git ls-files` 全部 16,439 个已追踪文件按扩展名、资产/二进制文件、`*manifest*`/`*license*`/`*notice*`/`*copying*` 文件名，以及 `Copyright`、`SPDX-License-Identifier`、`Permission is hereby granted`、`GPL`/`AGPL`/`LGPL`、`License` 等关键词全仓检索，并逐一核对全部重点目录（`apps/stage/public/motions/`、`core/crates/fact-view/models/`、`packages/voice/`、`spikes/`、`tools/`）及 `config/`、`content-template/`、`workers/`、`services/`、`executors/`、`reports/`。

### 一、已提交进本仓库的第三方文件（随本仓库源码树分发）

| 路径 | 来源 | 许可证 | 能否随本仓库分发 | 备注 / 排查结论 |
| --- | --- | --- | --- | --- |
| `apps/stage/public/sample.vrm`（10.78 MB） | `pixiv/three-vrm` 示例 `VRM1_Constraint_Twist_Sample`（`(c) 2022 pixiv Inc.`） | VRM Public License 1.0（内嵌 `https://vrm.dev/licenses/1.0/`，`allowRedistribution: true`，`modification: allowModificationRedistribution`，`commercialUsage: corporation`，`creditNotation: unnecessary`） | 是（沿用自身 VRM 1.0 许可，非 Apache-2.0） | 已在 `tools/asset-pipeline/tests/fixtures/manifest.json` 跨目录登记；`apps/stage/public/` 同级无 `manifest.json`。`apps/stage/tests/visual/__golden__/*.png`（20 张）为其视觉基准截图 |
| **🔴 `spikes/stage-overlay/src/renderer/public/VRM1_Constraint_Twist_Sample.vrm`**（10.78 MB） | 同上（与 `apps/stage/public/sample.vrm` SHA-256 逐字节一致） | 内嵌 VRM Public License 1.0；**仓库内无任何 `manifest.json` 登记** | 许可本身允许，但仓库缺少清单登记 | **🔴 【标红｜请 Claude 处理】** 没有在 `manifest.json` 登记的第三方文件，且与 `sample.vrm` 重复存储 |
| **🔴 `spikes/codex-app-server/generated/`**（888 个 `.ts` 文件） | OpenAI `codex-cli 0.162.1`（`codex app-server generate-ts --experimental`，`ts-rs` 生成） | **仓库内未登记许可证**（无 `manifest.json`，`drafts/cli-protocol-candidate.json` 无许可字段） | 待核实登记（上游 `openai/codex` 为 Apache-2.0，本仓未登记） | **🔴 【标红｜请 Claude 处理】** 没有登记许可证的第三方生成文件 |
| **🔴 `executors/codex-bridge/src/base/generated/`**（46 个 `.ts` 文件 + `MANIFEST.json`） | OpenAI `codex-cli 0.162.1`（`codex app-server generate-ts --experimental`，`ts-rs` 生成） | **仓库内未登记许可证**（`MANIFEST.json` 仅含 `generator` 与文件 SHA-256，无 `license`/`redistributable` 字段） | 待核实登记（上游 `openai/codex` 为 Apache-2.0，本仓未登记） | **🔴 【标红｜请 Claude 处理】** 没有登记许可证的第三方生成文件 |
| `workers/model-adapter/probe/injection.py`（及 `licenses/transformers-Apache-2.0.txt`、`Phi-4-mini-instruct-MIT.txt`） | 改编自 Hugging Face `transformers 4.57.3` 的 `Qwen3DecoderLayer`（Qwen/Alibaba/HF 2025）与 `Phi3`（Microsoft/HF 2024） | `Apache-2.0`（配套保留 `transformers-Apache-2.0.txt` 及 Phi-4 的 MIT 许可原文） | 是（已保留版权、许可与修改声明） | 合规；`workers/model-adapter/probe/README.md` 与代码头均有完整归属声明 |
| `packages/voice/src/asr/public-data.lock.json`（90 条评测文本元数据，不含音频） | Hugging Face `google/fleurs`（revision `168de341b3db...`，`cmn_hans_cn` / `en_us` / `ja_jp`） | `CC-BY-4.0`（每条记录均显式标注 `source`、`license`、`source_url`） | 是（沿用 CC-BY-4.0，已保留署名与来源；音频 `.wav` 未入仓） | 合规；注意所含转写文本沿用 CC-BY-4.0 而非 Apache-2.0 |
| `reports/P0-10/preparation/upstream/`（`qwen3.cpp`、`llama-adapter.cpp`、`llama.h`、`LICENSE`） | `https://github.com/ggml-org/llama.cpp`（commit `1a0b56ce...`，静态源码证据） | `MIT`（`Copyright (c) 2023-2026 The ggml authors`，同目录附完整 `LICENSE` 及 `upstream-manifest.json`） | 是（保留 MIT 版权与许可文本） | 合规 |
| `reports/P0-10/full-preparation-20261026/upstream/`（`qwen3.cpp`、`llama-adapter.cpp`、`LICENSE`） | `https://github.com/ggml-org/llama.cpp`（tag `b11669`，commit `620d2225...`） | `MIT`（同目录附完整 `LICENSE`，并在 `source-inventory.json` 登记） | 是（保留 MIT 版权与许可文本） | 合规 |
| `reports/P0-12/preparation/upstream/`（`llama-memory-hybrid.cpp`、`llama-memory-recurrent.cpp`、`qwen35.cpp`、`LICENSE`） | `https://github.com/ggml-org/llama.cpp`（tag `b11669`，commit `620d2225...`） | `MIT`（同目录附完整 `LICENSE`，并在 `upstream-inventory.json` 登记） | 是（保留 MIT 版权与许可文本） | 合规 |
| `reports/P0-12/preparation/model/`（`chat_template.jinja`、`config.json`、`LICENSE`） | `https://huggingface.co/Qwen/Qwen3.5-4B`（revision `851bf6e8...`） | `Apache-2.0`（`Copyright 2026 Alibaba Cloud`，同目录附完整 `LICENSE`，并在 `reports/P0-12/manifest.json` 登记） | 是（Apache-2.0，保留版权与许可文本） | 合规 |
| `reports/P6-08/evidence/koffi-3.3.3-LICENSE.txt` | `koffi 3.3.3` npm 包原文（`Copyright (C) 2026 Niels Martignène`） | `MIT`（登记于 `reports/P6-08/FFI-AND-LICENSE.md`） | 是（纯许可文本副本） | 合规 |

### 二、仓库内 `manifest.json` 及锁文件登记的外部下载资产（实体文件未提交进本仓库）

| 清单路径 | 登记条目与来源 | 许可证 | 能否随本仓库分发 | 备注 / 排查结论 |
| --- | --- | --- | --- | --- |
| `apps/stage/public/motions/manifest.json` | 5 个占位动作条目（`idle_stand.vrma`、`idle_relaxed.vrma`、`wave.vrma`、`nod.vrma`、`fidget_stretch.vrma`）；**无对应 `.vrma` 文件入仓** | 未填写（清单仅含 `id`/`name`/`file`/`category`，缺 `source`/`license`/`redistributable`） | 不适用（无实体文件入仓） | 目前为空占位清单；未来入仓真实 `.vrma` 前必须补齐许可与再分发字段 |
| `core/crates/fact-view/models/manifest.json` | `intfloat/multilingual-e5-small`（revision `614241f6...`，ONNX 权重及分词器文件） | `MIT` | 是（`redistributable: true`；权重未入仓，仅本地下载） | 合规 |
| **🔴 `packages/voice/src/asr/manifest.json`**（同 `config/models.lock.json` `asr` 节） | `model.int8.onnx`、`tokens.txt` — `csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17`（revision `2365baea...`） | `custom:FunASR-Model-1.1` | **否**（`redistributable: false`；权重未入仓，仅用户本地下载） | **🔴 【标红｜请 Claude 处理】** 许可证不允许再分发；按 2026-10-21 决定仅开发使用，P7 正式发布前重评 |
| `packages/voice/src/asr/manifest.json`（同 `config/models.lock.json` `asr` 节） | `silero_vad.onnx` — `https://github.com/snakers4/silero-vad`（revision `1e261b03...`） | `MIT` | 是（`redistributable: true`，保留 MIT 声明；未入仓） | 合规 |
| `packages/voice/src/asr/alignment/manifest.json` | `Qwen/Qwen3-ForcedAligner-0.6B`（revision `c7cbfc20...`）及 `qwen-asr 0.0.6` | `Apache-2.0` | 是（`redistributable: true`；标注仅本地评测、不随应用打包；未入仓） | 合规 |
| `packages/voice/src/asr/alignment/ctc-candidates.manifest.json` | `facebook/wav2vec2-base-960h`、`jonatasgrosman/wav2vec2-large-xlsr-53-chinese-zh-cn`、`jonatasgrosman/wav2vec2-large-xlsr-53-japanese` | `Apache-2.0` | 是（`redistributable: true`；仅元数据预检，未下载权重） | 合规 |
| `spikes/carrier-engine/manifest.json` | `Qwen3-4B-Instruct-2507`（revision `cdbee75f...`） | `Apache-2.0` | 许可允许（`redistributionPermittedByLicense: true`，`productionDistributionApproved: false`；未入仓） | 合规 |
| **🔴 `spikes/carrier-engine/manifest.json`** | `llama.cpp-b11669`（Windows x64 CUDA 12.4 二进制归档，commit `620d2225...`） | `MIT (source); LLVM OpenMP and NVIDIA terms also apply to binary runtime bundle` | **未审查 / 不可直接分发**（`redistributionPermittedByLicense: null`，`productionDistributionApproved: false`；未入仓） | **🔴 【标红｜请 Claude 处理】** 含 NVIDIA/LLVM 二进制运行时条款未审查，仅限本地实验缓存 |
| `workers/model-adapter/probe/manifest.json`（同 `config/models.lock.json` `p0_07_probe` 节） | `Qwen/Qwen3-4B`、`Qwen/Qwen3-4B-Instruct-2507`、`Qwen/Qwen3-8B`、`Qwen/Qwen3-TTS-12Hz-0.6B-Base`（均为 `Apache-2.0`）；`microsoft/Phi-4-mini-instruct`（`MIT`） | `Apache-2.0` / `MIT` | 是（`redistributable: true`；权重均未入仓） | 合规 |
| `tools/asset-pipeline/tests/fixtures/manifest.json` | 指向 `../../../../apps/stage/public/sample.vrm`（见表一） | VRM Public License 1.0 | 是（`redistributable: true`） | 合规 |
| `content-template/nemu/manifest.json`、`content-template/nemu/motions/manifest.json`、`content-template/nemu/LICENSE.md` | 空模板清单（`[]`）与占位声明（`No asset license is granted by this placeholder`） | 占位未授权 | 不适用（无第三方资产） | 合规 |

### 三、其他已核查但不属第三方代码/模型的文件说明

- `apps/stage/public/sample-bible-ref.svg`：仓库原创 SVG 设定参考占位图（无第三方版权）。
- `reports/P0-12/inputs/images/*.png`（21 张）与 `reports/P0-12/inputs/images/manifest.json`：本仓库通过 Windows `System.Drawing` 生成的合成测试图（色块、几何与短句），其中 9 张文字图使用系统字体 `C:/Windows/Fonts/msyh.ttc`（微软雅黑）栅格化；仓库未包含 `.ttc` 字体文件本体，清单未填 `license`/`redistributable` 字段。
- `reports/P2-05/content-manifest-snapshot.json`：为 `content` 私有子模块中用户自有候选（`author: nanake`，`custom:Nemu private candidate; license pending P7`，`redistributable: false`）的清单快照，对应的 `.vrm` 文件位于独立的 `content` 子模块，不在主仓库中。

## 验收

| 检查项 | 命令 / 方法 | 结果 | 证据 |
| --- | --- | --- | --- |
| `LICENSE` 与官方文本逐字节及 SHA-256 一致 | 从 `https://www.apache.org/licenses/LICENSE-2.0.txt` 独立拉取并比对本地 `LICENSE` | **通过** | 官方与本地均为 11,358 字节、纯 LF、SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`，`Exact byte match: True` |
| `NOTICE` 内容与格式核对 | 逐字核对任务卡原文及字节哈希 | **通过** | 321 字节、纯 LF、SHA-256 `9bf0a28a17e4880226db90440052ec44f37b2fb0bc4e7983322a627dd9cc5c59` |
| `git diff --check` | `git diff --cached --check` / `git diff --check origin/main` | **通过** | 退出码 0，无尾随空白或格式错误 |
| `README.md` 全部链接校验 | 校验全部 67 处链接（18 个唯一相对路径 + 13 个外部 URL） | **通过（1 个既有外链探测受限）** | 18/18 相对路径（共 49 处）全部存在（含新增 `docs/echo-core/`、`LICENSE`、`NOTICE`）；12/13 外链返回 HTTP 200；既有外链 `https://openai.com/codex/` 在本机 CLI 探测时返回 TLS EOF（非本次改动引入） |
| 本报告相对链接校验 | 逐个确认本报告中相对路径存在 | **通过** | 6 处相对路径链接（5 个唯一文件：`LICENSE`、`NOTICE`、`README.md`、`tasks/requests/DOC-LICENSE-01.md`、`REPORT.md`）均存在 |
| 改动范围核对 | `git diff --name-only origin/main` | **通过** | 仅改/新建 5 个允许文件：`LICENSE`、`NOTICE`、`README.md`、`tasks/requests/DOC-LICENSE-01.md`、`reports/DOC-LICENSE-01/REPORT.md`；未碰 `content` 子模块与 git 历史 |
| 代码 / 构建 / GPU / Windows 验收 | 轻档文档卡，不适用 | `not_run`（不适用） | 见下方限制关于根目录文件触发 CI 的说明 |

## 新增依赖

无。

## 失败、阻塞与未完成项

- **阻塞项（等待用户与 Claude 处理）**：
  1. 本卡修改了既有 `README.md` 并为仓库引入正式开源许可证，按 `AGENTS.md` 第 2/6 节与卡要求需由用户审查后合并（`needs-user`）。
  2. 全仓排查发现 4 项标红的第三方材料登记/再分发边界问题（见上「标红项」），按卡要求不做改动，交由 Claude 处理。
- **已知限制**：
  - `README.md` 既有外部链接 `https://openai.com/codex/` 在本机 Python `urllib` 与 `curl` 探测中出现 TLS EOF / 连接失败，未能在命令行下取得 HTTP 200（可能为站点反爬/网络边缘限制；该链接为已有内容，本次未改）。
  - 排查覆盖全仓 16,439 个已追踪文件与全部直接依赖清单，未展开审计 `pnpm-lock.yaml`、`core/Cargo.lock`、`uv.lock` 中所有传递依赖的许可文本。
  - 注：`.github/workflows/ci.yml` 的 `paths-ignore` 与 `Decide scope` 未将根目录 `LICENSE`、`NOTICE` 归入纯文档跳过范围，因此本 PR 推送后可能会触发一次 Ubuntu CI 作业；本卡按目录限制未修改 `.github/workflows/ci.yml`。

## 待用户确认

- [ ] （阻塞）批准仓库采用 Apache License 2.0（`LICENSE`、`NOTICE`）及 `README.md` 的英文简介与「License」一节改动，并合并 PR #254（`needs-user`）。
- [ ] （阻塞 / 交 Claude 处理）处理报告中标红的 4 项第三方材料清单与再分发问题。

## 后续建议（不在本任务范围内）

- 由 Claude 决策上述 4 个标红项的处理方案（例如：为 `spikes/stage-overlay/.../VRM1_Constraint_Twist_Sample.vrm` 补登清单或移除重复 10.78 MB 副本；为 `codex-cli 0.162.1` 两处 `generated/` 目录补登来源与 Apache-2.0 许可声明；在 P7 发布门控中跟踪 `SenseVoiceSmall` 与 `llama.cpp` CUDA 二进制归档）。
- 后续在 `apps/stage/public/` 补同级 `manifest.json` 登记 `sample.vrm`，并在 `apps/stage/public/motions/manifest.json` 放入真实 `.vrma` 前补全 `source`/`license`/`redistributable` 字段。
- 后续如希望各包元数据同步声明许可证，可另开卡在根 `package.json`、`core/Cargo.toml` 及各 `pyproject.toml` 中补充 `Apache-2.0` 字段，并将根目录 `LICENSE`、`NOTICE` 加入 `.github/workflows/ci.yml` 的纯文档路径规则。
