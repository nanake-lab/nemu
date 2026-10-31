风险档位：重档；修改 core-kernel 的成长故障处理，按 AGENTS.md §6 保留完整验证和失败证据。

# P5-26 重算、来源更正、暂停与复核 — 报告

## 当前状态

**R1 已修复，待复审；整卡仍为部分交付，PR 保持草稿。** 已确认的计算故障持续保留完整旧状态，普通事件、恢复成长、复核或版本切换不能隐式解除；必须提供绑定当前故障的显式 Repair，成功从 H 补算后才能继续成长。完整独立第二实现、真实 H 投影与 H/L 同事务接线、完整 C 规范导出、正式探测校准及 V2→V3 迁移/回退/SYS17 尚未完成，见 [BLOCKED.md](BLOCKED.md)。旧规律仅作条件性验证，不能用于生产。本次未新增规律、身份设定或默认参数。需要用户做什么：安排复审当前修复；无新增决定项。

分支 `task/P5-26-recompute`；[PR #121](https://github.com/nanake-lab/nemu/pull/121)。本次 fetch 后以 `db8802f317b5b4dd5fe57699ef724fbac0d940f1` 为 main 验证基线 B，用 merge 提交 `624eb309c303360651542dc1ecd42088a8c4dd96` 同步。已读最新 AGENTS §5/§6、任务卡与相关 ADR/规格变化；原基线到 B 只涉及规则与文档，不含 Rust、锁文件或工具链变化。#117 合并提交 `2adc03bac3aa1053c70b6f442793af030e813de3` 仍在 B 中。

本次仅修改卡目录、`reports/P5-26/` 和一个请求文件。按用户限定，未追加卡外 `lib.rs` 登记；因此工作区原有构建不编译本模块。目录内独立 harness 直接编译同一源码及 P5-24 evidence，链接同树实际 G/C/J；工作区检查另外执行，不互相冒充覆盖。

## 交付物

- [`recompute/`](../../core/crates/core-kernel/src/recompute/)：`Kernel` 显式提供规律、genesis、有效历史投影和事件/输入/状态逐位比较；`start/step/replay` 管理普通增量、从头折叠、来源/解释/规律版本、手动暂停与补算。
- `engine.rs`：不可变单提交扩展、目标/已应用版本区分、候选整体拒绝、按历史核验结果重放、逐报警复核。输出暂存差异与双接收方通知意图；不写存储或发通知。
- `inspection.rs`：完整状态比较与缺失/失败/超时分流；八个中性探测，按倾向身份比较 m 系数及身体读出，严格大于阈值报警。固定幅度/阈值须外部显式传入并绑定规律。
- `tests/legacy.rs`：仅测试构建存在的旧规律适配，实际调用 evidence→G→C；覆盖全部 R/V3 组字段与全部 C 字段。R/组用现有规范字节比较；C 用派生全字段 `PartialEq` 加 `Debug` 区分有符号零，沿用 P5-25 测试方法。**Debug 不是生产序列化/规范导出格式。**
- [`P5-26-integration.md`](../../tasks/requests/P5-26-integration.md)：正式 H、来源/解释记录、C 完整状态接口、H/L/通知合同、迁移和模块/验收登记的卡外请求。完整第二实现及迁移的任务责任仍保留在 P5-26。

## 新增依赖

不增加运行时依赖。临时 harness 使用仓库已有 `serde_json = 1.0.154`（MIT OR Apache-2.0）作为测试依赖；复用工作区 `libm = 0.2.16` 的实际 C，不另选数值库。

## R1 修复与回归

对应 [R1 审查评论](https://github.com/nanake-lab/nemu/pull/121#issuecomment-6514135179)。`engine.rs`/`types.rs` 用一个已有 `CheckRecord` 的可选值保存未解决故障，与待重算和手动暂停分开。普通提交继续保留 H，审计保留故障，此时不执行 G/C。Repair 引用当前报警 ID、谱系、H 前缀及规律/解释/来源绑定，使用当前目标资源；手动暂停期间拒绝 Repair，Resume 仅解除手动暂停。修复补算再次失败会保留新的故障，旧引用不能解除它。条件就近注明 V2-97(3)、V2-101、AC2-18，不新增阈值或框架。

[`tests/fault_retention.rs`](../../core/crates/core-kernel/src/recompute/tests/fault_retention.rs) 的四项回归分别覆盖不一致/计算故障后追加普通事件与暂停/恢复。**修复前四项全失败（退出 101）**：增量和重放都把第 4 条经历的状态错误推进到 6；修复后直接对照完整最后正确状态，均保持在 4。回归提交 `2aedc70970006ca46bd8826c3ac07b1f697d4223` 先于修复提交 `bcd035d91af73b6c7dd9ce5bb195bff5c082b042`。

另三项测试覆盖错绑定、手动暂停、成功修复后补算及后续成长、新检查/两种复核结论/新版本不能代替修复，以及修复再次失败与过期引用。`tests/boundaries.rs` 还验证内核仍有真实计算故障时普通事件不会重新调用它；`tests/properties.rs` 将未解决故障纳入 I23 全控制记录比较；`tests/controls.rs` 更新显式修复输入。`verify.ps1` 仅增加可选 TestFilter，最终全量测试不使用过滤器。README 说明机制边界，正式修复记录的认证与 H/L 接线追加到请求文件。

## 验收与证据

检查串行执行。每次命令、工具链、基线、开始/结束 HEAD、源码 SHA-256、stdout/stderr 和锁文件保留于 [`verification/`](verification/)。入口为 `pwsh -NoProfile -File core/crates/core-kernel/src/recompute/verify.ps1 -Mode <模式> -RunId <目录名>`；所有数据为合成数据或临时存储。最终源码和原始日志另存新 ZIP，旧归档不改写。

| 命令或检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 原问题回归 Test + `-TestFilter recompute::tests::fault_retention::r1_` | 预期失败：0 通过、4 失败，退出 101 | `verification/r1-20261101-red/` |
| 同四项回归，修复后 | 4/4 通过 | `verification/r1-20261101-green/` |
| R1 全部七项回归/边界 | 7/7 通过 | `verification/r1-20261101-boundaries/` |
| 同树 harness `cargo test --offline` | **68/68 通过**；40 项上游 evidence、28 项本卡，无忽略 | `verification/r1-20261101-test-final/` |
| 同树 harness Clippy `--all-targets -- -D warnings` | 通过，退出码 0 | `verification/r1-20261101-clippy/` |
| 模块 `rustfmt --edition 2024 --check` | 通过 | `verification/r1-20261101-format/` |
| `cargo test --workspace --locked --offline -- --nocapture --test-threads=1` | **通过，退出码 0** | `verification/r1-20261101-workspace-test/` |
| 工作区 Clippy `--workspace --all-targets -- -D warnings` | 通过，退出码 0 | `verification/r1-20261101-workspace-clippy/` |
| 工作区 `cargo fmt --all -- --check` | 通过 | `verification/r1-20261101-workspace-format/` |
| P5-23 Python 标量/份额对照 | 2,539,894 状态、261,581 份额向量、7 种显式故障，零位差异 | 同工作区 stdout 的 `Same-input parity` |
| P5-23 Python 事件对照 | 13,339 个事件、5 个原子故障，R/分配/反事实零差异 | 同工作区 stdout 的 `Full event parity` |
| 完整第二套有效 H→(R,θ) 实现 | **未完成**；已有 Python 对照仅覆盖子集 | `BLOCKED.md` |
| 真实迁移/失败回退/SYS17、H/L 事务、真实通知、生产 F/J 探测 | **not_run**，接口/目录边界尚未闭合 | 请求文件 |

P5-23 Python 入口保持原样：`law::tests::reference_parity::adr0019_python_legacy_and_extra_same_input_all_scalar_bits`、`law::tests::reference_events::adr0019_python_events_review_and_side_tapes_match_g_phase_machine`。这些测试仍以原 Python 参考实现生成同输入对照，不把重跑同一 Rust G/C 算作独立实现。

工作区原有 `theta_1_original_500_streams_zero_violations` 按 V2-93 保持忽略（旧 Python `**2` 舍入诊断，原失败证据在 P5-25）；本卡未增加 ignore、改变阈值或删除断言。stderr 的 `E0308` 等是现有谱系隔离 `compile_fail` 文档测试的预期输出，对应 doctest 均通过，不能误读成工作区编译失败。

原始失败保留：`test-01` 的夹具编译失败（不存在的 `Limits::default()`）、中间 54/59 项与旧最终 61 项结果、原 `source-final.zip` 和 `raw-verification.zip` 均不改写。R1 的原始失败永久保留，不用绿色结果覆盖。

## 规格对照与验收边界

| 规格/卡条款 | 实现与验证 | 状态/限制 |
| --- | --- | --- |
| v2 §2.2、§5.4；AC2-08、I23 | `step` 在投影前缀逐位一致时续算；`replay` 在每个有效历史前缀从 genesis 折叠。固定种子 1–16、每条 24 前缀；包括暂停/恢复、复核、来源更正、新规律、不可用/不一致/故障记录；比较 R、全部组、完整 θ 和控制记录（含未解决故障） | 384 次条件性比较；共享控制逻辑，R1 另以最后正确状态断言；真实 H/存储入口未验收 |
| v2 §2.5；AC2-18 | 暂停不调用 G/C，继续保留 H；暂停期仅推进目标版本；无故障时恢复全 H 补算，有故障时 Resume 仅解除手动暂停，暂停期间拒绝 Repair | 机制测试通过；`GROWTH_PAUSED` 正式合同及 H/L 事务待接线 |
| v2 §2.2、§5.4；AC2-19 | 更正选择新的有效来源投影，从 genesis 重建 q/V 与 C；移除事件等于从未出现。账号合并从四组重算为一组，不改旧原始事件 | 条件性验证；真实更正认证/原文保全/说话撤回语义由 H 投影层提供 |
| 新规律版本、重新解释；AC2-20 核心 | 显式版本事件强制全算；故障未修复时仅推进目标版本，绑定 Repair 成功后才应用；两种复核结论均无否决结果入口 | 版本机制通过；新解释实际追加与持久差异记录未接线 |
| v2 §12；V2-97、V2-101；AC2-20/21 | 无既有故障时，一致/缺失/失败/超时均应用并报警；明确不一致或计算故障持续保留整个旧状态，只有绑定修复补算成功才解除；历史重放读取旧结论 | R1 已修复，待复审；**完整独立实现未交付**，不可用分支不是验收替代 |
| AC2-21 探测 | 四通道各正负一次，共八个探测；真实数值 J 输出的系数/身体比较；阈值相等不报警、超过报警，拒绝非有限/重复系数/错序/错资源 | 仅比较器与 J 条件验证；未验收实际 F→J 投影、校准或每次运行时提交触发 |
| AC2-09 | 重算/G/C 输入无 x/m；在真实 G/C 合成历史中改变 g/λ 先验范围，R 与证据组不变、θ 确有变化 | 纯算子边界验证；调用方改变 x/m 的完整生产管线 not_run |
| ADR 0019、V2-96 | 使用真实 G 的有限 binary64、固定顺序与下溢拒绝；中途计算失败不泄漏半个候选，记录故障后才可重放拒绝 | G 的数学/Python 子集对照通过；不新增容错 clamp/epsilon |
| ADR 0021 §11 | 来源更正/新规律从 H 重建 q/V，不从旧 B−c 推导、不补 q=1 | 内存候选可验证；**V2→V3 存储迁移、原子发布及回退/SYS17 未完成** |
| V2-105 | 容量 0 时 C 仍处理经历、不自动暂停；容量提高并显式重提后从全部有效 H 恢复证据 | 合成历史中三次外源事件仍更新 C、重提收回四组；不建隐藏等待队列 |
| I18、V2-87 | 缓存字段私有、可丢弃重建；暂停/核验/复核按显式系统输入重放，不送入 G/C 经历 | 通用机制成立；正式系统记录类型和权威事务未实现 |

`Kernel` 只规定机制需要的纯接口，具体规律可更换状态/估计器表示和公式；漂移分量身份属于所选规律，本层不固定其布局。进程内 DTO 不带 Serialize/WireMessage，没有发明另一套权威存储格式。来源/解释映射均须已接受、完整且版本不可变；缺少资源即明确返回错误，不猜测来源或调用模型补记录。

## ADR 0032 对验收的影响

| 被影响条款 | 本卡处理 |
| --- | --- |
| D12 取代 EV7 直接教导 | 上游 evidence 旧用例和旧适配仅作条件性机制回归；不声称现行教导规则，不接入生产 |
| D20 取代 EV4/I27 撒谎排除 | 不把旧排除行为写入通用重算层；更正来自外部有效 H 投影，旧用例通过不等于现行 I27 验收 |
| D13 取代 EV9 单一可核实权重 | 不在机制中内置 verifiable→权重策略；旧 G/组装数值只验证旧版本的确定性 |
| D07 出生基线随经历消退 | 不设计新公式或新默认值；只验证版本替换后能够从头重算 |
| D09/D10 取消统一回落 | 不实现后继公式，不用旧回落结果作为现行保证 |
| AC2-25、COG-A12 被取代部分 | **不作为本卡验收**；完整工作区中的旧规律历史测试结果也不构成这些条款的现行验收 |

## 基线、证据与剩余工作

最终源码清单与六项本机检查记录逐文件一致（15 个文件，含 13 个 Rust 文件），见 [source-check.json](review-r1-20261101/source-check.json)。原问题复现使用的 engine/types/旧适配与旧 test-04 清单一致，见 [regression-origin.json](review-r1-20261101/regression-origin.json)。原基线到本次 B 的文件变化及 #117 前置条件见 [baseline.json](review-r1-20261101/baseline.json)。收尾 fetch 所见 M 仍等于 B，无新增影响，见 [main-closeout.json](review-r1-20261101/main-closeout.json)。

本轮汇总见 [result.json](review-r1-20261101/result.json)。新 [源码归档](review-r1-20261101/source.zip) 与 [原始日志归档](review-r1-20261101/raw-verification.zip) 共 60 个文件已逐文件核对原始字节，见 [archive-check.json](review-r1-20261101/archive-check.json) 和 [归档哈希](review-r1-20261101/artifacts-sha256.json)。日志 ZIP 保留 Git 换行规范化前的字节；上次审查的日志与归档均未改写。

本次最终源码提交为 `bcd035d91af73b6c7dd9ce5bb195bff5c082b042`，全部最终本机检查在该 HEAD 上开始；源码清单包括 13 个 Rust 文件及 README/验证脚本。工作区检查与本模块 harness 检查各自覆盖自己的范围，不互相冒充。

完整第二实现及必要 C/H 接口、迁移扩围、真实事务/探测接线列于 BLOCKED 和请求文件，未用 Missing 分支替代验收，未把它们默默改派他卡。按无人值守要求选保守边界完成其余工作，不替用户作规律/身份决定，不改设置、不操作 runner。因整卡验收未完成，按 AGENTS.md §2“部分完成时保持草稿”保留 draft，留言“请审查”请求审查当前机制，不宣称已达到转 ready 或合并条件。结束后停止本卡，不接下一张卡。
