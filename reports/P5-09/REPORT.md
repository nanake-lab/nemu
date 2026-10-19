# P5-09 作用解析与授权（W04） — 报告

## 结果
完成。执行者：Claude 施工线（AGENTS.md 第 10 节）。合并前还需 Codex 交叉审查和 Windows CI。

## 交付物
- `core/crates/core-policy/`：scope DSL 与字段目录（`scope.rs`、`catalog.rs`、`endpoint.rs`），确定性相交检查（`overlap.rs`），策略对象的 CanonicalV1 编码与摘要（`canonical.rs`）。
- `core/crates/core-kernel/src/effects/`：
  - `compile.rs`：`compile_authorized_effects`，纯函数；
  - `evaluate.rs`：`evaluate_effects` 与按端点白名单构造的 `TypedContext`；
  - `admission.rs`：候选准入（`admit_candidate`）和 A_x 路由审计（`audit_drive_route`）；
  - `dag.rs`：依赖 DAG 解析；
  - `params.rs`：有界稀疏作用参数及其规范编码。
- `docs/traceability/P5-09.md`：规格对照表。
- `reports/P5-09/evidence/`：本地 fmt、clippy、test 日志与工具链版本。

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过 | `evidence/fmt.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | `evidence/clippy.log` |
| `cargo test --workspace --locked` | 通过（本卡新增 28 项） | `evidence/test.log` |
| J06 结构部分 | 通过 | `j06_*` 三项、`compile_budget_overflow_in_stored_state_is_a_fault` |
| F06 结构部分（路由审计） | 通过 | `f06_drive_proposal_may_not_reencode_an_r_or_theta_effect` |
| I07、I21 检查 | 通过 | `i07_*` 两项、`i21_*` |
| DSL 相交测试 ≥ 50 例 | 通过：47 例 × 双向 = 94 次，另 4 个 UNKNOWN_OVERLAP 用例 | `core-policy/tests/overlap_cases.rs` |
| 另覆盖 I06、I08、I12、I22 | 通过 | 见规格对照表 |

本地工具链是 Rust 1.97.0；仓库锁定 1.99.0，以 CI 结果为准（AGENTS.md 第 10.3 节）。

## 规格对照
见 `docs/traceability/P5-09.md`。

不变量检查方式：I03 由类型约束（本卡函数只返回值，不持有任何写能力）；I06、I07、I08、I12、I21、I22 由单元测试覆盖，并且 I07、I22 在编译路径上作为运行时断言（返回 `StructuralFault`）。

IDL 与黄金向量：未改动 `core/contracts/`。scope 和作用参数按 `EffectContract` 里的 `ObjectRef` 引用，它们的规范编码在 `core-policy` 内定义（见待确认第 2 条）。

差异登记：无新增。

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| libm | 0.2.16（`default-features = false`） | MIT | 纯 Rust 的 `exp`，使 q 包络衰减不依赖平台数学库（总方案第 6 节「固定超越函数实现版本」） |
| sha2 | 0.11.1 | MIT 或 Apache-2.0 | scope 与参数对象摘要；工作区已有，同版本 |

## 待用户确认
以下都是规格允许范围内的实现选择，不阻塞合并：
- [ ] （不阻塞）scope DSL 只支持 AND、OR，没有 NOT。规格写的是「有界布尔组合」；去掉 NOT 后表达力更窄，但相交检查更简单可靠，且偏保守。需要排除某些值时，用枚举集合显式列出。
- [ ] （不阻塞）scope 对象与作用参数对象的规范编码和类型标记（`echo.scope-predicate`、`echo.effect-parameters`）定义在 Trusted Core 内部，不是 IDL 消息。它们目前不跨进程。将来 G 的候选 worker（v1.x）若要提交 scope，需要把它们升为 IDL 并写 ADR。
- [ ] （不阻塞）作用参数第一版只支持常量稀疏向量。《核心算子》第 12 节也允许「x 的有界函数」，那种形式需要单独的解释器版本，留给需要时再加。
- [ ] （不阻塞）幅度用 L1 范数作保守上界（不小于 L2）。编译时按「当前包络系数 × 幅度界」累计；候选准入时对已有作用按完整幅度界累计，更保守。
- [ ] （不阻塞）字段目录（哪些字段、对哪些端点开放）属于 Ω 的 scope 解释器资源，由部署 profile 提供；本卡没有定义任何带语义的字段，测试只用结构夹具。
- [ ] （不阻塞）R 证书的有效期允许「无到期时间」，这时该作用只会因相位、失效屏障或包络归零而退出。

## 后续建议（不在本任务范围内）
- P5-18 收集验收用例时，把本卡的 J06、F06 结构测试纳入 `core/tests/acceptance/`；本卡的测试按目录规定放在两个 crate 内部。
- F06 的条件消融部分需要 P5-10（F）与 P5-11（A_x）接入后补做。
- `core-kernel/src/effects/tests.rs` 为 447 行，超过 400 行：它是按验收编号排列的测试表，拆开反而难以对照，故保留。

## 下一步
等待：
1. Codex 交叉审查（AGENTS.md 第 10.3 节，建议 Astra 档）；
2. Windows runner 上线跑 Windows 作业。

两者都通过后合并。接下来取的卡：P5-10（依赖 P5-09）。

---

## 交叉审查修复与更正（2026-10-19）

Codex（GPT-6 Astra）的交叉审查发现了 7 个阻塞问题，见 PR #22 评论 `issuecomment-6264645430`。修复在分支 `task/P5-09-review-fixes` 上完成。本节是追加内容；上文与本节冲突的说法，以本节为准。

### 结果
B1–B7 全部修复。审查给出的 9 个反例原样作为回归测试保留，修复前在 main 上全部失败，修复后全部通过。另按审查建议补了 17 个正反例。合并还需要 Codex 复审；复审通过后去掉 PR #22 的 `unverified` 标签。

### 逐项修复
| 编号 | 问题 | 修复 | 规格依据 |
| --- | --- | --- | --- |
| B1 | 接管只按责任 ID 全局抑制 R | 接管登记改为「责任 + 端点 + 已验证 scope 子域」。只有能证明落在子域内的 R 条目才被抑制；其他端点上的条目，以及能证明与子域不相交的条目，继续由 R 承担；既不在内也不相交的条目返回 `AmbiguousTakeover` 结构故障，不扩大抑制范围。binding 与 module、mechanism 合同的端点、scope、revision、参数、carrier、slice 逐项比对。禁用的 module 保留接管记录，它的子域也必须能验证。 | O 3.1、O 9「子域与剩余 R 的域必须不重叠；若不能证明，保持整个责任未接管；未覆盖的责任仍由 R 承担」、S 5 |
| B2 | 资格及其上游没有进入失效闭包 | 依赖闭包的根改为资格、carrier、依赖，以及 θ ValidityContract 的依赖假设。R 和 θ 两条路径都对整个闭包做失效屏障、未知边和环检查：θ 出问题时隔离，R 出问题时排除。格式不对的引用直接拒绝，不再被 `filter_map` 静默丢掉。 | S 12「依赖 DAG 包含机制生成、资格、解释资源和行动/数据来源，循环、未知边和断引用均阻止授权」、I12 |
| B3 | 实际使用的有效期合同没有绑定 module 的 `validity_contract` | `ThetaValidity` 新增 CanonicalV1 编码，类型标记为 `echo.theta-validity-contract`，并新增作用 scope 摘要和依赖假设两个字段。编译时核对 `ThetaModule.validity_contract` 的类型、资源版本、长度和摘要，必须与实际使用的合同一致；合同覆盖的 scope 也必须就是 module 的 scope。旁路替换期限或条件都会被拒绝。 | O 10.2（ValidityContract 声明作用域、依赖假设、有限 valid_until 或可验证条件）、I22 |
| B4 | 编译时不重新校验 scope | 所有编译和准入路径统一经过 `check_scope`：端点一致、解释器版本一致，并重新执行 `Scope::new` 的白名单、深度、节点和值域检查，最后核对 scope 引用的绑定。 | O 3.3、S 12 |
| B5 | 已有作用声明的冲突在编译时丢失 | `CompiledEffect` 保留冲突声明。准入时任一方声明冲突都算数；已提交集合中两项声明冲突的作用若 scope 可能相交，返回 `DeclaredConflict` 结构故障。 | O 3.3、O 12.1「冲突责任必须由资格过程选择唯一作用，或编译为单一 composite 后重新验收」 |
| B6 | 整数相交判定与 binary64 求值不一致 | 数值字段按 catalog 的定义是 10^-d 单位的定点数。上下文的 binary64 值先精确舍入到该单位（四舍六入、逢五取偶，用整数运算精确完成），再与区间做整数比较，和相交证明用的是同一套整数语义。证明不相交的两个区间，不可能被同一个值同时命中。 | S 6「浮点误差不得使边界判据时过时不过」 |
| B7 | 预算累加的浮点舍入可能向下 | 新增 `effects/bound.rs`。加法用 TwoSum、乘法用 FMA 残差（纯 Rust `libm::fma`）恢复精确误差，只有在精确值高于浮点结果时才上调一个 ulp。编译预算、准入预算和参数 L1 幅度界都改用这个上界。精确运算不放大，正好压在预算线上的合法组合仍可通过。 | S 6「预算计算使用保守上界」、O 12.1 |

### 回归测试
- `effects/tests/review.rs`（B1–B3）、`review_scope.rs`（B4–B5）、`review_numeric.rs`（B6–B7），共 26 项。各组第一项是审查给出的反例，测试名带 `review_`。
- B6 的性质测试：精度 0–12 位，大整数（接近 ±2^53）、小整数和零附近的区间，在边界及其相邻浮点值上逐一验证「证明不相交 ⇒ 不存在同时命中的值」。
- B7 用独立的大整数精确求和作对照，2000 组随机数据验证上界从不低于精确总和。
- 变异核对：把修复临时换回旧写法后，对应测试会失败——普通加法有 3 项失败，浮点除法判定 2 项，全局抑制 4 项。恢复后全部通过。

### 验收（本次修复）
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过 | `evidence/review-fixes/fmt.log` |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通过 | `evidence/review-fixes/clippy.log` |
| `cargo test --workspace --locked` | 通过（core-kernel 43 项，其中新增 26 项） | `evidence/review-fixes/test.log` |
| 审查的 9 个反例 | 修复前 9 项全部失败，修复后全部通过 | 同上，`review_` 前缀 |

本地工具链为 Rust 1.97.0（`evidence/review-fixes/toolchain.log`），锁定版本 1.99.0 以 CI 为准。

### 对上文的更正
- 「I08」那一行：原测试名写的是 evidence revision，实际改的是 causal_revision。现已改名为 `i08_new_causal_revision_does_not_release_a_taken_over_responsibility`。R 记录目前没有单独的 evidence revision 字段，真正只改证据的路径留作后续。
- 「I07、I21 检查 通过」：原有 I07 测试只覆盖完整接管，不能证明部分接管和数值边界正确。现在以本节 B1、B6 的测试为准。
- 「另覆盖 I12、I22」：原测试只覆盖直接依赖和旁置期限。现在以本节 B2、B3 的测试为准。
- 「DSL 相交测试 ≥ 50 例」：数量统计没有变，反向检查是对称性核验，不算独立的语义场景。另有 B6 的性质测试补足语义一致性。
- `effects/tests.rs` 现为 461 行。I22 测试改为每次修改后重新绑定合同并断言具体原因，因此比原来多了一些行数；保留原因同上文。

### 待用户确认（新增，均不阻塞）
- [ ] （不阻塞）θ ValidityContract 对象的编码和类型标记 `echo.theta-validity-contract` 和 scope、参数对象一样，定义在 Trusted Core 内部，不是 IDL 消息。将来需要跨进程时，升为 IDL 并写 ADR。
- [ ] （不阻塞）数值字段的上下文值先精确舍入到字段单位（逢五取偶），再与区间比较。这与 catalog「定点整数单位」的定义一致；它和原先的浮点除法比较只在边界一个最小单位以内有差别，属于实现选择。
- [ ] （不阻塞）判断 R 条目是否落在接管子域内，采用「单项包含」证明：可靠但不完全。如果某个 R 条目只被几个接管项的并集覆盖，会按「不能证明」处理，返回结构故障，偏保守。
- [ ] （不阻塞）IDL 中 `EffectContract.ancestors` 是 revision 字符串，不是带摘要的对象引用，进不了按摘要索引的依赖图。机制失效经由资格与依赖引用的上游传导到作用。如果以后要把祖先本身纳入闭包，需要在 IDL 中把它改为对象引用（契约变更，需要 ADR）。
- [ ] （不阻塞）θ 记录没有冲突声明字段。R 对 θ 责任声明的冲突仍会被双向检查覆盖。
- [ ] （不阻塞）禁用的 θ 模块如果 scope 已不能在当前解释器下验证（例如 catalog 升级后），编译返回结构故障，不会默认解除接管。catalog 升级本来就需要显式迁移（S 6）。

### 下一步
等 Codex 复审本修复 PR。复审通过并合并后，去掉 PR #22 的 `unverified` 标签，然后接 P5-10。
