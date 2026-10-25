# P0-10 载体推理引擎实测 — 报告

## 当前状态

**部分完成：前置盘点与静态注入位置核对完成，所有 GPU/运行时验收未实测。** 已从 `origin/main` `d66241672be5d3932c2d62daef5ec3754a6dc852` 建立 `task/P0-10-carrier-engine`，草稿 [PR #87](https://github.com/nanake-lab/nemu/pull/87) 认领。等用户决定 [BLOCKED.md](BLOCKED.md) 的模型/依赖准备方式，之后再安排 GPU 安静窗口；保持草稿，不请求最终审查、不转为可审查、不合并、不取下一卡。

完整阅读 AGENTS、ARCHITECTURE、CONTRACTS、Echo Core README 以及本卡列出的 P0-09、ADR 0012、P0-07、P0-08 和 v2.0 §9.1/9.5。代码范围仅 `spikes/carrier-engine/`；本目录按用户要求与 AGENTS §6 交报告、证据。未修改任务卡、生产配置、IDL、worker 或用户/系统设置。

## 交付物

- [MEASUREMENT-PLAN.md](../../spikes/carrier-engine/MEASUREMENT-PLAN.md)：同修订量化、优先注入实验、原/紧凑 IRL、显存与窗口计划。
- [preflight.ps1](../../spikes/carrier-engine/preflight.ps1)：只读收集 runner、预期路径、输入 SHA-256；不加载引擎或模型、不查询 GPU。
- [inventory.json](preparation/inventory-01/inventory.json)：2026-10-25 13:07 UTC（Perth 21:07）原始盘点；200 条 IRL（中/英/日 67/67/66）、输入绑定、D 盘剩余字节与 runner 状态。
- [upstream/](preparation/upstream/) 和 [来源清单](preparation/upstream-manifest.json)：固定上游源码快照与 MIT 许可，只用于静态核对；没有安装或运行该引擎。`llama.h`、`llama-adapter.cpp` 超过 400 行，因保留未改写的上游证据而保持原样，不属于新增实现。

## 注入相关结果（最重要）

| 问题 | 当前能证明什么 | 运行时结论 |
| --- | --- | --- |
| 位置差异 | P0-07 的 [injection.py](../../workers/model-adapter/probe/injection.py) 在 attention 残差之后、FFN 的 `post_attention_layernorm` 之前加向量；固定上游 `qwen3.cpp:132–151` 先完成 FFN 和第二次残差，再 `build_cvec`。两个源码位置不同 | 同向量在两位置造成的激活/logits/token 差异**未实测**，不能称等价 |
| token 掩码 | 旧探针只改完整 prefill 最后位置与逐个 decode；上游 `llama-adapter.cpp` 的 `apply_to` 是张量加法，没有旧探针的最后位置选择代码 | 实际 prefill/decode 覆盖、位置差与掩码差的配对**未实测** |
| 零调制一致 | ADR 0012 §2.1 要求同权重、量化、内核配置，与未安装 hook/向量的路径比较 | 未安装/全零/撤除向量三臂逐 token/logits **未实测**；旧 Transformers 的零差不能代替 llama.cpp 证据 |
| 每请求更新向量 | `llama.h:735–747` 声明 `llama_set_adapter_cvec(ctx, data, len, n_embd, il_start, il_end)`；源码有写入向量和清除分支 | 同一模型/上下文连续请求的更新、graph 复用、KV 清理、A→B→0→A 对新上下文的对照均**未实测**。不能仅据 API 宣称可行 |
| 在新位置重提取 | v2.0 §9.5 要求重提取全部方向并按 §9.6 验证；计划单列新位置对比提取 | 提取与三语通路/能力无损均**未实测**；没有迁移许可或方向可沿用结论 |

上游核对固定为 [`1a0b56cec73dac938e0602c974b21dc8e2862830`](https://github.com/ggml-org/llama.cpp/tree/1a0b56cec73dac938e0602c974b21dc8e2862830)：[Qwen3 图](https://github.com/ggml-org/llama.cpp/blob/1a0b56cec73dac938e0602c974b21dc8e2862830/src/models/qwen3.cpp)、[control vector 实现](https://github.com/ggml-org/llama.cpp/blob/1a0b56cec73dac938e0602c974b21dc8e2862830/src/llama-adapter.cpp)、[C API](https://github.com/ggml-org/llama.cpp/blob/1a0b56cec73dac938e0602c974b21dc8e2862830/include/llama.h)。只是源码核对，未编译，也未选作实际测量/生产版本。

## 与已有基线的对比

以下旧值来自仓库已提交报告，均不是本轮重测，也不是不同引擎的配对因果结论。没有为缺失的 llama.cpp 数字补估算。

| 指标 | Transformers 既有基线 | llama.cpp Q4_K_M | llama.cpp Q8_0 |
| --- | --- | --- | --- |
| P0-08 配对 50 条，无约束 prefill wall P50/P90 | 394.45 / 450.39 ms | 未实测 | 未实测 |
| 同组首 token P50/P90 | 396.67 / 452.50 ms | 未实测 | 未实测 |
| 同组 decode 速率 P50，无/有约束 | 21.18 / 13.55 token/s | 未实测 | 未实测 |
| 同组完成 P50，无/有约束 | 9905.26 / 15389.07 ms | 未实测 | 未实测 |
| 原本地 200 条 schema / 来源检查 | 192/200 / 169/200 | 未实测 | 未实测 |
| 紧凑 schema 输出 token/完成时间/展开后合格率 | 没有本卡对应基线 | 未实测 | 未实测 |
| 载体单独显存 | 没有本卡对应单载体基线 | 未实测 | 未实测 |
| GPT-SoVITS V5 + 渲染 + 载体显存 | 未实测 | 未实测 | 未实测 |

基线来源：[P0-08 LOCAL-TIMING](../P0-08/review-r1/LOCAL-TIMING.md)、[复审报告](../P0-08/REVIEW-R1-REPORT.md)。原 200 条与 50 条诊断总体分别列出；旧有 8 条日文解析失败、49/50 的配对组 schema 结果仍保留，未以新证据覆盖。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pwsh -NoProfile -File spikes/carrier-engine/preflight.ps1 -OutputDirectory reports/P0-10/preparation/inventory-01` | 退出 0，仅盘点完成。Listener PID 43336 存在；PATH 与预期路径无引擎/量化文件 | inventory.json |
| 上游源码读取与 SHA-256 | 固定提交 4 文件已保存；不是运行时验收 | upstream-manifest.json |
| 准备材料机械复核 | diff 检查退出 0；PowerShell 语法错误 0；两个 JSON 可解析；上游哈希 4/4、输入绑定 16/16 一致；11 条改动路径均在本卡代码/报告目录 | 角色返回记录与上述原始 JSON |
| 本卡速度、两量化、200 原 schema、紧凑 schema | **未实测**，模型/引擎前置未具备 | BLOCKED.md |
| 注入位置配对、零调制、每请求更新 | **未实测** | BLOCKED.md |
| 载体/GPT-SoVITS V5/渲染显存 | **未实测** | BLOCKED.md |
| 全语言工作区检查、CI | 本卡尚无业务实现，未运行整库静态/测试；不声称 CI 全绿或任务完成 | 本报告状态 |

限定只读盘点未发现 GPT-SoVITS V5 部署，故下一次按用户指示先只测载体。ADR 0027 的约 3.5 GB 是第三方评测转述，未实测，不与载体数字相加替代同驻数据。旧 Qwen3-TTS 缓存不能代替本次 TTS 对象。

准备时 Context7 返回月度额度耗尽；改读官方源码。GitHub 匿名 release API 返回 rate limit；没有反复重试、安装替代工具、获取凭据或把空响应当作已固定 release。源码 HEAD 通过 `git ls-remote` 固定。两项工具读取限制不代表引擎或模型性能失败。

机械盘点使用 `routine_reporter` 角色名，配置要求 Luna/max，子代理只读返回素材，主代理核对并生成原始预检。没有委派判断或重负载；本地 TOML 自动加载能力未单独验证。

## 规格对照

| 条款 | 本卡安排 | 当前证据 |
| --- | --- | --- |
| P0-10 步骤 1–4 | 量化/注入/IRL/显存计划 | 只有预检，所有测量未实测 |
| v2.0 §9.1、§9.5–9.6；I17 | 方向绑定当前载体配置；换位置/引擎不能沿用旧通路批准 | 静态位置核对，不是迁移通过 |
| ADR 0012 §2.1 / §2.2 | 同配置零调制；专用/共享各峰值相加 | 新引擎未实测 |
| J05 / m 变化后的缓存失效 | 同上下文更新与 fresh 对照，显式 KV 清理 | 未实测 |
| AGENTS §9 | 先报告内容与时间、用户停 runner、入口复查 | Listener 在线，未启动 GPU |

## 新增依赖

无已安装的运行时依赖。拟用于 spike 的 llama.cpp 为 MIT（仅保存源码快照）；固定 Qwen 模型为 Apache-2.0，尚未下载。转换/构建工具的精确版本与许可证待用户决定准备方式后固定；不在此引入生产依赖或另选模型。

## 待用户确认

- [ ] （阻塞）按 BLOCKED.md 选模型/引擎准备方式。
- [ ] （阻塞）准备完成后安排 15–20 分钟最小 GPU 实验窗口，并由用户停掉 runner；当前没有确认。

## 后续建议（不在本任务范围内）

- 本卡结束时只给 P0-09 引擎/注入/IRL 结论草稿，由用户另行决定 ADR；当前证据不足以给出这些选型结论。
- GPT-SoVITS V5 的部署/适配由声音工作处理；本卡不恢复暂停的 P4-02。
