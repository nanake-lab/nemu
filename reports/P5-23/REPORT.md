# P5-23 规律引擎 G — 报告

## 当前状态

ADR 0019 改造及本地核验完成，等待用户安排独立交叉审查。分支包含最新 `origin/main` 的 `0b5412e`，原分支 V2-51 已改为 V2-61。远端 CI 以 PR #49 对应提交的结果为准；按用户要求转为可审查、写「请审查」后停下，不合并、不取下一张卡。

## 交付物

| 卡片要求 | 当前实现 |
| --- | --- |
| 候选 B 与完整事件次序 | `recurrence.rs`、`replay.rs` 显式保存 V，首次计入时生成 q，A/V/M/U 使用同一 q，P/c 不衰减；普通修订按 ordinal 折叠历史 q。`engine.rs` 先完成旧修订再捕获 cf，随后计入首次分配；容量分摊独立审计 |
| 数值与分侧语义 | `shares.rs` 正负两边分别按 2^-52 最大余数量化；未变侧不重算。零信号 q=1；权重因子及 W×\|S\| 的正数乘积下溢、严格 LP1 精度耗尽及非有限结果显式故障，整个 G 不发布部分结果。M/U 仍独立舍入，U=0、M≠0 合法 |
| 同输入 Rust/Python 对照 | `tests/reference_*` 原样调用参考 legacy/extra、80,000 整步、events/review/sides 的种子、规模与判据；比较全部标量浮点位、计数、完整分配、cf、故障类别及回滚 |
| P5-34 前的字段 | `pending.rs` 只在 law 内承载进程内 q/V；无生产序列化。测试包络用于 G 重放，不是后继 IDL 编码验收 |
| 清理与差异登记 | 旧候选 ADR 撤下，原研究保留并标注被取代；ADR 0018 保留。原 V2-51 改为 V2-61，V2-49 已由 ADR 0019 / V2-55 取代；V2-38–V2-47 白话清单由 Claude 整理 |

`change_rate.rs` 将仅依赖证据计数的 `flip-frequency-v1` 独立成带版本的部件，在本组之前提供 h∈[0,1)，与同一规律资源绑定。它仍是符号翻转频率估计；更好的检测写法由 P5-33/P5-30 另行比较和发布新规律版本。

## 验收

证据目录：`verification/adr0019-20261023/`；本轮工具链 Rust 1.99.0、Python 3.12.10，target、缓存与临时目录均在 D 盘。

| 命令或检查 | 当前结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过 | `fmt-final.*` |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通过 | `clippy-01.*` |
| 两个 Python 对照适配器的 Ruff / `mypy --strict` | 通过 | `ruff-final.*`、`mypy-final.*` |
| `cargo test --workspace --locked -- --nocapture --test-threads=1` | 149 个单元测试、2 个文档测试通过，退出 0；其中 core-kernel 83 项 | `workspace-01.*` |
| 原 legacy + extra 同输入标量/份额 | 2,539,894 个标量状态、261,581 组份额、7 个显式故障；零位差异；80,000 整步零违例、零故障 | `workspace-01.stdout.log` 的 REFERENCE 与 Same-input parity |
| events/review/sides 同输入事件 | 13,339 个不同成功事件、5 个原子故障；R、分配与 cf 零位差异 | 同日志 EVENT_REFERENCE 与 Full event parity |
| AC2-01 独立命令 | passed，退出 0；π 由 0.8857822109669752 升至 0.8858475837751462 | `AC2-01/adr0019-20261024/result.json`、`ac2-01-final.*` |
| 原 LAW-6 | failed，退出 1；原判据 3,000 序列、3,064 个状态违例 | `LAW-6/adr0019-20261024/result.json`、`law-6-final.*` |

原 LAW-6 的失败由 ADR 0019 / V2-60 接受为保证变化，不将诊断测试通过写成 LAW-6 通过。其余原 LAW 判据保留；LAW-7 的闭式差异计数为 488，与当前参考一致。

workspace stderr 中的两条编译诊断属于 core-store 标明 `compile fail ... ok` 的类型能力文档测试，不是构建失败。

首轮 `kernel-01` 为 81 通过、2 失败：旧组修订适配器未按 Python 规范化负零，以及晚阶段故障夹具实际先触发 LP1 精度故障。当前修正保留全部位比较，并分别通过 LP1 精度耗尽和乘积下溢的整步回滚检查。原失败日志、诊断日志、早期编译失败与旧研究证据全部保留；源哈希见 `validated-source-sha256.json`，结果与输入流摘要见 `summary-final.json`。最终检查后核对源码哈希未变。

## 规格对照与证据边界

见 [P5-23 规格对照](../../docs/traceability/P5-23.md) 的 ADR 0019 追加节。I03/I10/I18/I24 由纯函数、受限输入及显式状态约束；I23 由存下的分配重放与合成 H 前缀重做检查；I25 按每边精确守恒检查；I29 仅用合成夹具和临时存储。

事件对照提供预建倾向和 W/S 计划，复用真实 G 的计划、修订、首次插入与发布流程；候选、别名、a(e) 到 S、关系和容量由生产 G 回归覆盖。对照结果不证明完整运行时、真实有效 H 的来源更正、暂停补算或后继 IDL 的生产持久化。原 Python 参考文件、IDL、黄金向量及冻结规格均未改动。

## 新增依赖

无运行时依赖；law 内 Rust 文件均不超过 400 行。

## 待用户确认与下一步

- [ ] 独立交叉审查；V2-38–V2-47 白话清单由 Claude 整理后供用户一次确认，本卡不代做。
- [x] ADR 0019 / V2-60 的保证变化已按用户决定执行；原 LAW-6 继续报告失败。

P5-34 合并前使用本卡获准的临时结构；合并后替换生成类型并重验。在原 PR #49 完成审查交接后停下。
