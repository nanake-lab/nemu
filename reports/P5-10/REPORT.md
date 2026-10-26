# P5-10 快状态 F（v2.0 第 6 节）— 报告

## 当前状态
完成，等交叉审查（高难度卡）。F 的纯函数部分全部实现；接入 `process_head` 属于 P5-19。按 2026-10-26 追加说明，F 区分 ADR 0026 第 6.3 节的三类事件，自致事件按 κ^n 加权。接下来取 P5-12（J 编译与读出）。

## 交付物
- `core/crates/core-kernel/src/fast/`：
  - `types.rs`：`FastState`（四个天性通道 + R⁺ 各一维，按倾向 ID 的规范次序）、`Drive`、`EventClass`（新外源事件、外部的其他事件、内部记录、行动结果）、`EventInput`、`DriveLaw`、`FastError`、`ContinuityBreak`。
  - `space.rs`：`validate`、`project_time`（Φ_x，写成 x + (−expm1(−λΔ))(x_rest − x) 再截断，λ_k 取参考式）、`apply_drive`（F_in 的 Proj_X）、`f_out`、`distance`、`serialize`/`deserialize`（`FastStateSnapshotV2` 的 CanonicalV1 字节）、`REPRESENTATION_VERSION`。
  - `drive.rs`：`compute_drive`（D(e)，签名中没有 x̄；保守幅度检查用绝对值之和）、`kappa_power`、`next_internal_count`、`empathy_weight`。
  - `recovery.rs`：`reset_to_init`、`washout_sufficient`。
  - `tests.rs`：20 项测试。
- `profiles/structural_fixture/fast/fixture.json`：结构夹具（b = 0、λ_i = 1/s、g_i = 1），只用于逻辑测试。
- `core/crates/core-kernel/Cargo.toml`：恢复 `libm =0.2.16`（固定的 expm1、exp 实现，各平台逐位相同），加测试依赖 `serde_json =1.0.154`（工作区已有）。
- 差异登记 V2-115：共情权重取亲近度最高的人、κ^n 下溢取 0、天性评估缺失时的处理。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| AC2-15 / I05（ADR 0008 扩展）：同一 θ、R 与事件序列，两个随机初值在含事件的区间上 ‖δx(t)‖∞ ≤ e^{−λ_min t}‖δx(0)‖∞ | 通过（200 组 × 30 步，含四类事件与 κ 加权） | `ac2_15_contraction_holds_on_intervals_with_events` |
| AC2-15：没有事件时 x 松弛到 (b, 0)，距离单调不增 | 通过 | `ac2_15_without_events_x_relaxes_to_rest` |
| Δ = 0 时投影逐位不变（核验 D-2；施工合同 10.6） | 通过，含 1e−17、5e−324 等样例 | `zero_delta_is_identity_bit_for_bit` |
| F01：分段投影一致（容差 1e−12），同一锚点重复读取不累计衰减（10.6） | 通过 | `f01_piecewise_projection_and_no_accumulated_decay` |
| F02：随机投影与驱动后都在 X 内；NaN、越界、维数错配、超界驱动被拒绝而不是缩放 | 通过（2 万例） | `f02_*` 两项 |
| F03：两段近期历史在停止驱动后按 λ_min 收缩到 ε 以内；λ = 0 被拒绝 | 通过 | `f03_*` 两项 |
| F05：R⁺ 不变时 F_out 恒等；同一事件给出同一脉冲 | 通过 | `f05_*` 两项 |
| 结构检查：计算 D(e) 的函数签名里没有 x̄（ADR 0008 §1、PHI-7 无法表达） | 通过（编译期以函数指针类型约束） | `drive_signature_has_no_fast_state` |
| ADR 0026 §6.3：内部记录乘 κ^n、行动结果乘 κ^{n_发出}、外部事件原式；n 的转移 | 通过 | `self_caused_events_are_weighted_by_kappa_powers`、`internal_count_follows_the_event_classes`、`kappa_power_*` |
| 施工合同 10.5：天性评估缺失时不伪造驱动，只保留倾向作用 | 通过 | `missing_appraisal_keeps_only_the_tendency_effects` |
| 快照 round trip 逐位一致；未知表示版本被拒绝 | 通过 | `snapshot_*` 两项 |
| 恢复：重置到 x_init、washout 判据 2e^{−λ_min t} ≤ ε | 通过 | `recovery_reset_and_washout` |
| `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings` | 通过 | 本机 |
| `cargo test -p core-kernel` | 31 通过 | 本机 |

**环境说明：** 本机用 Rust 1.97.0（云端无法下载固定的 1.99.0），以 CI 为准。

## 规格对照
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| v2.0 §6 空间 X = [−1,1]^{4+\|R⁺\|}，learned 按倾向 ID 规范次序 | `types.rs`、`space.rs::validate` | `f02_*`、`snapshot_*` |
| v2.0 §6 时间投影 Φ_x，x_rest = (b, 0)，λ ≥ λ_min > 0，λ_k 参考式 | `space.rs::project_time` | `ac2_15_*`、`f01_*`、`f03_lambda_zero_is_rejected` |
| 核验 D-2：expm1 写法，Δ = 0 逐位不变，投影后截断 | `space.rs::relax` | `zero_delta_is_identity_bit_for_bit`、`f02_projection_and_drive_stay_in_x` |
| v2.0 §6 事件驱动 D_i、D_k、f_k（P5-27） | `drive.rs::compute_drive` | `self_caused_*`、`missing_appraisal_*` |
| v2.0 §6 幅度界 η_x，O §5.2 绝对值之和、超界拒绝不缩放 | `drive.rs::compute_drive` | `f02_invalid_inputs_are_refused_not_repaired` |
| ADR 0008 §1 事件映射非扩张、D 不读 x̄ | `drive.rs` 签名；`space.rs::apply_drive` 截断 | `drive_signature_has_no_fast_state`、`ac2_15_contraction_*` |
| v2.0 §6、O §5.3 F_out 只增删快分量 | `space.rs::f_out` | `f05_f_out_*` |
| O §5.3 污染恢复：固定 x_init 重置或已证明安全的 washout，记录连续性中断 | `recovery.rs` | `recovery_reset_and_washout` |
| ADR 0026 §6.3 三类事件与 κ^n | `types.rs::EventClass`、`drive.rs` | `self_caused_*`、`internal_count_*` |
| V2-115 共情权重取最亲近的人 | `drive.rs::empathy_weight` | `empathy_weight_takes_the_closest_person` |
| V2-31 λ 以每秒计，H 时间为整数纳秒 | `space.rs::project_time` | `f01_*` |
| V2-35 `FastStateSnapshotV2` | `space.rs::serialize` | `snapshot_round_trip_is_bit_exact` |

## 新增依赖
无新依赖：`libm =0.2.16` 是 P5-09 起就用的固定版本（MIT/Apache-2.0），P5-27 暂时去掉后在此恢复；`serde_json` 只作测试依赖，版本与 core-types 相同。

## 待用户确认
- [ ] （不阻塞）V2-115 的三处读法，尤其「涉及几个人时取最亲近的那一个」。

## 后续建议（不在本任务范围内）
- P5-19 接入编排时：维护内部计数 n（`next_internal_count`）；系统记录与超期的行动结果不调用 F；天性评估降级时记录 `A_DEGRADED`。
- P5-38：行动结果的 n_发出 记在 Q 的在途行动里，传入 `EventClass::ActionResult`。
