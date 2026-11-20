# P5-38 卡外改动与读端对照

行号为提交中的差异块起点。除产物和结构夹具外，表中给出了每一处改动的读取或发布含义。

| 位置 / 行 | 依据 | 新读法 |
| --- | --- | --- |
| [core/Cargo.lock](../../core/Cargo.lock#L159) · 159、169 | 用户最终依赖确认 | 只登记 task-layer 与已锁定的 sha2 0.11.1 两条直接依赖边。 |
| [core/contracts/templates/python/cognition.py](../../core/contracts/templates/python/cognition.py#L32) · 32、34、78、81 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |
| [core/contracts/templates/typescript/cognition.ts](../../core/contracts/templates/typescript/cognition.ts#L31) · 31、70、73 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |
| [core/contracts/verify_cognition.py](../../core/contracts/verify_cognition.py#L5) · 5、40、52 | 用户确认新增标签正反例 | 在原向量之外分别以新记录和新提交标签检验接受与拒绝；三端比较规范字节。 |
| [core/crates/core-runtime/Cargo.toml](../../core/crates/core-runtime/Cargo.toml#L12) · 12 | 用户最终依赖确认 | 只登记 task-layer 与已锁定的 sha2 0.11.1 两条直接依赖边。 |
| [core/crates/core-runtime/src/lib.rs](../../core/crates/core-runtime/src/lib.rs#L11) · 11 | AGENTS §2 模块登记，用户确认存储范围 | 公开本卡 cognition 或存储模块入口与类型。 |
| [core/crates/core-runtime/src/store.rs](../../core/crates/core-runtime/src/store.rs#L33) · 33、69、125、139、144 | ADR 0046 §4，批准的事件生产点 | 从最近发布的权威根准备事件，按原绑定直接构造 V4.event_binding。 |
| [core/crates/core-runtime/tests/store.rs](../../core/crates/core-runtime/tests/store.rs#L2) · 2、23、55、120、123、151、403 | 临时数据库验收 | 共用原有真实存储引导夹具；允许合成的 Q 与已接受的外部解释，支持内部提交后的外部事件测试。 |
| [core/crates/core-store/Cargo.toml](../../core/crates/core-store/Cargo.toml#L10) · 10 | 用户最终依赖确认 | 只登记 task-layer 与已锁定的 sha2 0.11.1 两条直接依赖边。 |
| [core/crates/core-store/src/activity.rs](../../core/crates/core-store/src/activity.rs#L1) · 1 | ADR 0045 §4、0046 §2.1 | 绑定已登记的输入、验证采纳及解释，核对前后根、Q 和后继，区分内部发布与仅审计结果。 |
| [core/crates/core-store/src/activity/transition.rs](../../core/crates/core-store/src/activity/transition.rs#L1) · 1 | ADR 0045 §4、0046 §2.1 | 绑定已登记的输入、验证采纳及解释，核对前后根、Q 和后继，区分内部发布与仅审计结果。 |
| [core/crates/core-store/src/activity_audit.rs](../../core/crates/core-store/src/activity_audit.rs#L1) · 1 | ADR 0045 D8，用户确认拒收审计 | 按原回执幂等记下原因和不可变引用，恢复时独立验证。 |
| [core/crates/core-store/src/derived.rs](../../core/crates/core-store/src/derived.rs#L5) · 5、33、45、50、55、59、91、100、104、127、145 | 用户确认重建、混合历史及恢复 | 按来源重建事件事实或内部索引/在途行动，校验独立游标、根与唯一性。 |
| [core/crates/core-store/src/ingress.rs](../../core/crates/core-store/src/ingress.rs#L113) · 113、122、142 | 用户确认事件生产与已有事件行为 | 以 V4 发布外部事件；为外部输入、重复项和内部提交分配独立的物理提交位置。 |
| [core/crates/core-store/src/lib.rs](../../core/crates/core-store/src/lib.rs#L5) · 5、13、20 | AGENTS §2 模块登记，用户确认存储范围 | 公开本卡 cognition 或存储模块入口与类型。 |
| [core/crates/core-store/src/manifest.rs](../../core/crates/core-store/src/manifest.rs#L1) · 1 | ADR 0046 §4，同类读端预批准 | 先按原标签解码；事件专用接口拒绝内部和未知来源；可取回原快照身份。 |
| [core/crates/core-store/src/migration.rs](../../core/crates/core-store/src/migration.rs#L19) · 19、28、30、33、40、71、89、107、119、126 | SYS17，用户确认为存储格式改动 | 格式 3 增加活动身份与审计表及持久回执目录；格式 2 仍按旧结构核验。 |
| [core/crates/core-store/src/objects.rs](../../core/crates/core-store/src/objects.rs#L1) · 1、24、35、73、99、153、162、232、239、288、298 | ADR 0046 §2.1，用户批准字节资源及预算 | 区分存储类型，按原始长度与 SHA-256 核验，继承对象和存储总预算，禁止原始资源走 CanonicalV1。 |
| [core/crates/core-store/src/publication.rs](../../core/crates/core-store/src/publication.rs#L8) · 8、12、28、141、179、194 | 用户确认重建、混合历史及恢复 | 按来源重建事件事实或内部索引/在途行动，校验独立游标、根与唯一性。 |
| [core/crates/core-store/src/queries.rs](../../core/crates/core-store/src/queries.rs#L66) · 66、92、144 | 用户确认存储兼容与 W19 用途 | 区分发布位置和外部输入位置，按显式前缀投影事件；保留最后已发布的快照。 |
| [core/crates/core-store/src/raw.rs](../../core/crates/core-store/src/raw.rs#L1) · 1 | ADR 0046 §2.1，用户批准字节资源及预算 | 区分存储类型，按原始长度与 SHA-256 核验，继承对象和存储总预算，禁止原始资源走 CanonicalV1。 |
| [core/crates/core-store/src/reader.rs](../../core/crates/core-store/src/reader.rs#L10) · 10、66、119、128、130、134、138 | ADR 0046 §4，同类读端预批准 | 先按原标签解码；事件专用接口拒绝内部和未知来源；可取回原快照身份。 |
| [core/crates/core-store/src/recovery.rs](../../core/crates/core-store/src/recovery.rs#L11) · 11、64、78、81、139、229 | 用户确认重建、混合历史及恢复 | 按来源重建事件事实或内部索引/在途行动，校验独立游标、根与唯一性。 |
| [core/crates/core-store/src/store.rs](../../core/crates/core-store/src/store.rs#L87) · 87、129、188、213、322、346、362、378、393、411 | 用户确认存储兼容与 W19 用途 | 区分发布位置和外部输入位置，按显式前缀投影事件；保留最后已发布的快照。 |
| [core/crates/core-store/src/tests/atomic.rs](../../core/crates/core-store/src/tests/atomic.rs#L34) · 34 | 原有存储验收 | 对已有事件调用保留事件视图，对实际发布对象使用其原标签。 |
| [core/crates/core-store/src/tests/consistency.rs](../../core/crates/core-store/src/tests/consistency.rs#L250) · 250、262、266、312 | SYS17、旧格式故障注入 | 保留原格式专用夹具，覆盖新增目录的同步与故障，验证恢复后可读写。 |
| [core/crates/core-store/src/tests/crash.rs](../../core/crates/core-store/src/tests/crash.rs#L234) · 234 | SYS17、旧格式故障注入 | 保留原格式专用夹具，覆盖新增目录的同步与故障，验证恢复后可读写。 |
| [core/crates/core-store/src/tests/format_three.rs](../../core/crates/core-store/src/tests/format_three.rs#L1) · 1 | SYS17，合成旧库 | 构造真实旧标签的历史，验证迁移、回退、标签隔离、拒绝有损回退和字节资源。 |
| [core/crates/core-store/src/tests/isolation.rs](../../core/crates/core-store/src/tests/isolation.rs#L111) · 111 | 原有存储验收 | 对已有事件调用保留事件视图，对实际发布对象使用其原标签。 |
| [core/crates/core-store/src/tests/legacy_v2.rs](../../core/crates/core-store/src/tests/legacy_v2.rs#L199) · 199 | SYS17，合成旧库 | 构造真实旧标签的历史，验证迁移、回退、标签隔离、拒绝有损回退和字节资源。 |
| [core/crates/core-store/src/tests/mod.rs](../../core/crates/core-store/src/tests/mod.rs#L88) · 88、229 | SYS17，合成旧库 | 构造真实旧标签的历史，验证迁移、回退、标签隔离、拒绝有损回退和字节资源。 |
| [core/crates/core-store/src/tests/terminal_ingress.rs](../../core/crates/core-store/src/tests/terminal_ingress.rs#L127) · 127、354 | SYS17、旧格式故障注入 | 保留原格式专用夹具，覆盖新增目录的同步与故障，验证恢复后可读写。 |
| [core/crates/core-store/src/tests/terminal_ingress/migration_checks.rs](../../core/crates/core-store/src/tests/terminal_ingress/migration_checks.rs#L25) · 25、47 | SYS17、旧格式故障注入 | 保留原格式专用夹具，覆盖新增目录的同步与故障，验证恢复后可读写。 |
| [core/crates/core-store/src/transaction.rs](../../core/crates/core-store/src/transaction.rs#L7) · 7、13、30、53、108、115、140、151、226、233、284、336 | 用户确认事件生产与已有事件行为 | 以 V4 发布外部事件；为外部输入、重复项和内部提交分配独立的物理提交位置。 |
| [core/crates/core-store/src/witness.rs](../../core/crates/core-store/src/witness.rs#L38) · 38、95、100、107、114、152、156、211、239、242、249、276 | SYS17，用户确认为存储格式改动 | 格式 3 增加活动身份与审计表及持久回执目录；格式 2 仍按旧结构核验。 |
| [core/crates/core-types/src/semantic/cognition.rs](../../core/crates/core-types/src/semantic/cognition.rs#L18) · 18、112、115 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |
| [core/crates/fact-view/src/index.rs](../../core/crates/fact-view/src/index.rs#L7) · 7、24、105 | 用户确认 W19 快照身份兼容 | 索引保留原始前缀快照引用，以实际发布的快照判断当前性。 |
| [core/crates/fact-view/src/retrieval.rs](../../core/crates/fact-view/src/retrieval.rs#L125) · 125 | 用户确认 W19 快照身份兼容 | 索引保留原始前缀快照引用，以实际发布的快照判断当前性。 |
| [core/crates/gateway/src/progress.rs](../../core/crates/gateway/src/progress.rs#L64) · 64 | 用户批准的回执读点 | event_snapshot 按旧 V2 或 V4 事件来源读取，之后继续核验观察到的提交、事件、分支与摘要。 |
| [core/crates/task-layer/src/lib.rs](../../core/crates/task-layer/src/lib.rs#L5) · 5 | AGENTS §2 模块登记，用户确认存储范围 | 公开本卡 cognition 或存储模块入口与类型。 |
| [packages/contracts/src/generated/echo/cognition.ts](../../packages/contracts/src/generated/echo/cognition.ts#L31) · 31、70、73 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |
| [workers/drive/generated/echo/cognition.py](../../workers/drive/generated/echo/cognition.py#L32) · 32、34、78、81 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |
| [workers/irl/generated/echo/cognition.py](../../workers/irl/generated/echo/cognition.py#L32) · 32、34、78、81 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |
| [workers/model-adapter/generated/echo/cognition.py](../../workers/model-adapter/generated/echo/cognition.py#L32) · 32、34、78、81 | ADR 0046 §4，用户确认两个引用读点 | latest_step、origin_record 接受两代活动记录标签；commit 标签仍拒绝。 |

W19 另有两个间接消费者。fact-view/src/source.rs:101 与 fact-view/src/memory.rs:69 只消费 Reader::transactions_at 返回的外部事件。两处仍按 EventEnvelope 核验其 staged_h。组合回归从内部提交之后的快照调用实际 W19，可以同时检验事件正文、旧反应及显示标记。[回归源码](../../core/crates/core-runtime/src/cognition/tests/w19.rs)

core-store/src/tests/consistency.rs 中直接调用 V2::from_canonical 的测试只对测试自身的 V2 值作原格式往返。生成类型和兼容探针继续保留旧标签。生产端直接读取 V2 原始字节的分支位于 manifest.rs 与 reader.rs。旧 V3 的通用类型读取仍须使用原始 V3 标签，主分支没有历史 V3 事务生产者。

## 对应回归用例

| 读点 | 可定位的测试 | 核对事项 |
| --- | --- | --- |
| manifest、gateway、reader 的事件回执边界 | [sys17_v2_v4_event_history_and_rollback_preserve_bytes](../../core/crates/core-store/src/tests/format_three.rs#L5) | 旧 V2 事件和 V4 事件都可读取；未知标签和错代直接解码拒绝。 |
| 同一回执读点与当前头查询 | [local_and_remote_replay_identically_across_both_publication_boundaries](../../core/crates/core-runtime/src/cognition/integration_tests.rs#L12) | V4 活动是可读的当前快照，event_snapshot、Store::current 和 Reader::current 的事件接口均拒绝它。 |
| runtime/store、store、queries、W19 事件来源 | [external_event_after_internal_round_keeps_the_two_cursors_and_sources_separate](../../core/crates/core-runtime/src/cognition/integration_tests.rs#L251) | 外部提交、内部提交、再外部提交，重启后取回两个外部事件与中间活动头。 |
| fact-view 的 index、retrieval、source 和 memory | [w19_historical_reaction_after_internal_publication](../../core/crates/core-runtime/src/cognition/tests/w19.rs#L13) | 旧索引返回滞后；新索引从实际 W19 取得旧事件及其反应，两者保留来源与显示标记。 |
| derived、recovery、witness 以及活动审计 | [attempt_before_and_after_sql_recovers_without_advancing_q](../../core/crates/core-runtime/src/cognition/integration_tests.rs#L332) | 审计事务在提交前后分别故障；重启、重试后只有一条审计，Q 与头不动。 |
| W02 事件前缀预算 | [complete_transaction_prefix_is_verified_and_bounded](../../core/crates/core-store/src/tests/reader.rs#L4) | 原有预算与输入完整性断言保留。 |
| W02 派生索引 | [all_derived_tables_match_authoritative_rebuild](../../core/crates/core-store/src/tests/consistency.rs#L129) | 原有投影重建断言保留。 |
