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
