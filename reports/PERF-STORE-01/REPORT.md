重档。本卡涉及 core-store 对象核验和双平台正式计时。

# PERF-STORE-01 对象核验与计时

共享只读 schema 后，core-store 串行整包测试的进程中位数在 Ubuntu 从 1127.15 秒降至 170.67 秒，在 Windows 从 1623.98 秒降至 328.86 秒。两个平台各在同一作业依次测量基线和候选，每个修订跑三轮。四组最慢的库测试都是 `x03_receipt_publish_and_repair_can_crash_again`。Ubuntu 的计时已完成，随后在启动 perf 前失败，函数占比和火焰图待一尺线跟进。[配对运行](https://github.com/nanake-lab/nemu/actions/runs/45543148644) · [审查要求](https://github.com/nanake-lab/nemu/pull/265#issuecomment-7157983165)

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
- 两次 Ubuntu 运行均在上传计时后，因 `perf_bin` 为空而以 127 退出，最慢五项的剖析均未执行。工作流现先查 PATH，再查 `/usr/lib/linux-tools` 下可执行的普通文件和符号链接，调用前用 `test -x` 校验。`actionlint` 1.7.12、YAML 完整解析、`bash -n` 和离线查找用例均通过，覆盖 PATH 优先、普通文件、符号链接、目录不存在和空目录。两个修订的函数占比及火焰图仍待新配对运行验证。[原基线作业](https://github.com/nanake-lab/nemu/actions/runs/45543135745) · [原配对作业](https://github.com/nanake-lab/nemu/actions/runs/45543148644/job/138966266388)
- Windows W02 独立步骤耗时、#240 的两项 128 笔 release 复测（新 head 核验、完整核验每笔耗时）以及对象探针实际执行，两版均为 `not_run`。#240 曾测得 149.324 ms/新 head 和 138.20 ms/笔，仅作历史背景。[#240 测量口径](../FIX-SYS01-01/REPORT.md)
- 此前的四次调度也保留了失败记录。第一组因 Rustdoc 的 compile-fail 显示名拒绝整包样本，第二组在精确筛选单个文档测试时空跑。本次正式结果来自随后的整组文档测试计时。[首组基线](https://github.com/nanake-lab/nemu/actions/runs/45485885498) · [首组配对](https://github.com/nanake-lab/nemu/actions/runs/45486419758) · [第二组基线](https://github.com/nanake-lab/nemu/actions/runs/45524016105) · [第二组配对](https://github.com/nanake-lab/nemu/actions/runs/45524027980)
- 工作流修订 `d04af52e` 的 Ubuntu 常规 CI 通过。其独立 Windows 工作流的汇总步骤失败，W02 作业跳过。本报告的计时与测试通过数来自手动托管运行。PR 保持草稿。[Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/45543104247) · [Windows 汇总](https://github.com/nanake-lab/nemu/actions/runs/45543104221)

## 原始证据

四份 Actions 原始 ZIP 已按字节归档至私有证据仓库的 `PERF-STORE-01/`，提交 `17e29f75289a8910bddc155b2b6115a2f3276a9e`。下表 SHA-256 与 Actions 公布的归档摘要一致。

| 原始归档 | SHA-256 |
| --- | --- |
| [perf-store-Linux-45543135745-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Linux-45543135745-1.zip) | `bf283030c9684d45cafb96da3e70c7261e98dad660088c4ea7fab7f07cb03226` |
| [perf-store-Linux-45543148644-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Linux-45543148644-1.zip) | `2866e9c7c606fbff85f8bcef7a83a03b04a65bd4b4c7500ba7428256167235f6` |
| [perf-store-Windows-45543135745-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Windows-45543135745-1.zip) | `293d755b1edb99d833225a9470c5ad93063ff1dea3106d77f235dcaf7cd348fa` |
| [perf-store-Windows-45543148644-1.zip](https://github.com/nanake-lab/nemu-evidence/blob/17e29f75289a8910bddc155b2b6115a2f3276a9e/PERF-STORE-01/perf-store-Windows-45543148644-1.zip) | `50b3f34b3f89c086a27e7903f4fff74df006e63cbff6993af4d69b0d2cd249e6` |
