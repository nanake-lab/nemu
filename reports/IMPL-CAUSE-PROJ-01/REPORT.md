风险档位：中档（改动 `core/crates/task-layer/` 新增确定性纯函数模块与报告，不涉及权威存储、契约/IDL、行动闸门或 GPU 测量）

# IMPL-CAUSE-PROJ-01 来由投影组包器 — 报告

## 当前状态
已完成全部代码、单元测试与本地静态检查（50 个测试全过，零 clippy 警告，格式与 `Cargo.lock` 校验通过），等待 GitHub CI 与交叉审查后由用户合并。

## 交付物
- [lib.rs](../../core/crates/task-layer/src/lib.rs)：仅追加 `pub mod cause_projection;`。
- [mod.rs](../../core/crates/task-layer/src/cause_projection/mod.rs)：公共入口 `assemble()`、`RequestCodec` trait、类型重导出与 ADR 0042 章节对照文档。
- [types.rs](../../core/crates/task-layer/src/cause_projection/types.rs)：第 4 节全部输入/输出/错误类型与只读 `RecordTable`。
- [validate.rs](../../core/crates/task-layer/src/cause_projection/validate.rs)：第 5.1 步边界校验与第 5.2 步全局可得性检查（按 Q2 读法 A 与 Q6 读法 B）。
- [rank.rs](../../core/crates/task-layer/src/cause_projection/rank.rs)：第 5.3 步（ADR §1.2、U1）按 `max_i |h_i|` 排序并丢弃零槽。
- [closure.rs](../../core/crates/task-layer/src/cause_projection/closure.rs)：第 5.4 步（ADR §1.3）确定性 `BTreeMap`/`BTreeSet` 关系索引与闭包计算（按 Q3A、Q5A、Q6B、Q9A）。
- [assemble.rs](../../core/crates/task-layer/src/cause_projection/assemble.rs)：第 5.5–5.9 步（ADR §§2.1–2.4、3.2、3.3、5.1、5.3）四段组包与整包预算装填（按 Q1A、Q4A、Q7A、Q8A）。
- 测试模块：[tests/mod.rs](../../core/crates/task-layer/src/cause_projection/tests/mod.rs)（`TestCodec` 与无魔数夹具）、[rank.rs](../../core/crates/task-layer/src/cause_projection/tests/rank.rs)、[closure.rs](../../core/crates/task-layer/src/cause_projection/tests/closure.rs)、[golden_2_3.rs](../../core/crates/task-layer/src/cause_projection/tests/golden_2_3.rs)、[accept_3_2.rs](../../core/crates/task-layer/src/cause_projection/tests/accept_3_2.rs)、[step0_decisions.rs](../../core/crates/task-layer/src/cause_projection/tests/step0_decisions.rs)、[determinism.rs](../../core/crates/task-layer/src/cause_projection/tests/determinism.rs)。
- [IMPL-CAUSE-PROJ-01.md](../../tasks/requests/IMPL-CAUSE-PROJ-01.md)：任务卡原文（第 0 步提交）。
- [OPEN.md](./OPEN.md)：第 0 步 Q1–Q9 边界分析与 PR #221 用户定夺原文追加。

**布局与类型微调说明（任务卡 §8）：**
- 测试布局在建议列表之外增加 `tests/step0_decisions.rs`，把第 0 步 Q1–Q9 定夺用例单独归档，确保每个源码与测试文件都不超过 400 行。
- `InvalidInput` 具象为 6 个变体（`NonFiniteH { basis, index }`、`ConflictingDuplicateRecord(RecordId)`、`DuplicateSlotBasis(BasisId)`、`EmptySlotMembers(BasisId)`、`MissingBaseRecord(RecordId)`、`InvalidPartialRange { id, start, end, body_len }`），不携带 `f64`，保证即使输入含 `NaN` 也满足 `Eq`。

## 验收
| 命令或检查 | 结果 | 证据与说明 |
| --- | --- | --- |
| `cargo fmt -p task-layer -- --check` | 通过（exit 0） | 见下方原文末尾 |
| `cargo fmt --all -- --check` | 通过（exit 0） | Rust 工作区全部格式一致 |
| `cargo clippy -p task-layer --all-targets --locked -- -D warnings` | 通过（exit 0） | 0 warnings |
| `cargo test -p task-layer --locked` | 通过（exit 0） | 50 passed; 0 failed; 0 ignored |
| `git diff --exit-code core/Cargo.lock` | 通过（exit 0） | `Cargo.lock` 与 `Cargo.toml` 无改动 |
| 约束扫描（禁用 `HashMap`/`HashSet`、非测试代码禁 `unwrap`/`expect`/`panic!`/`unsafe`） | 通过 | 两项匹配数均为 0；所有文件 ≤ 400 行 |
| `main` 增量影响核对 | 无影响 | 基线 `B = 55135c76` 至最新 `M = 9a35cad2` 仅 `docs/journal/README.md` 改动（#211），不涉及源码、构建或规格 |

