# P0-10 载体推理引擎实测 — 报告

## 当前状态

**本卡测量与本地验收完成。** 已实测注入，以及 Q4_K_M/Q8_0 各 474 次 completion（各 4 预热、470 正式）的速度、原/紧凑 IRL 和载体显存。最新 GPU 运行提交 `a1f18b0f953a0291dd3f09701d35662cca436402`，Perth 18:38:24–19:58:29，父进程退出 0；注入未重跑，GPU 已释放并通知恢复 runner。此前漏查参数顺序的失误、两次失败、修正提交及成功补测均保留，见 [BENCHMARK-RERUN.md](BENCHMARK-RERUN.md)。最终提交仅报告/证据，按 AGENTS §9 不触发CI，不把旧提交绿色当本次通过。[PR #87](https://github.com/nanake-lab/nemu/pull/87) 转为可审查，等待用户审查；不合并、不取下一卡。

测前 merge main `b4eb7fd`，测后先 merge `86779b1`，收尾再同步到 `317dde9`（merge `a6e5926`）并重跑 14 个实现 Python 的 Ruff/strict mypy 和 6 项 unittest，全部通过。必读文档和本卡未变；测后合并没有改变测量输入或 spike 源码。full-v5 解压 jobs/schema 与前次逐字节相同，仅日志参数顺序的源码绑定改变。本卡不选择引擎、不批准迁移，也不作为持续认知或完整语音链路验收。

## 最重要的注入结论

| 问题 | 实测证据及边界 |
| --- | --- |
| 零调制 | 本次中/英/日分别 24/11/16 个输出位置（含末个 prefill 与随后 decode），旧位置/整层回调全零、原生全零、B 后清零和撤除与未装回调/向量 plain 的 **51/51 位置完整 logits 字节一致、最大差 0**；四个零/撤除自由生成臂 token 也分别一致。中文达到 24 token 上限、英日遇 EOS；不是任意输入或跨量化一致的证明 |
| 每请求更新 | 同一已加载模型/上下文 A→B→零→A→清除；A 重复、fresh-A、fresh-B 在各自 **51/51 位置字节一致**。A/B 为本次新位置提取方向的 ±1 倍；33 次非空安装（含诊断）均返回 0。合法请求**显式清 KV**；保留旧 KV 的 INVALID 诊断与 fresh-B 不一致，不宣称自动失效或并发可用 |
| 位置差异 | 零起算 layer 17，同单位基向量 e0、同最后位置掩码：attention 残差后与整层残差后的 logits 最大差 zh **0.513860**、en **0.545275**、ja **0.576917**。不能当作等价位置；这些 teacher-forced 位置的 argmax 恰好一致，不代表概念行为一致 |
| token 掩码 | 原生向量实际改变 prefill 全部 **32/24/32 行**；旧掩码只改最后一行。整层 callback-all 与 native-all 在各自 51 位置 logits 字节一致；整层 last 对 all 最大差 zh **0.739853**、en **0.961383**、ja **1.171038**。位置差与掩码差均需处理 |
| 新位置重提取 | 使用 P0-07 原有三语 calm/excited 共 3 对样本，两位置均提取出有限 2560 维方向，离线从激活重算最大差 0；方向余弦 **0.7970414064**。已证明本配置的提取技术可行，不替代 v2.0 §9.5–9.6 的全部方向、三语单调性和能力无损验证，不能批准迁移 |

本次详情见 [INJECTION-REPORT.md](INJECTION-REPORT.md)、[逐位置结果](full-run-01/summary.json)；较早最小实验与两次失败仍见 [SMOKE-REPORT.md](SMOKE-REPORT.md)。不覆盖原证据。本次 3 对提取 + 每语言 24 请求，共 78 次，native 与 sampler 均退出 0，日志确认 CUDA 与 37/37 层 GPU offload。

**P5-13 的实施约束：每次请求更新注入向量前必须显式清理 KV，然后安装本次向量并重新 prefill；不得沿用上一向量生成的 KV。** 已测串行接口更新依赖这一前提；保留旧 KV 的诊断与 fresh-B 不一致，设置向量成功不等于 KV 自动失效。自动失效、保留 KV 的安全复用和生产并发均未实测。本条按用户 2026-10-26 指示写入结论，不在本卡修改 P5-13 的任务文件或生产实现。

## 交付物

- [转换及二次哈希](conversion-20261026/inventory.json)：同一 Qwen 修订 `cdbee75f17c01a7cc42f958dc650907174af0554`，BF16、Q4_K_M、Q8_0 三产物在 D 盘缓存。Q4_K_M 2,497,279,104 bytes；Q8_0 4,280,403,584 bytes；转换四步退出 0。
- [最小 CUDA 原始记录](smoke-gpu-03/)：21 请求、原始 logits、token、激活配置、WDDM、stdout/stderr 和源码快照。
- [完整测量准备](full-preparation-20261026/)：CPU 检查、编译记录、固定输入与依赖元数据；**不是 GPU 测量结果**。
- [本次完整原始证据](full-run-01/raw-filemap.json)：6,254 文件、830,513,016 原始字节，以 SHA-256 去重成 1,115 个 gzip 对象（246,863,053 bytes），所有原文件可逐字节恢复，独立解压校验全匹配。包含成功注入和失败 server；缓存原文件仍保留，没有删改。
- `spikes/carrier-engine/injection_probe.cpp`、`native_probe.hpp`、`activation_trace.hpp`：本次已执行的配对探针。`summarize_injection.py` 离线分析；`archive_evidence.py` 无损归档，只处理已有证据，不加载引擎。
- `benchmark_plan.py`、`benchmark_native.py`、`http_probe.py`、`full_window.py`：每量化 4 条排除预热、20 条固定句、50 条无约束旧诊断、200 条原 wire schema、200 条紧凑 schema。保持原输入、模板和独立完整 schema/来源校验；原始 SSE、计时、token 和失败逐条保留。顺序执行，不并载两量化。
- `compact_codec.py`：只缩短字段、用枚举/来源编号编码，展开后回到原 SemanticCandidate；节点 ID、角色、外部来源和文本不丢失。仅 spike，不修改生产 IDL。
- [模型与引擎登记](../../spikes/carrier-engine/manifest.json)、[分段下载](../../spikes/carrier-engine/DOWNLOADS.md)、[固定方案](../../spikes/carrier-engine/MEASUREMENT-PLAN.md)。

证据内的上游 `llama-adapter.cpp` 超过 400 行，保留固定提交原文与 MIT 许可便于核对；锁文件是生成的数据。新增实现均未超过 400 行。

新增 [速度/IRL 报告](BENCHMARK-REPORT.md)、[给 P0-09 的选项草稿](P0-09-OPTIONS.md) 和 [本次原始证据](benchmark-run-03/README.md)：3775 文件共 138,579,801 bytes 无损归档为 3152 个 gzip 对象、14,077,348 bytes；另用 .NET 独立解压核验全部对象/映射匹配。独立审计重新校验 948 请求及每量化 450 条正式 IRL。证据中的 reporter 审计脚本超过400行，用于保留完整机械核验方法，不进入应用。

## 与已有基线的对比

旧值来自 [P0-08 LOCAL-TIMING](../P0-08/review-r1/LOCAL-TIMING.md) 和 [复审报告](../P0-08/REVIEW-R1-REPORT.md)，不是本轮重测。权重修订一致不代表量化、内核和 HTTP 观测开销一致，不能作纯引擎因果归因。

| 指标（P50，除另注明） | 已有 Transformers | Q4_K_M | Q8_0 |
| --- | ---: | ---: | ---: |
| 同 50 条无约束 prefill，旧 wall / 新原生，ms | 394.45 | 432.02 | 432.55 |
| 同组客户端 TTFT，ms | 396.67 | **3737.55** | **3672.88** |
| 同组 decode，无/有约束，token/s | 21.18 / 13.55 | 127.99 / 128.37 | 99.13 / 98.21 |
| 同组整次完成，无/有约束，ms | 9905.26 / 15389.07 | 5635.36 / 2571.07 | 5994.16 / 2914.63 |
| 200 条原 Schema / 来源合格 | 192 / 169 | 200 / 192 | 200 / 191 |
| 200 条原/紧凑输出 token | 无对应紧凑基线 | 237 / 97 | 228 / 97 |
| 同组原/紧凑整次完成，ms | 无对应紧凑基线 | 3315.21 / 4564.45 | 4309.32 / 4807.78 |
| 紧凑展开完整 Schema / 来源合格（200 条） | 无对应基线 | 200 / 133 | 200 / 147 |
| 载体单独显存，独立峰值相加，bytes | 无同配置单载体基线 | 4,723,433,472 | 6,506,024,960 |
| GPT-SoVITS V5 + 渲染 + 载体 | 未实测 | 未实测 | 未实测 |

完整 P50/P90、语言计数和方法见 [BENCHMARK-REPORT.md](BENCHMARK-REPORT.md)。decode 加快不等于首 token 全面改善：无约束客户端 TTFT 约 3.7 秒，有约束 P90 也超过 4 秒，未对额外等待作归因。紧凑版每对 token 都减少，但更快仅 Q4 92/200、Q8 95/200，来源合格率下降，不能只按 token 选用。

固定句输入 IDs 与 P0-07 相同，但两量化输出都额外加入顿号，精确匹配各 **0/20**；保留慢样本，不宣称通过旧固定句或 600 ms 选型验收。32 条响应因 UTF-8 分片少返回 SSE token ID，原生计数保留，不补造逐 token 时间；输出 token/TPS 使用原生字段。

显存按 ADR 0012 两独立峰值相加，context 8192，Q4 3730/3729、Q8 3964/3963 行采样/匹配；非连续峰值。注入 context 512 的 3.931 GB 和最小 context 256 的 3.297 GB 单列，不混为同配置。限定部署盘点未发现 GPT-SoVITS V5，按用户授权只测载体，实际 TTS/渲染同驻未实测，不把约 3.5 GB 估算相加。

## 验收

| 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 原始模型 12 文件、引擎 2 归档重新哈希 | 全部匹配 | preparation/downloads-20261025/ |
| CPU 转换、Q4/Q8 量化、三 GGUF 重哈希 | 全部完成 | conversion-20261026/ |
| 最小 CUDA 探针 | native 与 WDDM sampler 退出 0；37/37 层上 GPU | smoke-gpu-03/ |
| 本次配对注入、零调制、逐请求更新、方向提取 | 78 次执行完成；51 个生成位置配对结果见上表，能力无损/概念行为仍未实测 | full-run-01/、INJECTION-REPORT.md |
| 两量化速度、原/紧凑 IRL、载体显存 | 948 次完成，父进程 0；两 server 均 37/37 offload，正常主动结束；两个 sampler 均 0。原始失败输出保留 | benchmark-run-03/、BENCHMARK-REPORT.md |
| 参数失误与前两次失败 | 072141f 日志3；7ab7420仅改4仍漏早期消息，各正式样本0；a1f18b0只提前参数，原判据通过。未删失败、不重跑注入 | BENCHMARK-RERUN.md、benchmark-run-02/ |
| 失败保留 | run-01 因探针 DLL 搜索配置落到 CPU，不作 GPU 结果；run-02 绑定函数到错误 DLL，退出 1。修正探针后 run-03 有 CUDA 证据 | smoke-cpu-fallback-01/、smoke-api-binding-02/、SMOKE-REPORT.md |
| 14 个实现 Python Ruff / strict mypy | 最终 main 合并后 a6e5926 全部通过 | benchmark-run-03/audit/final-main-checks/ |
| schema 往返、流式计时/缓存/截断/空响应 | 最终合并后 6 项 CPU unittest 通过 | 同目录 unittest.log |
| PowerShell AST / C++ `/W4 /WX` | 3 脚本语法错误 0；最终原生探针编译退出 0，后续源码未变 | full-preparation-20261026/build.json、full-suite-psparse-20261026-04.log |
| CPU 准备过程失败 | 类型收窄、测试UTF-8、C++ int→float警告等已修，原始失败日志保留；不是硬件性能结果 | full-preparation-20261026/历史检查日志 |
| 新离线分析/归档检查 | Ruff 与 strict mypy 见本次检查日志；原文件到 gzip 全量独立核验 6,254/6,254、1,115/1,115 匹配 | full-run-01/analysis-checks/ |
| 本次独立数据复核 | 948 请求检查无差异；每量化450正式IRL的Schema/来源布尔无差异；统计/显存/输出另核验 | benchmark-run-03/audit/ |
| 分析异常与失败保留 | 主代理等计数假设失败及reporter首版计入预热均保留；按实际字段另存正确统计，没有改测量判据 | benchmark-run-03/analysis/attempt-01/、audit/ |
| 最终报告提交 CI | 仅报告/证据，按 AGENTS §9 paths-ignore 未触发，不声明当前head通过；最终本地实现检查已通过，源码未再改 | benchmark-run-03/context/report-only-ci.json、PR #87 checks |

## 规格对照

| 条款 | 实现/证据 | 尚欠 |
| --- | --- | --- |
| P0-10 步骤 1–4 | 两量化、配对注入、提取、速度、200条原/紧凑IRL和载体显存全部有记录 | 用户允许先载体单测；同驻仍未实测 |
| v2.0 §9.1、§9.5–9.6 | 三语样本两位置提取；无权威存储或配置改动 | 全部方向、单调性与能力验收不在本卡批准范围 |
| ADR 0012 §2.1/§2.2 | 同配置零向量逐位置对照、context8192的WDDM独立峰值 | 非连续峰值；同驻/三次选型复现未实测 |
| 每事件 m 更新与 KV | 显式清 KV + fresh A/B 逐位置配对；保留 KV 另作无效诊断 | 未证明自动失效或生产并发 |
| AGENTS §9 / 用户阻塞即停 | 前两次失败即停；分别获批后修正，本次无runner观测、完成后通知恢复 | 无未解除的执行阻塞 |

## 新增依赖

| 名称 | 版本 | 许可与用途 |
| --- | --- | --- |
| llama.cpp | b11669 / `620d2225` | 源码 MIT；官方 CUDA 12.4 包还含 LLVM OpenMP / NVIDIA 条款；本地测量，不批准整包生产分发 |
| Qwen3-4B-Instruct-2507 | `cdbee75f` | Apache-2.0；固定载体及量化派生 |
| CPU 转换环境 | torch 2.11.0+cpu、transformers 4.57.6、numpy 2.2.6、sentencepiece 0.2.2、protobuf 4.25.9、gguf 0.19.0 | 完整 28 包许可元数据见 conversion-20261026/environment.json；torch 无 CUDA。certifi/tqdm 含 MPL-2.0，regex 含 CNRI-Python；工具环境，无产品运行时依赖或分发批准 |
| 独立 schema 校验 | jsonschema 4.26.0、attrs 26.1.0、jsonschema-specifications 2025.9.1、referencing 0.37.0、rpds-py 2026.9.1 | 这五包 MIT；完整 33 包元数据见 full-preparation-20261026/environment.json；requirements-eval.lock 固定 wheel 哈希 |
| 下载/检查/编译工具 | 系统 curl、Python 3.12.10、既有 Ruff 0.15.6 / mypy 1.20.2、MSVC 19.44.35229.0 | 仅工具；没有新装下载器、编译器或修改全局配置 |

## 待用户确认

- [x] 已授权分段下载、CPU 转换、最小与本次完整 GPU 窗口；本次已结束。
- [x] 用户选 1 批准的 server 日志 3→4 已应用，原判据保留；补测启动后再次停止，失败原记录已保存，注入未重跑。
- [x] 用户批准的仅提前参数补丁已应用，值/判据不变；无runner后补测完成，前后提交与失败全部保留。
- [x] 两量化速度、200条原/紧凑IRL与载体显存完成；注入未重跑，P5-13显式清KV约束已写入结论。
- [ ] （不阻塞本测量卡，后续ADR）用户据 [P0-09选项草稿](P0-09-OPTIONS.md) 决定引擎/注入/IRL；本卡没有选型、迁移或生产接入批准。

## 后续建议（不在本任务范围内）

- GPT-SoVITS V5 部署后另做实际同驻验证；本卡先载体单测符合用户明确降范围指示，不拿估算替代。
- P5-31 仍需当前配置的全部方向、三语行为单调性和能力无损；三组对比样本只回答提取技术可行性。
