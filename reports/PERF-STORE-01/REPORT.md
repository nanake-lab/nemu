重档。本卡涉及 core-store 对象核验和双平台正式计时。

# PERF-STORE-01 对象核验与计时

共享只读 schema 后，core-store 串行整包测试的进程中位数在 Ubuntu 从 1127.15 秒降至 170.67 秒，在 Windows 从 1623.98 秒降至 328.86 秒。两个平台各在同一作业依次测量基线和候选，每个修订跑三轮。四组最慢的库测试都是 `x03_receipt_publish_and_repair_can_crash_again`。随后一次成功运行补齐了 Ubuntu 最慢五项的用户态 CPU 剖析；Ready 后 Windows W02 用时 343.1324 秒。128 笔 release 同机配对中，新 head 由 133.579 ms 降至 52.334 ms，强制全验由 120.356 ms/笔降至 41.666 ms/笔。候选对象示例的 22 条计时列于下文。[配对运行](https://github.com/nanake-lab/nemu/actions/runs/45543148644) · [审查要求](https://github.com/nanake-lab/nemu/pull/265#issuecomment-7157983165)

## 同机配对结果

固定基线为 `6734adc2bfe8b145e8062a2fc669dd2024dd2ba6`，隔离计时候选为 `d7ff3fb6dafa4c53f4ee4d774ab537728834cd71`。两次运行使用工作流修订 `d04af52e1faef70cf6eedded9672af3ed2c2b1cf`、稳定版 Rust/Cargo 1.99.0 和预编译的 test profile。配对运行中，同平台两版的 runner、镜像、工具链和测试清单一致。Ubuntu 镜像版本为 `20261116.202.1`，每轮整包通过 71 个库测试和 3 个文档测试。Windows 镜像版本为 `20261116.315.1`，每轮通过 69 个库测试和 3 个文档测试。[工作流](../../.github/workflows/perf-store.yml)

框架秒数取自 libtest 和 rustdoc 汇总行，整包时两者相加。进程秒数从启动 Cargo 之前计到进程退出。下表两列中位数分别由三轮独立计算，节省量为基线减候选，降幅以基线为分母。

| 平台 | 整包计时 | 基线（秒） | 候选（秒） | 节省（秒） | 降幅 |
| --- | --- | ---: | ---: | ---: | ---: |
| Ubuntu | 进程 | 1127.151409 | 170.673135 | 956.478274 | 84.86% |
| Ubuntu | 框架 | 1127.05 | 170.56 | 956.49 | 84.87% |
| Windows | 进程 | 1623.9818874 | 328.8602699 | 1295.1216175 | 79.75% |
| Windows | 框架 | 1623.70 | 328.59 | 1295.11 | 79.76% |

### 各修订最慢的 15 个库测试

每项均按名称单独运行三轮，按框架中位数降序排列，同时列出同一测试的进程中位数。整包和每轮整组运行的三个文档测试均单独记样。

#### Ubuntu · 基线

| 排名 | 库测试 | 框架中位数（秒） | 进程中位数（秒） |
| ---: | --- | ---: | ---: |
| 1 | `tests::crash::x03_receipt_publish_and_repair_can_crash_again` | 475.56 | 475.614 |
| 2 | `tests::crash::x03_targeted_r_segments_manifest_and_reused_objects` | 146.22 | 146.268 |
| 3 | `tests::crash::x03_ingress_and_transaction_publication_boundaries` | 131.10 | 131.155 |
| 4 | `tests::crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 39.30 | 39.352 |
| 5 | `tests::format_three::sys17_v2_v4_event_history_and_rollback_preserve_bytes` | 20.99 | 21.033 |
| 6 | `tests::theta::actual_states_commit_close_reopen` | 20.98 | 21.027 |
| 7 | `tests::terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 19.64 | 19.685 |
| 8 | `tests::consistency::all_derived_tables_match_authoritative_rebuild` | 18.35 | 18.402 |
| 9 | `tests::consistency::existing_initialization_namespaces_complete_every_sync_before_admission` | 17.70 | 17.753 |
| 10 | `tests::reader::reader_receipts_precede_sql_snapshot_under_controlled_appends` | 15.01 | 15.055 |
| 11 | `tests::atomic::recovery_rejects_valid_but_mixed_head_and_missing_ledger` | 13.17 | 13.222 |
| 12 | `tests::reader::incremental_verification_hashes_new_references_and_full_checks_cold_history` | 12.69 | 12.742 |
| 13 | `tests::theta::invalid_new_and_legacy_staging_never_publish` | 11.20 | 11.248 |
| 14 | `tests::reader::complete_transaction_prefix_is_verified_and_bounded` | 10.08 | 10.132 |
| 15 | `tests::reader::receipts_extend_the_verified_prefix_and_recheck_after_reopen` | 9.77 | 9.821 |

#### Ubuntu · 候选

| 排名 | 库测试 | 框架中位数（秒） | 进程中位数（秒） |
| ---: | --- | ---: | ---: |
| 1 | `tests::crash::x03_receipt_publish_and_repair_can_crash_again` | 73.83 | 73.879 |
| 2 | `tests::crash::x03_targeted_r_segments_manifest_and_reused_objects` | 22.29 | 22.339 |
| 3 | `tests::crash::x03_ingress_and_transaction_publication_boundaries` | 20.63 | 20.679 |
| 4 | `tests::crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 6.61 | 6.655 |
| 5 | `tests::terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 3.75 | 3.794 |
| 6 | `tests::format_three::sys17_v2_v4_event_history_and_rollback_preserve_bytes` | 3.32 | 3.372 |
| 7 | `tests::theta::actual_states_commit_close_reopen` | 2.96 | 3.005 |
| 8 | `tests::consistency::existing_initialization_namespaces_complete_every_sync_before_admission` | 2.75 | 2.797 |
| 9 | `tests::consistency::all_derived_tables_match_authoritative_rebuild` | 2.61 | 2.662 |
| 10 | `tests::reader::reader_receipts_precede_sql_snapshot_under_controlled_appends` | 2.10 | 2.153 |
| 11 | `tests::atomic::recovery_rejects_valid_but_mixed_head_and_missing_ledger` | 1.84 | 1.890 |
| 12 | `tests::reader::incremental_verification_hashes_new_references_and_full_checks_cold_history` | 1.82 | 1.870 |
| 13 | `tests::theta::caller_limits_reach_prepare_commit_and_read` | 1.59 | 1.634 |
| 14 | `tests::reader::complete_transaction_prefix_is_verified_and_bounded` | 1.52 | 1.571 |
| 15 | `tests::reader::receipts_extend_the_verified_prefix_and_recheck_after_reopen` | 1.47 | 1.514 |

#### Windows · 基线

| 排名 | 库测试 | 框架中位数（秒） | 进程中位数（秒） |
| ---: | --- | ---: | ---: |
| 1 | `tests::crash::x03_receipt_publish_and_repair_can_crash_again` | 678.10 | 678.290 |
| 2 | `tests::crash::x03_targeted_r_segments_manifest_and_reused_objects` | 208.64 | 208.840 |
| 3 | `tests::crash::x03_ingress_and_transaction_publication_boundaries` | 186.11 | 186.254 |
| 4 | `tests::crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 57.01 | 57.158 |
| 5 | `tests::theta::actual_states_commit_close_reopen` | 30.23 | 30.379 |
| 6 | `tests::format_three::sys17_v2_v4_event_history_and_rollback_preserve_bytes` | 29.72 | 29.859 |
| 7 | `tests::terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 29.37 | 29.515 |
| 8 | `tests::consistency::existing_initialization_namespaces_complete_every_sync_before_admission` | 26.10 | 26.252 |
| 9 | `tests::consistency::all_derived_tables_match_authoritative_rebuild` | 26.04 | 26.186 |
| 10 | `tests::reader::reader_receipts_precede_sql_snapshot_under_controlled_appends` | 21.34 | 21.480 |
| 11 | `tests::atomic::recovery_rejects_valid_but_mixed_head_and_missing_ledger` | 19.24 | 19.388 |
| 12 | `tests::reader::incremental_verification_hashes_new_references_and_full_checks_cold_history` | 18.10 | 18.241 |
| 13 | `tests::theta::invalid_new_and_legacy_staging_never_publish` | 15.93 | 16.077 |
| 14 | `tests::reader::complete_transaction_prefix_is_verified_and_bounded` | 14.36 | 14.514 |
| 15 | `tests::reader::receipts_extend_the_verified_prefix_and_recheck_after_reopen` | 13.87 | 14.017 |

#### Windows · 候选

| 排名 | 库测试 | 框架中位数（秒） | 进程中位数（秒） |
| ---: | --- | ---: | ---: |
| 1 | `tests::crash::x03_receipt_publish_and_repair_can_crash_again` | 134.39 | 134.541 |
| 2 | `tests::crash::x03_targeted_r_segments_manifest_and_reused_objects` | 41.93 | 42.076 |
| 3 | `tests::crash::x03_ingress_and_transaction_publication_boundaries` | 37.84 | 37.980 |
| 4 | `tests::crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 12.63 | 12.772 |
| 5 | `tests::terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 7.73 | 7.877 |
| 6 | `tests::format_three::sys17_v2_v4_event_history_and_rollback_preserve_bytes` | 6.05 | 6.195 |
| 7 | `tests::theta::actual_states_commit_close_reopen` | 6.03 | 6.178 |
| 8 | `tests::consistency::existing_initialization_namespaces_complete_every_sync_before_admission` | 5.77 | 5.910 |
| 9 | `tests::consistency::all_derived_tables_match_authoritative_rebuild` | 4.78 | 4.927 |
| 10 | `tests::reader::reader_receipts_precede_sql_snapshot_under_controlled_appends` | 4.22 | 4.370 |
| 11 | `tests::atomic::recovery_rejects_valid_but_mixed_head_and_missing_ledger` | 3.89 | 4.038 |
| 12 | `tests::reader::incremental_verification_hashes_new_references_and_full_checks_cold_history` | 3.68 | 3.822 |
| 13 | `tests::reader::complete_transaction_prefix_is_verified_and_bounded` | 2.88 | 3.034 |
| 14 | `tests::reader::receipts_extend_the_verified_prefix_and_recheck_after_reopen` | 2.77 | 2.944 |
| 15 | `tests::theta::invalid_new_and_legacy_staging_never_publish` | 2.69 | 2.837 |

## 单独基线诊断

另一作业单独运行相同的固定基线。Ubuntu 整包框架中位数为 **1382.91 秒**，进程为 **1383.032775 秒**。Windows 分别为 **1409.24 秒**和 **1409.4927125 秒**。两台机器的结果仅用于观察基线在不同运行间的波动。上面的降幅均取自配对作业。[单独基线运行](https://github.com/nanake-lab/nemu/actions/runs/45543135745)

<details>
<summary>单独基线的库测试前 15 名</summary>

#### Ubuntu · 基线

| 排名 | 库测试 | 框架中位数（秒） | 进程中位数（秒） |
| ---: | --- | ---: | ---: |
| 1 | `tests::crash::x03_receipt_publish_and_repair_can_crash_again` | 581.61 | 581.666 |
| 2 | `tests::crash::x03_targeted_r_segments_manifest_and_reused_objects` | 178.87 | 178.929 |
| 3 | `tests::crash::x03_ingress_and_transaction_publication_boundaries` | 159.89 | 159.948 |
| 4 | `tests::crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 48.38 | 48.438 |
| 5 | `tests::format_three::sys17_v2_v4_event_history_and_rollback_preserve_bytes` | 25.94 | 25.992 |
| 6 | `tests::theta::actual_states_commit_close_reopen` | 25.74 | 25.796 |
| 7 | `tests::terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 23.98 | 24.038 |
| 8 | `tests::consistency::all_derived_tables_match_authoritative_rebuild` | 22.66 | 22.714 |
| 9 | `tests::consistency::existing_initialization_namespaces_complete_every_sync_before_admission` | 21.78 | 21.835 |
| 10 | `tests::reader::reader_receipts_precede_sql_snapshot_under_controlled_appends` | 18.29 | 18.343 |
| 11 | `tests::atomic::recovery_rejects_valid_but_mixed_head_and_missing_ledger` | 16.15 | 16.206 |
| 12 | `tests::reader::incremental_verification_hashes_new_references_and_full_checks_cold_history` | 15.44 | 15.497 |
| 13 | `tests::theta::invalid_new_and_legacy_staging_never_publish` | 13.76 | 13.814 |
| 14 | `tests::reader::complete_transaction_prefix_is_verified_and_bounded` | 12.35 | 12.407 |
| 15 | `tests::reader::receipts_extend_the_verified_prefix_and_recheck_after_reopen` | 11.94 | 12.000 |

#### Windows · 基线

| 排名 | 库测试 | 框架中位数（秒） | 进程中位数（秒） |
| ---: | --- | ---: | ---: |
| 1 | `tests::crash::x03_receipt_publish_and_repair_can_crash_again` | 575.40 | 575.558 |
| 2 | `tests::crash::x03_targeted_r_segments_manifest_and_reused_objects` | 187.60 | 187.785 |
| 3 | `tests::crash::x03_ingress_and_transaction_publication_boundaries` | 158.06 | 158.187 |
| 4 | `tests::crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 48.39 | 48.507 |
| 5 | `tests::format_three::sys17_v2_v4_event_history_and_rollback_preserve_bytes` | 26.58 | 26.708 |
| 6 | `tests::theta::actual_states_commit_close_reopen` | 26.44 | 26.569 |
| 7 | `tests::terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 25.21 | 25.338 |
| 8 | `tests::consistency::existing_initialization_namespaces_complete_every_sync_before_admission` | 22.96 | 23.085 |
| 9 | `tests::consistency::all_derived_tables_match_authoritative_rebuild` | 22.56 | 22.687 |
| 10 | `tests::reader::reader_receipts_precede_sql_snapshot_under_controlled_appends` | 18.42 | 18.550 |
| 11 | `tests::atomic::recovery_rejects_valid_but_mixed_head_and_missing_ledger` | 16.39 | 16.509 |
| 12 | `tests::reader::incremental_verification_hashes_new_references_and_full_checks_cold_history` | 16.19 | 16.368 |
| 13 | `tests::theta::invalid_new_and_legacy_staging_never_publish` | 13.90 | 14.025 |
| 14 | `tests::reader::complete_transaction_prefix_is_verified_and_bounded` | 12.93 | 13.055 |
| 15 | `tests::reader::receipts_extend_the_verified_prefix_and_recheck_after_reopen` | 12.25 | 12.367 |

</details>

## 改动与行为核对

- `Schema::shared_builtin()` 用 `OnceLock` 复用编入二进制的描述符解析结果，保留原错误文本。需要独立可变描述符的调用继续使用 `Schema::builtin()`。对象引用构造、NSI1 索引识别、对象核验和引用闭包遍历仍逐次从实际字节检查长度、摘要、类型和语义。[schema 实现](../../core/crates/core-types/src/contract.rs) · [对象核验](../../core/crates/core-store/src/objects.rs) · [I16、I18、I21 存储映射](../../docs/traceability/P5-03.md)
- 隔离候选还在存储提交、发布、查询和 core-types 语义检查等路径复用同一描述符，只改变取得只读 schema 的方式。集成分支另有 `semantic/initial_publication.rs` 两处替换。基线和隔离候选的库测试目录 git tree 均为 `1424d339e714106bc0c693bd2bd7c164b0df0edf`，契约及依赖锁一致。[候选差异](https://github.com/nanake-lab/nemu/compare/6734adc2bfe8b145e8062a2fc669dd2024dd2ba6...d7ff3fb6dafa4c53f4ee4d774ab537728834cd71) · [集成修订](https://github.com/nanake-lab/nemu/commit/93e09f2cdec389e877c89050d66171e5f136fc2a)
- 对象探针通过临时合成谱系的 `Store::prepare` 和 `prepare_r` 记录叶对象、带引用索引的首次写入与复用，检查复用前后引用相等。复测入口为 `cargo run --release --locked -p core-store --example object_hotspots`，两版应使用同一份探针源文件。[探针](../../core/crates/core-store/examples/object_hotspots.rs)

## 验证与未完成项

- 原始计时复核覆盖 1,296 条样本。每个修订每轮包括整包、所有逐项库测试和一组文档测试，Ubuntu 每修订共 219 条，Windows 共 213 条。逐份核对了实际 SHA、测试名、通过计数、框架汇总和重编译记录。从 `samples.jsonl` 独立重算的两种中位数均与 `medians.json` 一致。
- 此前两次 Ubuntu 运行均在上传计时后，因 `perf_bin` 为空而以 127 退出，最慢五项的剖析均未执行。工作流现先查 PATH，再查 `/usr/lib/linux-tools` 下可执行的普通文件和符号链接，调用前用 `test -x` 校验。`actionlint` 1.7.12、YAML 完整解析、`bash -n` 和离线查找用例均通过，覆盖 PATH 优先、普通文件、符号链接、目录不存在和空目录。后续成功运行已生成两个修订的函数占比及火焰图，候选结果见下文。[原基线作业](https://github.com/nanake-lab/nemu/actions/runs/45543135745) · [原配对作业](https://github.com/nanake-lab/nemu/actions/runs/45543148644/job/138966266388)
- 1/2 的两次运行未包含 W02、128 笔 release 复测及对象示例。2/2 的实际测量范围见下文。[#240 测量口径](../FIX-SYS01-01/REPORT.md)
- 此前的四次调度也保留了失败记录。第一组因 Rustdoc 的 compile-fail 显示名拒绝整包样本，第二组在精确筛选单个文档测试时空跑。本次正式结果来自随后的整组文档测试计时。[首组基线](https://github.com/nanake-lab/nemu/actions/runs/45485885498) · [首组配对](https://github.com/nanake-lab/nemu/actions/runs/45486419758) · [第二组基线](https://github.com/nanake-lab/nemu/actions/runs/45524016105) · [第二组配对](https://github.com/nanake-lab/nemu/actions/runs/45524027980)
- 工作流修订 `d04af52e` 的 Ubuntu 常规 CI 通过。其独立 Windows 工作流的汇总步骤失败，W02 作业跳过。本报告的计时与测试通过数来自手动托管运行。1/2 已合并。[Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/45543104247) · [Windows 汇总](https://github.com/nanake-lab/nemu/actions/runs/45543104221)

## 2/2 · Ubuntu 候选最慢五项

成功配对运行在 `d7ff3fb6` 上先逐项串行计时三轮，再按框架中位数选前五，每项单独用 perf 运行一次，五次均通过。前三列来自该次运行的 `test.json` 和 `test.log`。函数自占比取 `perf report --no-children` 第一行，保留其线程维度。五项的首位函数均为测试线程上的 `<str as core::cmp::Ord>::cmp`。[托管运行](https://github.com/nanake-lab/nemu/actions/runs/45590384397) · [剖析产物](https://github.com/nanake-lab/nemu/actions/runs/45590384397/artifacts/15020983381)

| 测试（去掉 `tests::` 前缀） | 框架中位数（秒） | perf 样本数 | 字符串比较自占比 | JSON | canonical | SHA | SQLite |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `crash::x03_receipt_publish_and_repair_can_crash_again` | 77.55 | 7,039 | 6.19% | 3.27% | 0.17% | 0.64% | 0.81% |
| `crash::x03_targeted_r_segments_manifest_and_reused_objects` | 27.71 | 2,114 | 5.87% | 3.22% | 0.05% | 0.57% | 0.99% |
| `crash::x03_ingress_and_transaction_publication_boundaries` | 20.51 | 1,929 | 6.95% | 3.11% | 0.05% | 0.47% | 0.73% |
| `crash::initialization_and_gc_crash_boundaries_have_separate_expectations` | 6.64 | 626 | 4.63% | 4.63% | 0.16% | 1.44% | 4.15% |
| `terminal_ingress::process_crashes_recover_allocation_audit_and_both_migration_directions` | 4.07 | 339 | 5.60% | 11.80% | 0.29% | 1.47% | 7.08% |

后四列表示可辨认栈所占的用户态采样权重。逐行读取 `stacks.folded`，对不区分大小写命中 `serde_json::`、`canonical`、`sha2::|sha2_asm::|sha256`、`sqlite3|rusqlite::` 的栈分别累加行末权重，除以该文件全部权重。同一条栈可以命中多列，栈帧不全、内联或缺失符号都会影响归类。采样覆盖 Cargo 启动的进程及其子进程。五项的 fsync、磁盘等待和 SQLite 等待时间均**未测**；`cpu-clock:u` 以 99 Hz 记录用户态 CPU，无法分解墙钟时间。字符串比较的自占比最高为 6.95%，现有证据没有定位到占大头且改动便宜的函数。优化建议到此为止。

## Ready 后的 Windows W02

| 托管作业 | 实测合并提交 | Cargo 缓存 | W02 步骤（秒） | 69 个库测试（秒） | 3 个 doctest（秒） |
| --- | --- | --- | ---: | ---: | ---: |
| [#249 历史作业](https://github.com/nanake-lab/nemu/actions/runs/45208141416/job/137809648690) | `74cfe2a` | 未命中 | 1,726.2140 | 1,650.31 | 0.22 |
| [#265 Ready](https://github.com/nanake-lab/nemu/actions/runs/45674066558/job/139417098493) | `4241654`（PR head `8003df39`） | 命中 | **343.1324** | **286.15** | **0.22** |

步骤按日志中 W02 命令组开始至下一上传步骤开始计时。Ready 作业的 UTC 时间为 18:21:19.2868075 至 18:27:02.4192196，包含工具链检查、Clippy、编译及串行测试。两行合并提交和缓存条件不同，W02 步骤的配对降幅未测。

## #240 release 配对与对象示例

在同一台 GitHub 托管 Windows VM 上按基线 `6734adc2bfe8b145e8062a2fc669dd2024dd2ba6`、候选 `d7ff3fb6dafa4c53f4ee4d774ab537728834cd71` 的顺序各运行一条独立的 128 笔合成历史，各测一次。工具链固定为 1.99.0。已与 #240 成功运行的命令、测量代码和日志核对。托管步骤将同一份合成探针注入两个临时 checkout，两个修订各 12 个文本替换锚点均通过唯一性检查。[运行时的探针](https://github.com/nanake-lab/nemu-evidence/blob/d11084fa5b8da217d71306fbefc71d4cebcc34d6/PERF-STORE-01/45731209735-1/source/reports/PERF-STORE-01/prepare_release_probe.py) · [托管执行](https://github.com/nanake-lab/nemu/actions/runs/45731209735)

- 原命令为 `cargo test -p gateway --release review_scale_probe -- --ignored --nocapture --test-threads=1`。保留在 2、8、32、128 笔的检查，报告取 128 笔。
- 新 head 先在 127 笔处核验前缀，提交第 128 笔后计时 `receipt_reader`。每个修订取唯一一条 `commits=128 operation=new_head` 的 `elapsed_ns / 1e6`（ms/新 head）。
- 强制全验从创建 `full_publication_check()` 前到返回的闭包执行成功为止，取唯一一条 `SYS01_FULL commits=128` 的 `total_ns / 1e6`（ms），再除以 128 得到 ms/笔。Cargo 编译和整条测试时长均在这两个计时区间之外。#240 的 149.324 ms/新 head 与 138.20 ms/笔仅为历史结果。
- 同次调度另开 GitHub 托管 Ubuntu 作业，在候选修订运行一次 `cargo run --release --locked -p core-store --example object_hotspots`，输出叶对象及索引对象各一次首次写入、各十次复用的纳秒数。基线没有该示例，此项仅报告候选。

托管 Windows 镜像 `20261116.315.1`，两个修订各通过 1 个探针测试。两份原始日志均含预定的四个测点，下表仅取 128 笔。每侧只有一条历史，表中差异只描述这次配对。[Windows 配对作业](https://github.com/nanake-lab/nemu/actions/runs/45731209735/job/139613374170)

| 修订 | 新 head（ms/新 head） | 强制全验总耗时（ms） | 强制全验（ms/笔） |
| --- | ---: | ---: | ---: |
| 基线 `6734adc2` | 133.5789 | 15,405.6184 | 120.356394 |
| 候选 `d7ff3fb6` | 52.3337 | 5,333.3043 | 41.666440 |

### 候选 object_hotspots 单次输出

以下为探针内部的纳秒计时，按同一采样序号并排列出。每次复用均通过对象引用相等断言。[托管 Ubuntu 作业](https://github.com/nanake-lab/nemu/actions/runs/45731209735/job/139613373840)

| 操作 | 样本 | 叶对象（ns） | 索引对象（ns） |
| --- | ---: | ---: | ---: |
| 首次写入 | 1 | 1,859,567 | 852,864 |
| 复用 | 1 | 299,147 | 280,178 |
| 复用 | 2 | 150,766 | 309,041 |
| 复用 | 3 | 293,929 | 276,483 |
| 复用 | 4 | 148,302 | 301,270 |
| 复用 | 5 | 291,094 | 269,863 |
| 复用 | 6 | 332,927 | 273,509 |
| 复用 | 7 | 261,410 | 269,192 |
| 复用 | 8 | 147,200 | 271,245 |
| 复用 | 9 | 284,084 | 267,840 |
| 复用 | 10 | 152,248 | 268,771 |

## 原始证据

四份 Actions 原始 ZIP 已按字节归档至私有证据仓库的 `PERF-STORE-01/`，提交 `17e29f75289a8910bddc155b2b6115a2f3276a9e`。下表 SHA-256 与 Actions 公布的归档摘要一致。

| 原始归档 | SHA-256 |
| --- | --- |
| [perf-store-Linux-45543135745-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Linux-45543135745-1.zip) | `bf283030c9684d45cafb96da3e70c7261e98dad660088c4ea7fab7f07cb03226` |
| [perf-store-Linux-45543148644-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Linux-45543148644-1.zip) | `2866e9c7c606fbff85f8bcef7a83a03b04a65bd4b4c7500ba7428256167235f6` |
| [perf-store-Windows-45543135745-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Windows-45543135745-1.zip) | `293d755b1edb99d833225a9470c5ad93063ff1dea3106d77f235dcaf7cd348fa` |
| [perf-store-Windows-45543148644-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Windows-45543148644-1.zip) | `50b3f34b3f89c086a27e7903f4fff74df006e63cbff6993af4d69b0d2cd249e6` |

### 2/2 证据核对

| 来源 | 原始归档或日志的 SHA-256 |
| --- | --- |
| 45590384397 · Ubuntu 计时 ZIP | `37dbc5a4c8d457b8abbba47bd9923af18dc0abfd12bc2095fe46a959dd023408` |
| 45590384397 · Windows 计时 ZIP | `40265af84278dc46c9679e82e6199dc56cca567bf05369137833478ce1a560ab` |
| 45590384397 · perf 剖析 ZIP | `621e8cc929f712c95c77c2de8dade0c89295409ff8f28edc2f45707c6b80f6c0` |
| 45208141416 · 历史 W02 作业日志（gh run view --log） | `e788c4781d3f1e8439cbbf95bca697c5b13f71a4a729bd9e687f2b522d8a833e` |
| 45674066558 · Ready W02 作业日志（gh run view --log） | `4f96963112a799b7cb88e416ee2118746d8533e19690320d69109e4acd65a032` |
| 45731209735 · Ubuntu 对象示例 ZIP | `ac200c0a6038b149c74e14e468e6aa6d10b6ac5ef6147f7e21994b4f4bcfa0d1` |
| 45731209735 · Windows release 配对 ZIP | `561f353104a4f6a890c3620a6004ec557b653bc14b7318e2fc46e42b7185c250` |

已从 45590384397 的逐项三轮样本独立重算前五名，并核对采样测试名、通过记录、采样数与剖析文件。三个原始 ZIP 及对象示例 ZIP 的 SHA-256 均与 Actions 公布值一致；剖析 ZIP 无损分为三片，按序重组的摘要一致。工作流通过 actionlint 1.7.12 和内嵌 Python 语法检查，报告相对链接及 `git diff --check` 通过。

私有证据索引提交 `053411af0b7bdfba79027d8e4a72fd501588fde1`，按上表运行号检索。

[运行 45731209735](https://github.com/nanake-lab/nemu/actions/runs/45731209735) 使用提交 `c37b61573a13c5edac9143a8a133ba2eff031936`。以下两份源文件已从该提交按原始字节归档，并从私有证据提交 `d11084fa5b8da217d71306fbefc71d4cebcc34d6` 回读核对。

| 运行时的源文件 | 字节数 | SHA-256 |
| --- | ---: | --- |
| [perf-store.yml](https://github.com/nanake-lab/nemu-evidence/blob/d11084fa5b8da217d71306fbefc71d4cebcc34d6/PERF-STORE-01/45731209735-1/source/.github/workflows/perf-store.yml) | 17258 | `6c5dbce4781762e5e421ef900a0389088b4987a1d7a63d4129614d16d04c5352` |
| [prepare_release_probe.py](https://github.com/nanake-lab/nemu-evidence/blob/d11084fa5b8da217d71306fbefc71d4cebcc34d6/PERF-STORE-01/45731209735-1/source/reports/PERF-STORE-01/prepare_release_probe.py) | 12907 | `dca4b59d8303f78051bdedf9ca871486e36f291bf659984c4b33a6093e755354` |