第 7 节三条命令实际输出末尾原样：

```text
>>> cargo fmt -p task-layer -- --check
exit=0
```

```text
>>> cargo clippy -p task-layer --all-targets --locked -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
exit=0
```

```text
test cause_projection::tests::step0_decisions::q8_excluded_pack_after_first_overflow_keeps_excluded_fate ... ok
test cause_projection::tests::step0_decisions::q9_multiple_unavailable_relations_resolved_in_canonical_order ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests task_layer

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

## 规格对照

### ADR 0042 §2.3 五例、§3.2 范围内各行与基础单元测试
| ADR / 卡条文 | 测试函数名 | 结果 |
| --- | --- | --- |
| §2.3 例 1 正文引用与接续分开 | `golden_2_3::ex1_body_source_and_continuation_separated` | 通过 |
| §2.3 例 1 变体 a（循环正文引用） | `golden_2_3::ex1_variant_a_body_source_cycle` | 通过 |
| §2.3 例 1 变体 b（接续目标不在记录表） | `golden_2_3::ex1_variant_b_continuation_target_absent_from_records` | 通过 |
| §2.3 例 2 目标更新 | `golden_2_3::ex2_target_updates_retained_without_local_only_revisions` | 通过 |
| §2.3 例 3 别名合并 | `golden_2_3::ex3_alias_merge_deduplicates_shared_quote_without_merge_record` | 通过 |
| §2.3 例 4 更正超预算与变体 | `golden_2_3::ex4_correction_over_budget_fails_base_request`、`golden_2_3::ex4_variant_optional_pack_overflow_still_sends_base` | 通过 |
| §2.3 例 5 长对话与变体 a/b | `golden_2_3::ex5_long_dialogue_continuation_chain_does_not_backfill_e`、`golden_2_3::ex5_variant_a_body_source_enters_e`、`golden_2_3::ex5_variant_b_body_source_over_budget_fails_base` | 通过 |
| §3.2 两包共享原文与更正 | `accept_3_2::two_packs_share_source_and_correction` | 通过 |
| §3.2 包的一部分已在对话 | `accept_3_2::pack_partly_in_dialogue_keeps_s_in_d_and_puts_a_c_in_e` | 通过 |
| §3.2 最大包放不下、后包不再试放 | `accept_3_2::largest_pack_overflows_stops_without_trying_shorter_pack` | 通过 |
| §3.2 基础材料耗尽预算（含零新增与差 1 失败） | `accept_3_2::base_fills_exact_capacity_overflows_pack_with_new_record`、`accept_3_2::base_fills_exact_capacity_minus_one_fails_base`、`accept_3_2::base_fills_exact_capacity_accepts_zero_new_record_pack` | 通过 |
| §3.2 旧迁移分解未知 / 前沿未确认 / 合并未决 | `accept_3_2::unavailable_when_frontier_incomplete`、`accept_3_2::unavailable_when_decomposition_unknown`、`accept_3_2::unavailable_when_candidate_merge_undecided` | 通过 |
| §3.2 包关系未知 / 原文丢失 / 资格未决 | `accept_3_2::unavailable_when_pack_hits_unknown_relation`、`accept_3_2::unavailable_when_body_source_target_missing`、`accept_3_2::unavailable_when_pack_record_eligibility_undetermined` | 通过 |
| §3.2 已知不合格（包排除 / 基础失败 / 全排除） | `accept_3_2::ineligible_in_pack_excludes_pack_and_tries_next`、`accept_3_2::ineligible_in_base_fails_base_request`、`accept_3_2::all_candidates_excluded_returns_no_eligible_experience` | 通过 |
| §2.1 / §5.1 D 含 o 去重、永不严格、校验边界 | `accept_3_2::dialogue_containing_latest_deduplicated_into_o`、`accept_3_2::all_sent_plans_have_strict_false`、`accept_3_2::validation_rejects_bad_partial_range_and_duplicate_slot_basis` | 通过 |
| §1.2 排序（卡 §6.1） | `rank::rank_abs_max_beats_coordinate_sum`、`rank::rank_ties_break_by_first_f_pos_then_basis_byte_order`、`rank::rank_drops_zero_negative_zero_and_empty_h`、`rank::rank_rejects_nan_and_infinite_h` | 通过 |
| §1.3 闭包边界（卡 §6.2） | `closure::closure_body_source_cycle_terminates_once_each`、`closure::closure_chained_correction_of_correction`、`closure::closure_outgoing_correction_does_not_include_target`、`closure::closure_local_only_not_expanded_either_direction`、`closure::closure_distinct_ids_same_body_both_retained`、`closure::closure_duplicate_id_conflicting_body_rejected_and_identical_deduped` | 通过 |
| §3.2 同一绑定重复组装、顺序无关与不可变（卡 §6.5） | `determinism::assemble_twice_yields_identical_outcome_and_bytes`、`determinism::reversing_all_input_collections_preserves_bytes_and_binding`、`determinism::assemble_does_not_mutate_input` | 通过 |

### 第 0 步结果（详见 [OPEN.md](./OPEN.md)）
| 条目 | 用户定夺 | 测试函数名 | 结果 |
| --- | --- | --- | --- |
| Q1：`D` 中 `Partial` 条目是否自动进 `E` | 读法 A（仅「完整所需原文」进 `E`） | `step0_decisions::q1_partial_d_item_not_in_e_unless_full_independently_needed` | 通过 |
| Q2：无外部来源记录或空成员槽处理 | 读法 A（空 `members` 判 `InvalidInput`；任一成员空 `sources` 整包 `Excluded(None)`） | `step0_decisions::q2_empty_member_sources_excludes_pack_and_empty_slot_members_invalid` | 通过 |
| Q3：`RelationKind::Unknown` 触发范围 | 读法 A（闭包已访问节点的双向关联） | `step0_decisions::q3_unknown_relation_triggers_on_incoming_or_outgoing_visited_only` | 通过 |
| Q4：溢出后或含 `Ineligible` 包遇 `Unavailable` | 读法 A（全部非零候选先求闭包并优先判 `Unavailable`） | `step0_decisions::q4_upfront_candidate_unavailable_blocks_after_overflow_and_with_ineligible` | 通过 |
| Q5：`Continuation.to` 可见且禁止/未决 | 读法 A（不检查目标资格，保留直接引用） | `step0_decisions::q5_continuation_target_ineligible_or_undetermined_kept_in_refs` | 通过 |
| Q6：槽种子缺失与全局不可得优先级 | 读法 B（槽种子缺失判 `MissingRecord`；全局不可得优先于基础记录存在性检查） | `step0_decisions::q6_missing_slot_seed_returns_missing_record_and_global_unavailable_precedes_missing_base` | 通过 |
| Q7：基础失败与候选 `Unavailable` 优先级 | 读法 A（全部 `Unavailable` 优先于 `BaseRequestFailed`） | `step0_decisions::q7_candidate_unavailable_precedes_base_ineligible_and_base_over_capacity` | 通过 |
| Q8：`FirstOverflow` 之后 `Ineligible` 包的 fate | 读法 A（保留 `Excluded`） | `step0_decisions::q8_excluded_pack_after_first_overflow_keeps_excluded_fate` | 通过 |
| Q9：多条异常关系的确定性返回次序 | 读法 A（种子缺失 → 弹出时 `Unknown` 优先于已知依赖缺失，按规范字节序 → 闭包后按 `(h_pos, id)` 判 `Undetermined`） | `step0_decisions::q9_multiple_unavailable_relations_resolved_in_canonical_order` | 通过 |

### 本卡不覆盖、留给接线卡的 §3.2 各行
- 两次组装间的撤回/权限变化
- API 拒绝或自动裁切
- 请求取消与部分投递的回执
- U6 的等待、失败、积压与后补解释
- 上下文位置的语义实测

## 新增依赖
无（仅使用 Rust `std`）。

## 待用户确认
- 无新增阻塞项（第 0 步 Q1–Q9 已在 PR #221 定夺并通过测试验证；ADR 0042 本身为 PROPOSED r4，本卡按要求不修改 ADR，PR 保留 `needs-user` 标签等待审查与用户合并）。

## 后续建议（不在本任务范围内）
- 由后续接线卡接入真实 H/F 槽投影、关系类型映射资源、真实 tokenizer/计量器及上述未覆盖的 §3.2 运行时测试。
