# P0-10 载体推理引擎实测 — 报告

## 当前状态

**部分完成：注入配对已实测；用户已批准修正日志参数并只补测两量化速度/IRL。** 前次 Perth 10:00–10:01:43 窗口的成功/失败原记录保留。现已将 server 日志 3 改为 4，原 CUDA/全 offload 判据不变，CPU 检查通过，正在准备新的补测窗口，不重跑注入。[PR #87](https://github.com/nanake-lab/nemu/pull/87) 保持草稿，不合并、不取下一卡；补测尚未实测，见 [BENCHMARK-RERUN.md](BENCHMARK-RERUN.md)。

本分支继续合并了 `origin/main` 的 `716f7f0`，重新读了变动后的 ARCHITECTURE；本卡、AGENTS、CONTRACTS、Echo Core README 未变化。最新 main 的核心/里程碑变动不修改本卡固定输入；解压后的 jobs 与 schemas 与前次逐字节相同，绑定源码只有 http_probe.py 改变。这些数据不能作为持续认知运行验收。

## 最重要的注入结论

| 问题 | 实测证据及边界 |
| --- | --- |
| 零调制 | 本次中/英/日分别 24/11/16 个输出位置（含末个 prefill 与随后 decode），旧位置/整层回调全零、原生全零、B 后清零和撤除与未装回调/向量 plain 的 **51/51 位置完整 logits 字节一致、最大差 0**；四个零/撤除自由生成臂 token 也分别一致。中文达到 24 token 上限、英日遇 EOS；不是任意输入或跨量化一致的证明 |
| 每请求更新 | 同一已加载模型/上下文 A→B→零→A→清除；A 重复、fresh-A、fresh-B 在各自 **51/51 位置字节一致**。A/B 为本次新位置提取方向的 ±1 倍；33 次非空安装（含诊断）均返回 0。合法请求**显式清 KV**；保留旧 KV 的 INVALID 诊断与 fresh-B 不一致，不宣称自动失效或并发可用 |
| 位置差异 | 零起算 layer 17，同单位基向量 e0、同最后位置掩码：attention 残差后与整层残差后的 logits 最大差 zh **0.513860**、en **0.545275**、ja **0.576917**。不能当作等价位置；这些 teacher-forced 位置的 argmax 恰好一致，不代表概念行为一致 |
| token 掩码 | 原生向量实际改变 prefill 全部 **32/24/32 行**；旧掩码只改最后一行。整层 callback-all 与 native-all 在各自 51 位置 logits 字节一致；整层 last 对 all 最大差 zh **0.739853**、en **0.961383**、ja **1.171038**。位置差与掩码差均需处理 |
| 新位置重提取 | 使用 P0-07 原有三语 calm/excited 共 3 对样本，两位置均提取出有限 2560 维方向，离线从激活重算最大差 0；方向余弦 **0.7970414064**。已证明本配置的提取技术可行，不替代 v2.0 §9.5–9.6 的全部方向、三语单调性和能力无损验证，不能批准迁移 |

本次详情见 [INJECTION-REPORT.md](INJECTION-REPORT.md)、[逐位置结果](full-run-01/summary.json)；较早最小实验与两次失败仍见 [SMOKE-REPORT.md](SMOKE-REPORT.md)。不覆盖原证据。本次 3 对提取 + 每语言 24 请求，共 78 次，native 与 sampler 均退出 0，日志确认 CUDA 与 37/37 层 GPU offload。

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

## 与已有基线的对比

旧值来自 [P0-08 LOCAL-TIMING](../P0-08/review-r1/LOCAL-TIMING.md) 和 [复审报告](../P0-08/REVIEW-R1-REPORT.md)，不是本轮重测。权重修订一致不代表量化、内核和 HTTP 观测开销一致，不能作纯引擎因果归因。

| 指标 | 已有 Transformers 基线 | llama.cpp Q4_K_M / Q8_0 |
| --- | --- | --- |
| 50 条无约束 prefill wall P50/P90 | 394.45 / 450.39 ms | 未实测 |
| 同组 TTFT P50/P90 | 396.67 / 452.50 ms | 未实测 |
| 同组 decode P50，无/有约束 | 21.18 / 13.55 token/s | 未实测 |
| 同组完成 P50，无/有约束 | 9905.26 / 15389.07 ms | 未实测 |
| 原本地 200 条 schema / 来源检查 | 192/200 / 169/200 | 未实测 |
| 紧凑 schema 输出 token/时间/展开合格率 | 无对应旧基线 | 未实测 |
| 载体单独显存 | 无对应单载体基线 | Q4 本次回调实验专用最大 3,804,901,376 + 共享最大 125,829,120 = **3,930,730,496 bytes（3.931 GB）**；Q8 未实测 |
| GPT-SoVITS V5 + 渲染 + 载体 | 未实测 | 未实测 |

上述 Q4 按 ADR 0012 两个独立采样峰值相加：67 行、66 行匹配 PID。此配置为 context 512、回调并含瞬时 fresh/reference 上下文，不能与原 context 256 最小实验的 3.297 GB 混为同配置；不是连续峰值、长输入或完整 Nemu 同驻通过。失败 server 的 5 行采样单列在 JSON 中，不作正式 benchmark 显存。GPT-SoVITS V5 未发现部署，按用户指示只测载体；不把第三方约 3.5 GB 与本机数字相加。

## 验收

| 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 原始模型 12 文件、引擎 2 归档重新哈希 | 全部匹配 | preparation/downloads-20261025/ |
| CPU 转换、Q4/Q8 量化、三 GGUF 重哈希 | 全部完成 | conversion-20261026/ |
| 最小 CUDA 探针 | native 与 WDDM sampler 退出 0；37/37 层上 GPU | smoke-gpu-03/ |
| 本次配对注入、零调制、逐请求更新、方向提取 | 78 次执行完成；51 个生成位置配对结果见上表，能力无损/概念行为仍未实测 | full-run-01/、INJECTION-REPORT.md |
| 本次 server/速度/IRL | 健康就绪，但缺少 CUDA/full-offload 日志；入口拒绝后主动结束 server（退出 1），sampler 退出 0；正式样本 0。不是引擎崩溃结论 | full-run-01/失败日志、BLOCKED.md |
| 失败保留 | run-01 因探针 DLL 搜索配置落到 CPU，不作 GPU 结果；run-02 绑定函数到错误 DLL，退出 1。修正探针后 run-03 有 CUDA 证据 | smoke-cpu-fallback-01/、smoke-api-binding-02/、SMOKE-REPORT.md |
| 本卡全部 Python Ruff / strict mypy | 12 文件通过 | full-preparation-20261026/，20261026-04 日志 |
| schema 往返、流式计时/缓存/截断/空响应 | 6 项 CPU unittest 通过 | 同目录 unittest-04.log |
| PowerShell AST / C++ `/W4 /WX` | 3 脚本语法错误 0；最终原生探针编译退出 0 | 同目录 |
| CPU 准备过程失败 | 类型收窄、测试读取未显式 UTF-8、C++ int→float 警告等均修正；原始失败日志保留；不是硬件性能结果 | 同目录历史检查日志 |
| 新离线分析/归档检查 | Ruff 与 strict mypy 见本次检查日志；原文件到 gzip 全量独立核验 6,254/6,254、1,115/1,115 匹配 | full-run-01/analysis-checks/ |
| 两量化速度、原/紧凑 IRL、长输入显存 | **未实测** | BLOCKED.md |
| 整库 TS/Rust/业务测试及当前 head CI | 未运行、不声称通过；本卡改动仅独立 Python/C++ spike 与报告 | PR 仍为草稿 |

## 规格对照

| 条款 | 实现/证据 | 尚欠 |
| --- | --- | --- |
| P0-10 步骤 1–4 | 两量化、配对注入、方向提取和显存 | 完整速度/IRL |
| v2.0 §9.1、§9.5–9.6 | 三语样本两位置提取；无权威存储或配置改动 | 全部方向、单调性与能力验收不在本卡批准范围 |
| ADR 0012 §2.1/§2.2 | 同配置零向量逐位置对照、WDDM 独立峰值 | 长载荷、同驻 |
| 每事件 m 更新与 KV | 显式清 KV + fresh A/B 逐位置配对；保留 KV 另作无效诊断 | 未证明自动失效或生产并发 |
| AGENTS §9 / 用户阻塞即停 | 用户确认、无 runner 后启动；server 日志检查失败后停止并通知恢复 runner | 等用户决定下一步 |

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
- [x] 用户选 1，批准 server 日志 3→4、保留原判据，补测两量化速度/IRL，不重跑已完成注入；补测结果待记录。
- [ ] （后续决策）依据全部实测在 P0-09 决定引擎/注入/IRL；当前没有自行选型或迁移。

## 后续建议（不在本任务范围内）

- GPT-SoVITS V5 部署后另做实际同驻验证；本卡先载体单测符合用户明确降范围指示，不拿估算替代。
- P5-31 仍需当前配置的全部方向、三语行为单调性和能力无损；三组对比样本只回答提取技术可行性。
