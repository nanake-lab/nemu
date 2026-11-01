风险档位：重档；PR 涉及 core-kernel，本轮合入完整 θ 存储及契约变更，按 AGENTS.md §6 验证新组合；本轮自有改动仅为交付说明和证据。

# P5-26 重算、来源更正、暂停与复核 — 报告

## 当前状态

**本次是显式输入上的重算机制的条件性交付；P5-26 整卡未完成，禁止据此接入生产。** 范围限于显式提供的不可变 H 投影、规律/解释/来源资源下的增量与整段重算、来源更正、手动暂停补算、版本切换及故障保持/修复。旧规律只用于条件性机制验证。

真实 H 接线（含 H/L 同事务、通知及可信来源/时间认证）、V2→V3 迁移与回退、完整独立 H→(R,θ) 第二实现、SYS17、ADR 0032 新规律及其正式验收均为**后继项**；完整 C 接口接入、正式探测校准和模块/验收登记同样未完成。详见 [BLOCKED.md](BLOCKED.md)。这些缺口没有消失或改记为通过，补模块登记或复用旧规律也不能解除生产禁用。

[复审已通过，R1 已关闭](https://github.com/nanake-lab/nemu/pull/121#issuecomment-6558797119)，结论绑定 `7142adc442e3014b84259802e2b5710e61012916` 的部分机制。按用户 2026-11-01 的明确指示，参照 [P5-24 #117 的条件性交付](https://github.com/nanake-lab/nemu/pull/117)，完成本轮组合检查后转为可审查并标记 `needs-user`。这只是本次限定交付的审批状态，不代表整卡验收完成或生产接入许可。需要用户做什么：审查并决定是否接受该条件性交付；本会话不合并。

分支 `task/P5-26-recompute`；[PR #121](https://github.com/nanake-lab/nemu/pull/121)。上次有效验证基线 B=`db8802f317b5b4dd5fe57699ef724fbac0d940f1`，被测源码 `bcd035d91af73b6c7dd9ce5bb195bff5c082b042`。本轮 fetch 后 M=`67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e`，以 `000e3c0990b04016d7ebc6180c10ddfab4232f78` merge，无冲突、无 rebase/强推。已确认 #108 合并提交 `acc71dadc5ba8982392d0aded64b9bf6772b514a` 及 #117 在 M 中。

本轮自有修改限 `reports/P5-26/` 和请求文件，recompute 源码/测试与已复审版本逐文件不变。按用户限定，未追加卡外 `lib.rs` 登记；工作区原有构建不编译本模块。同树 harness 直接编译本模块及 P5-24 evidence、链接实际 G/C/J，工作区检查另外执行。

## 交付物

- [`recompute/`](../../core/crates/core-kernel/src/recompute/)：`Kernel` 显式提供规律、genesis、有效历史投影和事件/输入/状态逐位比较；`start/step/replay` 管理普通增量、从头折叠、来源/解释/规律版本、手动暂停与补算。
- `engine.rs`：不可变单提交扩展、目标/已应用版本区分、候选整体拒绝、按历史核验结果重放、逐报警复核。输出暂存差异与双接收方通知意图；不写存储或发通知。
- `inspection.rs`：完整状态比较与缺失/失败/超时分流；八个中性探测，按倾向身份比较 m 系数及身体读出，严格大于阈值报警。固定幅度/阈值须外部显式传入并绑定规律。
- `tests/legacy.rs`：仅测试构建存在的旧规律适配，实际调用 evidence→G→C；覆盖全部 R/V3 组字段与全部 C 字段。R/组用现有规范字节比较；C 用派生全字段 `PartialEq` 加 `Debug` 区分有符号零，沿用 P5-25 测试方法。**Debug 不是生产序列化/规范导出格式。**
- [`P5-26-integration.md`](../../tasks/requests/P5-26-integration.md)：正式 H、来源/解释记录、完整 C 接口消费、H/L/通知合同、迁移和模块/验收登记的后继接线请求。完整第二实现及迁移仍是整卡未完成项。

## 新增依赖

不增加运行时依赖。临时 harness 使用仓库已有 `serde_json = 1.0.154`（MIT OR Apache-2.0）作为测试依赖；复用工作区 `libm = 0.2.16` 的实际 C，不另选数值库。

## R1 修复与回归

对应 [R1 审查评论](https://github.com/nanake-lab/nemu/pull/121#issuecomment-6514135179)。`engine.rs`/`types.rs` 用一个已有 `CheckRecord` 的可选值保存未解决故障，与待重算和手动暂停分开。普通提交继续保留 H，审计保留故障，此时不执行 G/C。Repair 引用当前报警 ID、谱系、H 前缀及规律/解释/来源绑定，使用当前目标资源；手动暂停期间拒绝 Repair，Resume 仅解除手动暂停。修复补算再次失败会保留新的故障，旧引用不能解除它。条件就近注明 V2-97(3)、V2-101、AC2-18，不新增阈值或框架。

[`tests/fault_retention.rs`](../../core/crates/core-kernel/src/recompute/tests/fault_retention.rs) 的四项回归分别覆盖不一致/计算故障后追加普通事件与暂停/恢复。**修复前四项全失败（退出 101）**：增量和重放都把第 4 条经历的状态错误推进到 6；修复后直接对照完整最后正确状态，均保持在 4。回归提交 `2aedc70970006ca46bd8826c3ac07b1f697d4223` 先于修复提交 `bcd035d91af73b6c7dd9ce5bb195bff5c082b042`。

另三项测试覆盖错绑定、手动暂停、成功修复后补算及后续成长、新检查/两种复核结论/新版本不能代替修复，以及修复再次失败与过期引用。`tests/boundaries.rs` 还验证内核仍有真实计算故障时普通事件不会重新调用它；`tests/properties.rs` 将未解决故障纳入 I23 全控制记录比较；`tests/controls.rs` 更新显式修复输入。`verify.ps1` 仅增加可选 TestFilter，最终全量测试不使用过滤器。README 说明机制边界，正式修复记录的认证与 H/L 接线追加到请求文件。

## 验收与证据

### 本轮 main 合入的影响与检查

B→M 的 #108 改动完整 θ 的导出/恢复、core-types 契约与生成物、core-store 版本/写锁/夹具、fact-view/gateway 夹具及相关工作流，不能按文档增量复用旧工作区结果。本轮在 merge 提交 `000e3c09` 串行重跑以下组合检查；没有修改契约或三语言生成器，P5-26 也未登记进生产调用链，因此不把 #108 的跨语言/存储证明重新签发为本卡生产验收。

| 模式/命令 | 本轮结果 | 原始证据 |
| --- | --- | --- |
| Format：模块 rustfmt --check | 通过 | `verification/conditional-20261101-format/` |
| WorkspaceFormat：cargo fmt --all -- --check | 通过 | `verification/conditional-20261101-workspace-format/` |
| Clippy：harness --all-targets -- -D warnings | 通过 | `verification/conditional-20261101-clippy/` |
| WorkspaceClippy：cargo clippy --workspace --all-targets --locked --offline -- -D warnings | 通过 | `verification/conditional-20261101-workspace-clippy/` |
| Test：同树 harness 全部测试 | **68/68 通过、0 ignored；384 前缀逐位比较零差异** | `verification/conditional-20261101-test/` |
| WorkspaceTest：cargo test --workspace --locked --offline -- --nocapture --test-threads=1 | **通过，退出码 0**；含 P5-23 Python 原入口对照 | `verification/conditional-20261101-workspace-test-detached/` |

工作区前两次运行在工具会话失联后没有留下退出码或完整结果，均未计为通过。原始部分日志保留在 `verification/conditional-20261101-workspace-test/` 与 `verification/conditional-20261101-workspace-test-retry/`；[首轮中断记录](conditional-20261101/workspace-interruption.json) 和 [第二次中断记录](conditional-20261101/workspace-retry-interruption.json) 将确切终止原因记为未知。随后用独立隐藏 PowerShell 进程，待已有本机 Rust 检查结束后，以同一提交、原命令在新目录串行重试；没有修改测试、阈值或忽略项，见 [启动记录](conditional-20261101/workspace-detached-launch.json)。

#108 已提供 `dynamics::export_snapshot` / `restore_snapshot`（显式 Limits、完整参数及资源绑定）、ThetaSnapshotV2 语义边界与 core-store 读写/恢复支持。原“缺完整 C 导出接口”请求已由该上游能力解除；**P5-26 还没有消费该接口、认证真实 H/资源、完成独立重算或迁移**。既有测试的 Debug 比较仍只是测试方法，不是规范持久格式。此处据 [ADR 0029](../../docs/adr/0029-theta-estimator-storage.md) 和当前源码更新边界，不将上游能力等同整卡闭合。

M 另含 ADR 0033：D12-B、D04-A、D20-A、D13-A 已选定，D20/D13 升级 B 仍需测试和用户批准标准。**设计选择不等于新规律实现或验收**；D07/D09/D10 后继公式仍不在本次实现范围，旧 EV7、I27 排除、EV9 布尔权重不能成为现行生产规则。

收尾 fetch 所见 main 为 `ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5`，比 M 仅新增 #127。已读六个文件的完整 diff：ADR 0034 条件载体切换、清除 KV/循环推理状态要求、P0-12 测量卡和 P5-13 依赖，不改变 P5-26 卡、重算/G/C 源码、数值契约、显式输入夹具、依赖或检查命令。本次不连接载体，正式 F/J 校准仍为后继项；按 AGENTS §6 判为对本次检查无影响，不追加 merge 或重跑，也不产生新验证基线，实际被测 main 仍是 M=`67cc296c`。见 [收尾影响核对](conditional-20261101/main-closeout.json)。

N1 非阻塞建议回应：认可只检查新增提交可减少重复校验；本轮暂保留现有实现，理由是用户限定本次为交付边界调整及合入后的组合验证，保留已复审源码便于核对 R1 结论。该简化可随后继机制修订配套非法追加回归并增量复审，不作为本次条件性交付阻塞。

### R1 修复时的历史验证（基线 B）

检查串行执行。每次命令、工具链、基线、开始/结束 HEAD、源码 SHA-256、stdout/stderr 和锁文件保留于 [`verification/`](verification/)。入口为 `pwsh -NoProfile -File core/crates/core-kernel/src/recompute/verify.ps1 -Mode <模式> -RunId <目录名>`；所有数据为合成数据或临时存储。最终源码和原始日志另存新 ZIP，旧归档不改写。

| 命令或检查 | 该轮结果 | 证据 |
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
| v2 §12；V2-97、V2-101；AC2-20/21 | 无既有故障时，一致/缺失/失败/超时均应用并报警；明确不一致或计算故障持续保留整个旧状态，只有绑定修复补算成功才解除；历史重放读取旧结论 | R1 复审关闭；**完整独立实现为未完成后继项**，不可用分支不是验收替代 |
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

R1 历史验证的源码清单与六项检查记录逐文件一致（15 个文件，含 13 个 Rust 文件），见 [source-check.json](review-r1-20261101/source-check.json)。原问题复现使用的 engine/types/旧适配与旧 test-04 清单一致，见 [regression-origin.json](review-r1-20261101/regression-origin.json)。该轮基线与收尾记录保留于 [baseline.json](review-r1-20261101/baseline.json)、[main-closeout.json](review-r1-20261101/main-closeout.json)，只描述 R1 当时的 B，不代表本轮 M。

R1 历史汇总见 [result.json](review-r1-20261101/result.json)。其 [源码归档](review-r1-20261101/source.zip) 与 [原始日志归档](review-r1-20261101/raw-verification.zip) 共 60 个文件已核对，见 [archive-check.json](review-r1-20261101/archive-check.json) 和 [归档哈希](review-r1-20261101/artifacts-sha256.json)。旧通过/失败日志及归档均不改写；本轮组合证据另存。

本轮被测组合提交为 `000e3c0990b04016d7ebc6180c10ddfab4232f78`；recompute 源码与已复审交付 `7142adc4` 相同，上游 C/契约/存储来自 M。后续报告提交不改变被测源码。[本轮汇总](conditional-20261101/result.json) 记录六项完整通过结果，八次运行的 15 个源码文件清单均与已复审源码一致；两次不完整运行不计为通过。[本轮原始归档](conditional-20261101/raw-verification.zip) 内 36 个文件逐字节核对通过，见 [归档核对](conditional-20261101/archive-check.json) 和 [原始文件哈希](conditional-20261101/raw-sha256.json)。源码继续引用未改动的 R1 归档，不重写旧证据。

真实 H 接线、V2→V3 迁移/回退、完整独立第二实现、SYS17 与 ADR 0032 新规律保留为后继项；整卡未完成，禁止据此接入生产。受影响检查已通过；按本轮用户明确授权，本次条件性交付转为可审查、标 `needs-user` 并留言“请审查”，请求审批当前限定交付；不以此自行合并、登记整卡完成或启用生产。结束后停止本卡。
