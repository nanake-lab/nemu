# P0-10 载体推理引擎实测 — 报告

## 当前状态

**部分完成：下载、CPU 转换和 Q4_K_M 最小 CUDA 实验完成；完整验收未完成。** 完整测量脚本已准备并做 CPU 检查，等待新的约 1–2 小时 GPU 安静窗口。[PR #87](https://github.com/nanake-lab/nemu/pull/87) 保持草稿，不合并、不取下一卡。上次最小窗口已结束并通知用户恢复 runner，不沿用该次确认。

本分支已 merge `origin/main` 的 `9ada24b`，重新读了变动后的 ARCHITECTURE；本卡、AGENTS、CONTRACTS、Echo Core README 未变化。新增 ADR 0026 的持续认知不改变本卡单请求测量的范围；这些数据不能作为持续运行验收。

## 最重要的注入结论

| 问题 | 实测证据及边界 |
| --- | --- |
| 零调制 | 三条中/英/日短输入中，未装向量、全零、撤除三臂的**末个 prefill logits 字节一致，最大差 0**；各生成 3 token（含 EOS）也一致。尚未完成逐 decode 位置的全 logits 对照，不能扩大为任意输入一致 |
| 每请求更新 | 同一已加载模型/上下文连续 A→B→清除→A，设置/清除 API 18 次全部返回 0；A 重复与 fresh-A 的末个 prefill logits 最大差均为 0。每请求**显式清 KV**。A/B 是 ±单位基向量；没有自动 KV 失效、并发或正式方向通路结论 |
| 位置差异 | P0-07 在零起算第 17 层 attention 残差后、FFN norm 前；实际 b11669 的 Qwen3 图在 FFN 和第二次残差后应用 control vector。静态位置不同；同向量/同掩码的实际效果**未实测** |
| token 掩码 | 旧探针仅改最后 prefill 位置与每个 decode。原生向量广播的实际覆盖及与最后位置掩码的配对**未实测** |
| 新位置重提取 | 准备使用 P0-07 原有三语 calm/excited 对比样本在两位置提取；**未实测**。不替代 v2.0 §9.5–9.6 的全部方向、三语单调性和能力无损验证，不能批准迁移 |

完整最小数据与失败见 [SMOKE-REPORT.md](SMOKE-REPORT.md)。A 对 plain / B 对 A 的最大 logits 差分别为 zh 0.666821 / 0.677299、en 0.642494 / 0.532833、ja 0.797075 / 0.669366；这是向量实际影响 logits 的证据，不能据此判断概念行为正确。

## 交付物

- [转换及二次哈希](conversion-20261026/inventory.json)：同一 Qwen 修订 `cdbee75f17c01a7cc42f958dc650907174af0554`，BF16、Q4_K_M、Q8_0 三产物在 D 盘缓存。Q4_K_M 2,497,279,104 bytes；Q8_0 4,280,403,584 bytes；转换四步退出 0。
- [最小 CUDA 原始记录](smoke-gpu-03/)：21 请求、原始 logits、token、激活配置、WDDM、stdout/stderr 和源码快照。
- [完整测量准备](full-preparation-20261026/)：CPU 检查、编译记录、固定输入与依赖元数据；**不是 GPU 测量结果**。
- `spikes/carrier-engine/injection_probe.cpp`、`native_probe.hpp`、`activation_trace.hpp`：同模型两位置/掩码配对、逐位置 logits、零调制自由生成、连续更新/fresh 对照、重提取可行性；尚未运行。回调的 observation/setter 组合为实验方法，不能仅因编译通过就宣称可用。
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
| 载体单独显存 | 无对应单载体基线 | Q4 最小短输入专用最大 3,196,669,952 + 共享最大 100,663,296 = **3,297,333,248 bytes（3.297 GB）**；Q8 未实测 |
| GPT-SoVITS V5 + 渲染 + 载体 | 未实测 | 未实测 |

上述 Q4 显存按 ADR 0012 的两个独立采样峰值相加：38 行、33 行匹配 PID；不是连续峰值或完整 Nemu 同驻通过。限定路径盘点未发现 GPT-SoVITS V5 部署，按用户指示先只测载体，不部署替代声音模型，不把第三方约 3.5 GB 与本机数字相加。

## 验收

| 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 原始模型 12 文件、引擎 2 归档重新哈希 | 全部匹配 | preparation/downloads-20261025/ |
| CPU 转换、Q4/Q8 量化、三 GGUF 重哈希 | 全部完成 | conversion-20261026/ |
| 最小 CUDA 探针 | native 与 WDDM sampler 退出 0；37/37 层上 GPU | smoke-gpu-03/ |
| 失败保留 | run-01 因探针 DLL 搜索配置落到 CPU，不作 GPU 结果；run-02 绑定函数到错误 DLL，退出 1。修正探针后 run-03 有 CUDA 证据 | smoke-cpu-fallback-01/、smoke-api-binding-02/、SMOKE-REPORT.md |
| 本卡全部 Python Ruff / strict mypy | 12 文件通过 | full-preparation-20261026/，20261026-04 日志 |
| schema 往返、流式计时/缓存/截断/空响应 | 6 项 CPU unittest 通过 | 同目录 unittest-04.log |
| PowerShell AST / C++ `/W4 /WX` | 3 脚本语法错误 0；最终原生探针编译退出 0 | 同目录 |
| CPU 准备过程失败 | 类型收窄、测试读取未显式 UTF-8、C++ int→float 警告等均修正；原始失败日志保留；不是硬件性能结果 | 同目录历史检查日志 |
| 完整位置、逐 token、一致更新、重提取 | **未实测** | BLOCKED.md |
| 两量化速度、原/紧凑 IRL、长输入显存 | **未实测** | BLOCKED.md |
| 整库 TS/Rust/业务测试及当前 head CI | 未运行、不声称通过；本卡改动仅独立 Python/C++ spike 与报告 | PR 仍为草稿 |

## 规格对照

| 条款 | 实现/证据 | 尚欠 |
| --- | --- | --- |
| P0-10 步骤 1–4 | 两量化、最小载体运行、完整准备脚本 | 完整速度/IRL/注入测量 |
| v2.0 §9.1、§9.5–9.6 | 配对提取计划；无权威存储或配置改动 | 全部方向、单调性与能力验收不在本卡批准范围 |
| ADR 0012 §2.1/§2.2 | 同配置零向量最小对照、WDDM 独立峰值 | 逐位置/完整载荷、同驻 |
| 每事件 m 更新与 KV | 库实际调用；显式清 KV + fresh-A | 完整逐位置配对；未证明自动失效 |
| AGENTS §9 | 最小窗口用户确认、入口查无 Listener/Worker，已结束；下一窗口另申请 | 新的完整窗口 |

## 新增依赖

| 名称 | 版本 | 许可与用途 |
| --- | --- | --- |
| llama.cpp | b11669 / `620d2225` | 源码 MIT；官方 CUDA 12.4 包还含 LLVM OpenMP / NVIDIA 条款；本地测量，不批准整包生产分发 |
| Qwen3-4B-Instruct-2507 | `cdbee75f` | Apache-2.0；固定载体及量化派生 |
| CPU 转换环境 | torch 2.11.0+cpu、transformers 4.57.6、numpy 2.2.6、sentencepiece 0.2.2、protobuf 4.25.9、gguf 0.19.0 | 完整 28 包许可元数据见 conversion-20261026/environment.json；torch 无 CUDA。certifi/tqdm 含 MPL-2.0，regex 含 CNRI-Python；工具环境，无产品运行时依赖或分发批准 |
| 独立 schema 校验 | jsonschema 4.26.0、attrs 26.1.0、jsonschema-specifications 2025.9.1、referencing 0.37.0、rpds-py 2026.9.1 | 这五包 MIT；完整 33 包元数据见 full-preparation-20261026/environment.json；requirements-eval.lock 固定 wheel 哈希 |
| 下载/检查/编译工具 | 系统 curl、Python 3.12.10、既有 Ruff 0.15.6 / mypy 1.20.2、MSVC 19.44.35229.0 | 仅工具；没有新装下载器、编译器或修改全局配置 |

## 待用户确认

- [x] 已授权分段下载、CPU 转换及完成的最小 GPU 窗口。
- [ ] （阻塞）新的约 1–2 小时完整 GPU 安静窗口；需用户停 runner 并确认，再查 Listener/Worker 均不存在才启动。
- [ ] （后续决策）依据全部实测在 P0-09 决定引擎/注入/IRL；当前没有自行选型或迁移。

## 后续建议（不在本任务范围内）

- GPT-SoVITS V5 部署后另做实际同驻验证；本卡先载体单测符合用户明确降范围指示，不拿估算替代。
- P5-31 仍需当前配置的全部方向、三语行为单调性和能力无损；三组对比样本只回答提取技术可行性。
