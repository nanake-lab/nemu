# P5-70 F：依据替换与截断归属 A1 — 报告

## 结果

原 head `38646315` 的独立交叉审查已通过（[评论](https://github.com/nanake-lab/nemu/pull/151#issuecomment-6771847710)）。两项非阻塞建议均已采纳，本轮代码提交 `723b14f5fa913d60026fbcef76da804ce9824bb6` 等待增量复审；不把原审查结论沿用为新 head 的通过。保持 draft / needs-user，由用户合并。

同一件事只保留一个依据槽，重访替换实际份额；越界损失归本次分量，其他槽不动。可见 x 是主值，停止事件后直接使用原 Φ，不从各槽重建 x，不定时清零。旧快状态迁移只保留 x 与锚点，槽为空，并返回绑定新旧快照内容的迁移记录。

基线：`51298e9ce5d571e8753d047eaac864fce007b867`（已合并 P5-68）；初版代码验收提交：`7a94bda3b527764f23d23e4de08e87adc73e2378`。分支 `task/P5-70-f-contribution`；[PR #151](https://github.com/nanake-lab/nemu/pull/151)。修改范围仅为 `core/crates/core-kernel/src/fast/` 与本报告目录。

## 2026-11-11 审查建议回应

1. **采纳：复用边界规范字节。** `codec::check` 返回已做结构/语义校验的规范字节，State/Effect 构造复用；State 的字节入口直接校验输入，不再走“解码成消息→重新编码→再次校验”的路径。保留 Schema 语义检查和生成类型的最终解码（现有 API 的语义层使用 Schema value，类型层仍需 decode），不新增运行时 JSON 依赖或通用框架。既有有限性、角色、版本校验保留；补查 -0→+0、无效字节与空治理引用拒绝。
2. **采纳：加强 §9(12) 夹具。** 移至 `tests/revision.rs`；合成 H 分别保存 θ、R⁺ 视图、解释和治理对象，以完整规范引用为键。θ₀…θ₃ 的 g 和 R⁺ 的 s 分别为 1/8、1/4、1/2、3/4；逐步解析对象并重新计算记录目标，连接目标为 0.09375→0.1875→0.375→修订重访 0.4375→当前更新 0.09375，后两步学得坐标为 0.125/0.5625。分别仅错配 θ₀/θ₁ 引用、仅错配 R₀/R₁ 引用，四个反例均需拒绝；输入不变。修订前重访、修订后重访及最后再重访也逐步核对引用。仍不是正式 H 存储验收。

同步核对：实际基线 B=`51298e9ce5d571e8753d047eaac864fce007b867`，最新 M=`31e6f71231e68b8e5464603a68d0b55a10d4a906`。B 是 M 的祖先；全部新增变化仅 `tasks/P4-voice.md` 的 IndexTTS 候选登记，不改变本卡规格、契约、构建或测试，故未合入 main。本轮重跑 core-kernel、clippy、fmt，结果与私有证据见下方补充。

本轮验收：最终代码 723b14f5 全包 257 passed / 0 failed / 1 原有 ignored（336.19 秒），clippy（1.32 秒）与 fmt 通过。首轮 31510bcd 全包 257 passed / 0 failed / 1 原有 ignored（697.08 秒），但本机 clippy 与 CI 42857935159 因夹具两处 StoredTheta 的 clone_on_copy 失败。去掉两处多余 clone 后重验，未放宽 lint；失败日志保留。最终测试使用 RUST_TEST_THREADS=4、构建 jobs=2，仍只运行一个重负载命令。证据见 EVIDENCE.md 的审查建议增量；当前代码 CI [42860287658](https://github.com/nanake-lab/nemu/actions/runs/42860287658) 仍在运行，不计作通过。

## 交付物

- `fast/contribution/mod.rs`：显式规律资源视图、目标计算、A1、共用锚点的 Φ；无 κ 参数或默认参数。
- `fast/contribution/codec.rs`：P5-68 规范编码与语义校验、快照恢复、坐标对齐；不重复定义跨进程类型。
- `fast/contribution/event.rs`：依据替换、按首次作用顺序重访、解释器修订后的目标/前态绑定；返回待提交作用记录。
- `fast/contribution/maintenance.rs`：别名有序合并、精确全零移除、F_out 与旧快照迁移准备。
- `fast/contribution/tests/`：16 项测试、独立 Python Fraction oracle 和可再生夹具。
- `fast/mod.rs` 正常登记模块；`fast/space.rs` 提取原 Φ 的 time_pull，`fast/drive.rs` 共用原 D 运算。旧入口与原测试断言保留。

新表示标识为 `echo-core-ADR0035/fast-state/1`；快照与作用绑定显式规律资源引用。盒边界、零与 schema_version 是结构常量，不是调参默认值。新增手写模块均不超过 400 行；oracle.json 为生成的测试数据。

## 初版验收（7a94bda3；本轮结果见上方）

最终运行信息与逐文件哈希见 [EVIDENCE.md](EVIDENCE.md)。

| 命令或检查 | 结果 | 范围 |
| --- | --- | --- |
| `cargo test --manifest-path core/Cargo.toml -p core-kernel --locked --offline` | passed | 257 passed / 0 failed / 1 原有 ignored，733.12 秒；含 16 项新增测试 |
| `cargo clippy --manifest-path core/Cargo.toml -p core-kernel --all-targets --locked --offline -- -D warnings` | passed | 本包与测试，14.77 秒 |
| `cargo fmt --manifest-path core/Cargo.toml --all -- --check` | passed | 工作区格式 |
| `python core/crates/core-kernel/src/fast/contribution/tests/generate_oracle.py` | passed | 229 向量、3125 距离比较，夹具再生 SHA-256 相同 |
| CI：正常 Rust 构建、workspace clippy / tests | passed | [run 42839385288](https://github.com/nanake-lab/nemu/actions/runs/42839385288)，代码提交 7a94bda3；现有 core 路径规则覆盖 |
| 真实时间、关机重启、H/L 原子发布、正式身份来源 | not_run | P5-75；本卡只有纯函数与规范序列化 |
| 跨时间传输重发消费键 | not_run | P5-72/73/75；不是 F 跨时间幂等 |
| 闭环、三语判断质量、真实载体 | not_run | P5-71/76/77 各自范围 |
| 独立交叉审查 | passed（原 head）/ not_run（本轮增量） | 原 head 38646315 已通过；723b14f5 请求原审查方增量复审 |

Windows、Rust 1.99.0、Python 3.12.10；全部合成状态，构建目录 `D:/Dev/cache/P5-70/target`，未打开排练或正式存储。重负载串行，`CARGO_BUILD_JOBS=2`、`RUST_TEST_THREADS=1`。第一轮与最终边界修改后的结果分开保留，不用第一轮代替最终代码验收。

第一轮为 257 passed / 0 failed / 1 ignored，耗时 706.98 秒。ignored 是仓库原有 `dynamics::tests::reference::theta_1_original_500_streams_zero_violations`（V2-93 的 Python 舍入诊断），本 PR 未新增跳过。原日志中旧 LAW-6 假设的 3064 个反例是既有保留的失败诊断，包内测试按既定预期通过；不将它改写为本卡反例消失或新规律全局通过。

## 规格对照：ADR 0035 §9 的 P5-70 全部条款

下表实现与测试路径相对于 `core/crates/core-kernel/src/fast/contribution/`。有限夹具通过不等于穷举所有 binary64 输入。

| 条款 | 实现位置 | 测试 |
| --- | --- | --- |
| 1 B1/B2，§4.4 | `State::project_time` 直接调用旧 Φ；槽共用 time_pull、learned_rate 与锚点 | `tests/mod.rs::b1_b2_b3_projection_uses_original_phi_and_shared_anchor`：盒外 h、最大有限数、次正规、不同基线/速率；零间隔规范字节恒等，重复读取一致 |
| 2 B3/B6/B6′ | `mod.rs` 槽衰减与 coordinate；`maintenance.rs::merge_aliases` | 上项及 `oracle.rs::b6_fixed_target_and_b6_prime_updated_history_enclosures`：固定目标从零、无合并才断言当前目标界；修订使用历史区间，合并逐步舍入；R4 与 0.1+0.2 合法 |
| 3 B4，§4.5 | coordinate，新槽 h=+0 | `b4_b5_first_effect_matches_old_f_and_repetition_is_bitwise_idempotent`：首次作用与旧 apply_drive 各坐标位模式一致 |
| 4 B5 | 内部支直接 h=v，饱和支夹至 h/v 区间 | 上项规范字节幂等、朴素写法变异；229 个 oracle 向量再作用逐位相同；不写跨时间幂等 |
| 5 E09/U1 | x 独立于槽的 Φ，A1 本次损失归属 | `e09_r4_saturation_revisit_and_target_revision_witnesses`、B1/A0 反例、`e09_binary64_matches_independently_rounded_rational_vectors`；误差包络与分解残差分报 |
| 6 §4.2 | `State::revisit` 以 first_position 排序；P5-68 role/basis 校验 | `histories.rs::e09_first_recall_shares_basis_surface_ids_and_internal_updates`、`revisit_order_is_first_effect_order_not_input_or_canonical_order`；原事件/首次回想单槽、背景双算变异失败、给定身份的表面变化不增槽；内部目标更新合法/重访非法 |
| 7 §4.3 | Effect 固化目标；重访不以当前 θ/R 重算；align 与 f_out | `boundaries.rs::f_out_aligns_current_and_target_but_revisit_reads_recorded_coordinates`：维度退出/进入从零，记录保留目标；补解释/后续报告用 TARGET_UPDATE |
| 8 §4.7 | active 失效过滤；有序别名合并不改 x；精确零移除；selected 绑定最近目标前态 | `invalidation_alias_merge_zero_removal_and_reentry` 与解释器测试：极小非零保留、全零移除后重入和保留路径字节相同；无 ε、容量淘汰 |
| 9 §4.8 | `State::migrate` 空槽、原 x/锚点，校验新旧内容摘要及长度 | `migration_preserves_x_anchor_empty_slots_and_binds_record_to_content`：拒绝错误摘要、unknown 标记、跨版本解码；只是迁移准备，不声称 SYS17 存储回退通过 |
| 10 AC2-15/I05，§4.6/§8 | 扩展状态 x/h；新目标无 κ；旧入口保留 | `ac2_15_i05_equal_slots_same_drive_and_different_decomposition_counterexample`；Fraction 3125 组 d◇ 比较、同槽 dyadic 同驱动比较、同 x 不同槽反例；`target_has_no_kappa_and_preserves_legacy_external_arithmetic` |
| 11 接触/缺席 | 同一依据目标更新，不同依据各占槽 | `contact_report_cadence_fixed_and_changed_targets_respect_historical_interval`：密/疏时间表冻结 a、μ、人物/w、θ/R⁺；固定/变化目标分开；身份为合成真值，不是识别质量测量 |
| 12 解释器前态序列 | selected 绑定最近目标更新 θ/R⁺；重访使用本次治理引用 | `revision.rs::interpreter_revision_uses_latest_target_prestate_and_next_update_uses_current`：θ₀→θ₁→θ₂→修订→重访→θ₃；错配 θ₀/θ₁ 拒绝；解释/前态引用与合成记录核对；真实 H 解引用归 P5-75 |

## 其他不变量与证据边界

| 条款 | 实现/检验 |
| --- | --- |
| I03 | F 只返回快状态与待提交记录，θ/R 只读借用；不写权威存储；错误时原输入不变 |
| I04 | x/h 共用锚点，本卡验证零时距作用；事件消费键与锚点同事务发布 not_run，归 P5-72/75 |
| I18 | 无私有持久缓存；State 仅包装版本化快照，HistoryBasis 是借用 H 视图；目标/前态/解释/治理引用在记录里 |
| I21 | EffectiveEffects 为只读作用视图；投影返回新快状态，不写 R/θ |
| I23 | `i23_incremental_serialized_restart_and_full_record_fold_match_bits`：同一记录/时间序列，逐次规范恢复与整段 fold 字节相同；不是磁盘事务或进程重启验收 |
| I24 | 本卡不写 R/θ、不将 x 反馈作学习证据；学习侧的完整验收不在本卡 |
| I29 | 仅使用合成夹具，不含用户数据；谱系存储权限隔离 not_run，非本卡 |
| v2.0 §6 数值边界 | 原有限 binary64 路径，显式运算顺序，无 FMA/fast-math；`numerical_and_contract_boundaries_reject_without_mutating_input` 检查非有限、溢出与输入不变 |
| v2.0 §6、ADR 0008 §1、ADR 0032 D09 | 旧 Φ/F 保留，新 I05 按 ADR 0035 分项，不假定只看 x 足以确定扩展状态 |

独立 oracle 不导入 Rust 递推：Fraction 构造精确 A1，逐算子舍入给出 binary64 预期；229 个向量，随机种子 5070。最大分解残差 `2113/36893488147419103232`（约 5.73e-17），不是零；每向量分别记录 x/h 对实数结果的舍入包络。3125 组 d◇ 属有限网格核验；B6′ 逐步舍入见证不升级为任意历史的穷举证明。R4 的跨时间重访 h 从 0.25 增至 0.625 是已接受行为。

[PR #146 审查记录](https://github.com/nanake-lab/nemu/pull/146#issuecomment-6742621711) 的角色边界已落实：内部依据不能 REVISIT，但 TARGET_UPDATE 合法；重访回执使用本次治理引用。全部生成消息走既有语义校验。

调用方须解析真实资源与 H，确认当前解释器、依据归属、失效/别名治理、最近目标记录及消费资格。本模块检查输入值可判定的契约/版本/前态绑定，不访问数据库、不猜身份、不验证引用对象真实存在、不生成消费键。不能把本卡测试称作真实端到端通过。

## 新增依赖

无。serde_json 是已有测试依赖；未改锁文件。

## 待用户确认

- [ ] （阻塞）新规律进入核心，由用户在独立审查通过后合并；决定已有 ADR 0035 和本卡承载，无新增规格读法。
- [ ] （阻塞）本轮增量复审通过；原 head 38646315 已通过独立交叉审查。

## 后续建议（不在本任务范围内）

按既有划分完成 P5-72 消费键、P5-75 真实 H/存储/时钟接线与迁移回退、P5-76/77 闭环与判断质量。这里不新增心理重置、硬容量、自动猜分解或默认资源。

## 下一步

保持 draft / needs-user，在 PR 写“请审查”后停止。未合并，不领取下一张卡。
