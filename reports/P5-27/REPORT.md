# P5-27 作用视图按 v2.0 简化 — 报告

## 当前状态
完成。作用视图按 Echo Core v2.0 重写为两个纯函数，v1.0 的 q 包络、ValidityContract、接管、资格、scope DSL 与相交检查已删除。接下来取 P5-10（快状态 F）。

## 交付物
- `core/crates/core-kernel/src/effects/`：
  - `compile.rs`：`compile_authorized_effects(stored_R, stored_θ, process_time, validity)` 保留 ADR 0008 §2 的签名（validity 为空的 `Validity`），返回 E_eff：R⁺ 中每个倾向的 s_k = max(0, 2π − 1)、r_k、d_k、概念引用、方向是否已验证，以及 θ；`project_r`（Φ_R，v2.0 中为恒等）；`strength`。
  - `evaluate.rs`：`evaluate_effects(E_eff, μ(e))` 返回每个 k 的 f_k(e) = s_k μ_k(e) r_k d_k 与 D_k(e) = s_k μ_k(e)，以及按规范次序求和的 Σ_k f_k。
  - `types.rs`：窄输入输出类型，不依赖 P5-21 的生成类型；倾向次序与单位方向容差复用 `core_types::semantic`（`tendency_order`、`UNIT_NORM_TOLERANCE`），与 F（P5-10）和 V2-32 保持一致。
  - `tests.rs`：12 项测试。
- `core/crates/core-policy/`：scope DSL、字段目录、端点白名单、规范编码与相交检查全部删除；crate 保留为空壳，使 AGENTS.md 第 2 节的目录归属不变，不再依赖 sha2。
- `core/crates/core-kernel/Cargo.toml`：去掉 core-policy 与 libm 依赖；`core/Cargo.lock` 相应删去 12 行，由 Cargo 重新生成。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| I06：候选（π ≤ ½，含 π = ½）对 x 和 m 没有作用；匹配器给候选的 μ 被忽略 | 通过 | `i06_candidates_have_no_effect_on_x_or_m` |
| I21：只借用输入、不写回；重复求值一致 | 通过 | `i21_inputs_are_only_borrowed_and_repeated_calls_agree` |
| ADR 0008 §2 组合律 Compile(Φ_R(Δ; R)) = Compile(R)，且与处理时刻无关 | 通过 | `adr0008_composition_law_and_no_time_dependence` |
| 数值：固定乘积次序 ((s·μ)·r)·d_i、Σf 按规范次序、无 −0 | 通过 | `evaluate_uses_fixed_product_order`、`f_sum_is_summed_in_canonical_order`、`unlisted_match_is_zero_and_negative_zero_is_normalized`、`strength_matches_v2_formula_and_has_no_negative_zero` |
| 存储值越域即拒绝（含候选），μ 越域或重复即拒绝 | 通过 | `stored_values_outside_their_domain_are_refused`、`theta_outside_its_domain_is_refused`、`situation_match_outside_its_domain_is_refused` |
| `cargo fmt --all -- --check` | 通过 | 本机 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | 本机 |
| `cargo test -p core-kernel -p core-policy` | 12 通过 | 本机 |
| `cargo test --workspace --no-fail-fast` | 全部通过（core-kernel 12、core-store 31、core-types 7 及契约测试 17 等），退出码 0 | `cargo-test-workspace.log` |
| 删除的代码没有残留引用 | 通过：工作区内不再有 `core_policy::` 的使用 | `grep` |

**环境说明：** 云端会话无法下载仓库固定的 Rust 1.99.0（static.rust-lang.org 被代理拒绝），本机检查用已安装的 1.97.0（`RUSTUP_TOOLCHAIN=stable`）。以 CI 上 1.99.0 的结果为准。

## 规格对照
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| ADR 0008 §2：Compile 只做一次时间投影，签名 (stored_R, stored_θ, process_time, validity) | `compile.rs` | `adr0008_*` |
| v2.0 §5.2：s_k = max(0, 2π_k − 1)，π_k > ½ 时进入 R⁺ | `compile.rs::strength` 与过滤 | `strength_*`、`i06_*` |
| v2.0 §6：f_k(e) = s_k μ_k(e) r_k d_k，D_k(e) = s_k μ_k(e) | `evaluate.rs` | `evaluate_uses_fixed_product_order` |
| v2.0 §7、§9.6：方向是否已验证随 E_eff 传给 J，不改变 f、D | `types.rs::TendencyEffect` | `direction_validation_is_passed_through_without_changing_drive` |
| v2.0 §15：q 包络、ValidityContract、接管、资格、scope DSL 与相交检查被取代 | 已删除 | 工作区编译与 clippy |
| LP5：R 不随时间变化，Φ_R 为恒等 | `compile.rs::project_r` | `adr0008_*` |
| V2-32：d_k 为单位向量（容差 1e-12）、π ∈ (−1, 1) | `compile.rs::check_tendency` | `stored_values_outside_their_domain_are_refused` |
| ADR 0019 §8：固定运算次序、无 −0 | `evaluate.rs` | 数值三项 |

## 新增依赖
无。删除了 core-kernel 对 core-policy、libm 的依赖和 core-policy 对 sha2、core-types 的依赖。

## 待用户确认
无。

## 后续建议（不在本任务范围内）
- P5-10（F）直接调用 `evaluate_effects`，按 ADR 0026 §6.3 对自致事件的 f、D 乘 κ^n 由 F 负责，本卡不做。
- AGENTS.md 第 2 节对 `core/crates/core-policy/` 的说明可在下次改 AGENTS 时注明「v2.0 后为空」。
