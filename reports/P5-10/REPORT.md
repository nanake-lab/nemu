# P5-10 快状态 F（v2.0 第 6 节）— 报告

## 当前状态

2026-10-26：PR #94 的四项审查问题及直接事件映射测试已修复，本卡全部测试和 Rust 工作区本地检查通过；高难度交叉复审尚未完成。保留草稿和 `needs-user`，不宣称可以合并。F 的纯函数接入 `process_head` 属于 P5-19。

分支 `task/P5-10-fast-state` 已通过 merge 同步 `origin/main`：先合入 `716f7f0`，随后合入新到的 `2bc04ba`，第二次同步提交为 `9534b7f`；未 rebase 或强推。以下最终检查均在第二次同步后运行。本次自有修改只涉及 `core/crates/core-kernel/src/fast/`、`reports/P5-10/`、`docs/traceability/P5-10.md` 与差异登记中的 V2-115。上游 merge 引入的其他目录变化不属于本卡修改。

## 审查修复

对应 [2026-10-26 审查评论](https://github.com/nanake-lab/nemu/pull/94#issuecomment-6408523309)：

1. **驱动检查与使用值绑定。** 检查作用 ID 规范次序及逐项有限性，在同一循环归约绝对值和与有符号和；实际驱动使用重算值，传入 `f_sum` 仅作一致性校验，不一致返回 `InconsistentSum`。检查 η、θ 驱动参数、作用幅度及加权前后的 nature/learned 驱动；learned 的 `sμ` 必须在 [0, 1]。即使 κ 权重已下溢为 0，非法原始驱动仍拒绝。`Drive` 对外只读，避免绕过检查。合法驱动之后的 F_in 盒投影保留。
2. **λ_k 防溢出。** 保留参考加权平均：选最大 |d_i|（并列取首通道），权重除以它，以该通道 λ 为初始均值；其余通道依固定次序更新 `mean += ((λ_i - mean) * weight) / accumulated_weight`。累计权重至少 1、新增权重至多 1，正有限速率的差不溢出，每次更新不超过剩余距离的一半；先乘后除保留极小方向与极大速率的乘积。检查派生速率有限、正且位于原速率范围；`λΔ` 溢出及非有限投影显式报错，输出再次验证。测试经正常 `compile_authorized_effects` 构造 1e308、最大有限数、最小正常数与最小正 subnormal，另含独立参考值及 10,000 组范围/参考比较。
3. **零的合法域统一。** `FastState::new` 将 nature/learned 的 -0 规范为 +0 后校验，恢复、解码及算子输出复用该入口。直接字段构造的非规范零由 `validate`、`serialize` 拒绝。CanonicalV1 本身拒绝带 -0 的非规范字节，解码不会返回带 -0 的状态。测试覆盖两类分量的负零、构造、固定初态恢复、快照往返及恢复后 Δ = 0 位型恒等，保留负 subnormal 和其他非零位型。
4. **V2-115 已确认。** 用户 2026-10-26 明确采用“多人取最高亲近度、无人 w = 1”，同意 κ^n 下溢取 0。登记改为“用户已确认（2026-10-26）”，对应实现不变，删除原“不阻塞”待确认项。A 缺失仅取消天性评估项的既有实现继续按本卡与施工合同处理。
5. **直接事件映射。** 新增 10,000 组 Δ = 0 性质测试，直接检查同一 D 的 F_in 及随后 F_out 不放大距离，覆盖四种事件枚举与不同 learned 维数；使用盒内平移/减法的显式 `4 * f64::EPSILON` 舍入余量，不借助时间收缩余量。另有固定数值、含饱和的独立正例。原有测试、断言与阈值全部保留。

## 交付物

- `core/crates/core-kernel/src/fast/`：状态校验、Φ_x、F_in/F_out、距离、快照、事件分类驱动、共情权重与恢复纯函数。
- `fast/tests.rs` 与 `fast/tests/{drive_bounds,rates,state_boundaries,event_map}.rs`：31 项 fast 测试，本次新增 12 项；core-kernel 共 43 项。原测试文件超过 400 行，保留原验收与共用夹具便于复核，新增案例分入四个小文件。
- `profiles/structural_fixture/fast/fixture.json`：原有 b = 0、λ_i = 1/s、g_i = 1 结构夹具，本次未改；只能证明逻辑性质，不能作为行为有效或正式谱系证据。
- [规格对照表](../../docs/traceability/P5-10.md)、[差异登记 V2-115](../../docs/traceability/diff-register.md)、[复验脚本](verify-review-fixes.ps1)、[原始日志](review-fixes-2026-10-26/) 与按验收编号整理的结果。

## 验收

卡片列出验收性质，没有独立命令行。core-kernel 命令覆盖本卡全部测试；另运行 Rust 工作区格式、静态检查及测试。主代理串行执行重负载；routine_reporter 只做路径、日志摘要和测试名的只读核对。

| 命令 | 结果 | 原始证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过 | [fmt](review-fixes-2026-10-26/fmt-20261026-131617.log) |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过 | [clippy](review-fixes-2026-10-26/clippy-20261026-131618.log) |
| `cargo test -p core-kernel --locked --offline` | 43 通过，0 失败、0 忽略 | [kernel](review-fixes-2026-10-26/kernel-20261026-131607.log) |
| `cargo test --workspace --locked --offline` | 98 项单元/集成测试及 2 项 compile-fail 文档测试通过，0 失败、0 忽略 | [workspace](review-fixes-2026-10-26/workspace-20261026-131653.log) |

环境：Windows，锁定 Rust 1.99.0；`CARGO_HOME=D:\Dev\cache\cargo`，构建与临时存储位于 `D:\Dev\cache\P5-10\`，jobs=2、测试线程=1。未打开用户排练或正式存储。全工作区测试用时 1043.88 秒，其中 core-store 1017.07 秒；全部跑完，未跳过慢用例。[测试输入哈希](review-fixes-2026-10-26/tested-inputs.json) 绑定本次源码和夹具。首轮因新增测试误用不存在的 `Evaluation::default()` 编译失败；改为现有 `evaluate_effects` 后通过，[失败原始日志](review-fixes-2026-10-26/kernel-20261026-131414.log) 保留。

| 验收项 | 结果与范围 | 证据 |
| --- | --- | --- |
| AC2-15 / I05 | 通过：200 组 × 30 步含事件区间收缩、无事件回落、10,000 组 Δ = 0 直接非扩张及独立正例 | [AC2-15](AC2-15/result.json) |
| F01、施工合同 10.6 | 通过：时间分段一致性原容差 1e-12、同锚点读取不累计衰减、合法域 Δ = 0 逐位恒等、恢复后快照位型一致 | [F01](F01/result.json) |
| F02 | 通过：20,000 组投影/驱动后在 X 内；错误总和、learned 超界、非有限 η/贡献、极端 λ、非规范零拒绝路径 | [F02](F02/result.json) |
| F03（结构） | 通过：近期历史停止驱动后按 λ_min 遗忘，λ = 0 拒绝 | [F03](F03/result.json) |
| F05（纯函数） | 通过：F_out 保留/删除/补零，同事件产生同一单次脉冲；不等同于事务去重验收 | [F05](F05/result.json) |
| v0.6.4 16.17（结构） | 通过：盒空间界、时间分段与近期扰动收缩由 F01–F03 覆盖；learned 维度跟随 R⁺ | [16.17](16.17/result.json) |
| 施工合同 10.5 | 通过本卡部分：A 缺失只去掉天性评估项，非法驱动拒绝且不伪造脉冲；降级审计归 P5-19 | `missing_appraisal_keeps_only_the_tendency_effects`、`f02_*` |
| D 不读 x̄ | 通过：函数签名无 FastState，编译期函数指针约束 | `drive_signature_has_no_fast_state` |
| ADR 0026 §6.3、V2-115 | 通过纯函数检查：事件权重、n 转移、多人/无人、κ 下溢 | `self_caused_*`、`internal_count_*`、`empathy_weight_*`、`kappa_power_*` |
| 快照与恢复 | 通过：CanonicalV1 逐位往返、未知版本拒绝、固定 x_init 重置及 washout 判据 | `snapshot_*`、`state_boundaries::*`、`recovery_reset_and_washout` |

固定求值次序与锁定 libm 是确定性基础；跨目标逐位兼容仍需黄金向量/回放证据，不能推及任意平台。κ 下溢用例不等于 COG-A12 硬件时间包络验收。系统记录/超期结果绕过 F、Q 绑定、实际 n 持久化、事务去重与降级审计仍属编排。当前 head 的 CI 与复审以 PR 检查/结论为准，旧 head 的通过不替代新 head。

## 规格对照与不变量

完整条款、实现与测试映射见 [docs/traceability/P5-10.md](../../docs/traceability/P5-10.md)。

| 不变量 | 本卡检查方式 |
| --- | --- |
| I02 / I03 | 只读输入与返回值，无数据库写能力；只产出 x 或 drive，外层事务负责提交 |
| I04 / I21 | 同锚点纯投影；F_out 不再驱动，重复读取一致；事务去重留给 P5-19 |
| I05 | D 签名无 x̄；AC2-15 直接映射与含事件区间测试 |
| I06 | 沿用 P5-27 的 R⁺ 编译，原 `i06_candidates_have_no_effect_on_x_or_m` 随测试运行 |
| I18 | 构造/检查为无持久状态纯函数，不增加缓存或隐藏状态 |
| I29 | 结构夹具、固定随机种子、临时存储 |

本次未改 IDL、跨进程契约、黄金向量或快照格式，表示版本仍为 `echo-core-v2.0/fast-state/1`。修复对象是未合并 PR 内的数值求值与输入边界。无新增差异编号，仅落实 V2-115 确认。

## 新增依赖

本次无新增依赖、清单或锁文件改动。原 PR 的 `libm =0.2.16`（MIT/Apache-2.0）用于 expm1/exp，为 P5-09 已用版本；`serde_json =1.0.154`（MIT/Apache-2.0）为工作区已有依赖，仅测试使用。

## 待用户确认

无未决规格读法；V2-115 已由用户于 2026-10-26 确认。仍等待独立交叉复审与当前 head 的 CI；实现方不代写“审查通过”。

## 后续建议（不在本任务范围内）

- P5-19：外层事务、时钟/锚点、内部计数 n、降级/连续性中断审计、事件去重；系统记录与超期行动结果不调用 F。
- P5-38：行动发出时的 n 随 Q 的在途行动保存，传给 `EventClass::ActionResult`。

## 下一步

本轮验证已完成，交付到 PR #94 并逐项请求复审。等待当前 head 的 CI 与用户安排的独立交叉复审；本轮不合并、不认领下一张卡。
