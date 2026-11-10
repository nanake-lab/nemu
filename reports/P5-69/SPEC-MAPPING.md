# P5-69 规格对照

仅使用合成 H 的已治理输入投影；测试名描述实际覆盖，不代表生产来源治理、运行时或真实体验已完成。下表的路径均在 `core/crates/core-kernel/src/dynamics/` 内。

| ADR 0035 §9 P5-69 条款 | 实现 | 测试与边界 |
| --- | --- | --- |
| 1：§3.2 规范次序、b 整条递推渐进下溢、其余故障不变 | `update.rs::statistics`：S→P→四通道残差/方差/σ/g/时长/λ→U/b；`numeric.rs` 原有规则保留 | `d07_1074_step_rational_oracle_and_gradual_underflow_golden`；`d07_subnormal_stagnation_iff_ties_and_normal_boundary`：P、U、P·b₀、最后商的下溢正例；残差平方下溢仍拒绝 |
| 2：出生份额实数界、P 非增、严格减小前提、次正规停滞 | `types.rs::prior_mass`、`update.rs` | `d07_birth_share_real_oracle_and_binary64_mass_paths`：4 个 P₀、3 个 ρ、256 前缀；Python Fraction 独立检验两条实数界，逐操作舍入 P/S 另比位；次正规 k=1…64 × ρ∈{1/2,3/4,7/8} 按整数不等式/中点奇偶判定及停滞界；最小正规数边界单列 |
| 3：AC2-05 无合格经历，全估计器不变 | `mod.rs::stage` 的 `None` 与排除来源分支 | `ac2_05_14_qualified_projection_only_and_duplicate_mutant`；A 缺失由调用方传 None，不能伪造 a=0；被排除输入即使时间为 u64::MAX 也不推进状态。真实时间：not_run（P5-75） |
| 4：AC2-14 自致结果、念头、桌面观察不改变 P/S/U | `types.rs::ReactionSource` / `learns` | 同上；观察类别与同次接触/缺席的身份判定由上游提供，纯 C 不认证这些声明 |
| 5：E07 固定 H、出生差/有限扰动、冻结与动态 ρ 分报 | `update.rs` 与现有 `detector.rs` | `e07_fixed_h_frozen_rho_birth_difference_and_finite_perturbation`：P₀=2、ρ=1/2 的 32 步二进有理数精确等式，α=1/2；`e07_dynamic_same_h_birth_difference_rational_envelope`：128 步动态 ρ 的同历史不同出生值对照，独立 Fraction 检查逐步差异包络；`e07_dynamic_rho_sign_symmetry_and_permanent_world_change`：128 步动态检测器、正负对照和永久改变；不据此签发反馈世界 AC2-25 |
| 6：§3.4 商缺陷恒等式、新包络、严格越界、旧项变异 | `bounds.rs::common_discount`、`StepBounds::check`；复用 `interval.rs` 外向舍入 | `d07_new_defect_bound_strict_rejection_and_old_prior_term_mutant`：a=b、δ=0，新界拒绝旧固定先验漂移；界上允许、next_up 拒绝；`d07_adaptive_extremes_stay_within_derived_bound`：9 条 ×128 步、全部12分量；`d07_outward_bound_covers_subnormal_quotient_and_range_fallback`：q/2 与大 P₀ 的范围回退 |
| 7：AC2-08/I23、AC2-18/19、AC2-20 | 同一 `stage` 逐事件递推；V3 导出/恢复 | `ac2_08_18_19_v3_prefix_restore_and_replay`：32 前缀整段重算；每前缀两种编码恢复并续算；暂停不调用 C、补算记录输入；来源更正从 genesis 重算与从未使用该输入相同。新版本重算通过显式新 genesis + 同一输入序列；真实 H/L 写入、暂停/更正编排及迁移：not_run（P5-75），不拿此测试冒充 AC2-20 全项 |
| 8：旧版本黄金向量、THETA-1/THETA-2 保留 | `DynamicsState::genesis`、V2 导出/恢复、旧算术分支 | 旧测试及黄金文件不改断言；唯一旧测试修改是在穷举状态比对中增加 P 的 Option/位值检查。既有 `**2` 诊断 ignored 沿旧授权保留，不记 passed |
| 9：五种最小变异分别检出 | 正式路径无变异开关 | 固定 n_b→出生差测试；重复质量→重复投影恒等测试及错误重新分类对照；x 替 a→反向信号对照且 API 无 x/m；按符号筛选→正负镜像同质量；截断继续→越界输入/候选返回 Err，强烈合法输入仍应用并报警 |
| 10：规范算术黄金向量、逐步有理数 oracle | `tests/d07-vectors.json`（测试夹具）、`tests/d07_oracle.py`（Python 标准库） | 1074 步每步严格比较 P/S/U/b 位值，并检查根据已舍入 P′/U′/S′ 构造的 N/T 精确有理数误差包络；q/2、(3/4)q、偶中点、最小正规数停滞分别断言。普通 cargo test 执行 Python `--check`，不写夹具 |

## 契约、版本及不变量

| 依据 | 实现与验证 |
| --- | --- |
| ADR 0035 §§2、3.6；P5-68 `law_state.proto` | 新数值路径 `ADR0035-D07-gradual-underflow-v1`；规律资源自己的 `law_id` 显式传入，不提供默认值。使用生成的 `ThetaSnapshotV3`、`ThetaEstimatorParametersV2`、原 `ThetaEstimatorState`；不新增 IDL/持久类型。V2/V3 导出互拒；V3 恢复核对 ObjectRef、完整参数规范字节和规范摘要，不猜 P |
| ADR 0029；ADR 0035 §3.3 | `storage.rs` 提取原有完整估计器映射供两版共用；保存实际 σ/方差/时长中间位值，恢复不重算。不改变固定 nσ/nτ 先验 |
| I13、I18、I23 | P 在已有 DynamicsState 中显式保存，None 表示旧版而不是缺失的新状态；V3 要求 Some(P)，包括 +0。C 只产生 StagedDynamics/ThetaChange，不写 L；全状态重放检查包含估计器、检测器、时间、游标和慢漂 |
| I24、I25、EV6 | 签名无 x/m/R；只消费已记录且无调制的 a。重复资格来自上游记录的治理投影，不加隐藏 seen-ID 集合；历史认证、同次接触解析不由本卡伪装实现 |
| I29 | 测试仅合成输入；没有读取排练或正式存储 |

## 浮点包络说明

新单步界使用 δ=(P·b₀+U)−b(P+S)，理想差为 [ρδ+(a−b)]/[ρ(P+S)+1]。另按实际 P′、S′、U′ 运算图构造区间 image；image−(b+identity) 包住本步舍入及区间宽度。绝对值相加并向外扩一格，覆盖最终差值舍入。每个乘除在零/次正规处也扩至相邻浮点，因此含最小正次正规数的绝对项；区间无效时只退回合法范围界，不改变候选、不判成长故障。它可能漏掉界内错误，不声称完备错误检测。

实数 β 趋零与 binary64 P 停滞是不同结论。有限测试只检验列出的前缀和边界；不补 ε，不把次正规残余当作永久先验的语义。
