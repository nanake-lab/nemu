# FIX-STORE-01 规格对照

| 规格条款 | 要点 | 实现 | 测试 |
| --- | --- | --- | --- |
| v2.0 §2.2、I16/I23 | 权威记录可重放；派生不成为独立权威 | core-store derived/recovery | all_derived_tables_match_authoritative_rebuild |
| v2.0 §2.5、I01 | 账本保留追加序 | queries::ledger_at | ledger_order_survives_normal_commit_and_reopen |
| I02、I29；ADR 0005 | 完整恢复准入；测试只用临时谱系 | witness::admit；TemporaryRoot | initialization_crash_repairs_empty_reject_namespace；missing_reject_namespace_on_nonempty_store_refuses_writer_and_reader |
| v2.0 §5.3；V2-178 | E/C 实数含义不变，显式数值边界 | law/recurrence.rs | evidence_components_avoid_intermediate_overflow_and_preserve_normal_bits |
| I01、I18 | 索引损坏不能静默影响历史 | derived::verify、collect_garbage | missing_reference_rows_are_rejected_before_gc；missing_latest_group_is_rejected_instead_of_resurrecting_association |
