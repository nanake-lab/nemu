# ADR 0030：持续认知记录、工作状态与内部提交契约

状态：PROPOSED，P5-37 契约变更；needs-user，由用户合并批准。基线 8839dcae。

## 背景与有效范围

ADR 0026 §§3–6、8、10 要求类型化的内部产物、Q 工作状态、输入/回执及原子提交材料。现有 PlanAndCarrierReceipt 的输出和模式 C 草稿不能冒充内部工作记录；ProposalBindingV2 是事件解释绑定，不能为内部请求伪造事件解释字段。

以 DOC-PRESETS-01 当前读法、ADR 0032 §§5、7 和 ADR 0035 §§4、5 为准：不新增 κ、κ^n、外源重置计数或心理结果截止点；不恢复 COG-A12。round_index 仅为本活动的请求顺序，供追溯，不是“自上个外源事件以来”的心理权重。旧 ActionPlan.deadline_ns 仍属执行计划的既有合同，不能用于判断 F 资格。分阶段消费键、发生/观察时间事实与 D11 资格交 P5-72。此处没有新的 F 依据身份；需要解释作用时引用 P5-68 FastEffectRecord/FastBasisIdentity，不另造标识。

## 决策与字段出处

新文件 cognition.proto，全部 schema_version=1；每条消息分配独立 echo.* CanonicalV1 标签。字段号、枚举数值和编码以 IDL 为唯一源，沿 ADR 0007；无默认参数、心理评分或新的时间规律。

| 消息/字段组（schema_version 均来自 ADR 0007） | 来源与语义 |
| --- | --- |
| CognitionBinding：activity_id、step_id、request_id、round_index | ADR 0026 §5 的活动/步/请求身份；请求顺序用于接续/去重（§6.4），不解释为情绪权重 |
| 同上：branch_id、base_commit、q_revision、input_frontier、snapshot_id | §§4.1、4.3、5、6.5–6.6：精确输入前沿与已提交 Q/快照，旧结果不能覆盖新 Q |
| 同上：environment、protocol、permission_snapshot、source_validity_version、read_set | §§4.2、5.1、6.5–6.6、7：版本化资源、权限、来源闭包与显式依赖；环境复用 VersionedEnvironmentV2 |
| CognitiveMaterial：content、metadata、past_reaction | §§5、5.1；V2-88/102：H 内容引用、已有 SourceMetadata、过去反应专用标记；标记不把推断变事实 |
| CognitiveReturnedResult：request_id、status、result、action_id | §§3、5.2、6.9：等待/无命中/拒绝/索引落后/缺失/预算/失败/未知分别表示；成功须实际结果；可关联既有 action_id |
| CognitiveInput：binding、materials、focus、new_inputs、results、retrievals、budget | §§4.1–4.2、5.1–5.2：本轮材料、关注引用、新输入、实际返回及 FactProjection，容量规则为显式资源引用 |
| CognitiveResultAndReceipt：binding、input、products、next_focus、queries、action_plan、completion、reason | §§4.4–4.5、5、6.7、7：模型 Proposal，实际产物、下一关注/查询提议、可选现有 ActionPlan；正常/截断/取消/失败分开，未完成的三类不得发布完整后继计划；空产物可表示合法 no-op |
| 同上：model_hash、tokenizer_hash、modulation、carrier_receipt、usage | §§5、7、9：实际资源与调制/用量回执；成功或截断须载体回执，失败/取消可尚未产生回执；编码不证明真的注入 m |
| InternalActivityRecord：record_id、previous、result、started_at_ns、finished_at_ns | §5、§6.7：H 中的内部记录，前序引用、真实结果和执行区间。区间是请求执行计时，不是 D11 的事件发生/观察时间 |
| ActivityState：activity_id、revision、materials、focus、latest_step、waiting、successor、status | §§3、5：Q.P 当前工作引用与局部运行/等待/完成/挂起；完成不要求清空材料 |
| CognitiveInflightAction：action_id、origin_record、origin_step_id、round_index、registered_at_ns、plan、status、request、actual_result、credential | §6.3 中保留的 outbox 起点/来源记录，§6.9 的 action_id、执行凭证与真实结果；不保存已取代的 κ 和心理期限。UNKNOWN 不意味着允许重发 |
| CognitionState：revision、mode、reason、activities、inflight、active_request、runnable_successor、transition | §§3、5、6：独立全局 RUNNING/PAUSED/FAULT 与局部活动，显式转换引用。RUNNING 必须且只处于 active_request 或 runnable_successor；暂停/故障须原因且无可执行后继，不清除历史活动/在途记录 |
| InternalActivityCommit：binding、record、interpretations | §§5、6.1–6.2、6.10：Trusted Core 的内部发布分支；绑定必须与记录的回执一致，解释引用为已接受的 m=0 路径材料，不伪造 ProposalBindingV2 |
| CoreTransactionV3：继承 staged_h/x/r/theta/l/q/u、processed_position、anchor_time_ns、staged_concepts、accepted_results、audit_refs、staged_ledger；新增 event_binding / internal_commit 必选一项 | §6.1 的同一外层事务发布 H/Q；§6.2/6.10 允许真实候选回溯写 L，不伪造 θ 成长；保留 CoreTransactionV2 的相同字段号，15 reserved，新分支 17/18。旧事件路径仍用 ProposalBindingV2 |
| SystemMaintenanceRecord：record_id、kind、reason、details、ledger_refs、related_commit、recorded_at_ns | V2-87/102：暂停、恢复、复核、报警、备份恢复的维护记录；与 EpistemicType.SYSTEM_RECORD 的触摸/观察来源标签区分；不直接进 F/G/C。读取后的新反应另行记录 |

自致事件类别不另造一个 F 身份枚举：InternalActivityRecord 的类型明确表示内部记录；CognitiveInflightAction 的 origin_record/origin_step_id 与 action_id 明确表示她发出的行动及其实际返回。结果通过 CognitiveReturnedResult.action_id 关联，正式作用仍引用 P5-68 的 FastEffectRecord。round_index 仅供这条请求链追溯；不补写已被取代的 n/κ。模型与 tokenizer 哈希绑定 environment 中相应资源的 digest，哈希域沿用既有 ObjectRef 资源身份；载体实际执行真实性由消费者核验。

引用必须由拥有 H/资源的可信调用方解析；本卡只验证字段、枚举、明确的类型与同消息绑定，不把引用存在、来源认证、权限、载体真实性、旧 Q 防覆盖或原子发布伪装为编码器能力。queries 保存有版本查询计划的 ObjectRef，查询内容与确定执行算法仍由既有事实查询所有者解释，不新增检索策略。认知分支提出 ActionPlan 时，task_version 使用本轮绑定的 q_revision，snapshot_id 与 protocol_version 也必须与本轮一致；这不重解释旧 ActionPlan 的字节或其他任务路径。

## 编码、边界与验收

所有集合按 IDL set 键规范字节排序、重复拒绝；产物、材料、查询、结果与输入顺序保留。uint64 完整保留，大于 2^53 不经浮点；字段未知、枚举未知、缺失与额外分支、非有限、预算超限按现有编解码器拒绝。

三种语言的显式语义入口在结构解码之后校验请求时间区间、合法结果/回执组合、全局状态后继、同一内部提交的完整绑定、内部记录与行动来源类型；不能只检查外层而跳过嵌套消息。每消息至少三合法/三非法黄金向量，新增交叉字段及两种 wire 解码负例；旧黄金字节逐一不变，check_compat 对全部基线零违例。

## 兼容、迁移与回退

旧消息保持可解码、可用于既有事件路径。CoreTransactionV3 是可选后继，使用已有 `@retains CoreTransactionV2`：仍执行继承字段与 reserved 的兼容检查。暂不加机器 `@superseded` 标记，因为 `core-store/src/objects.rs` 会拒绝旧类型新写入，而当前 store/runtime 仍生产 V2。P5-38 接线后再停用 V2；这项分阶段安排偏离卡中立即标记旧消息的通用要求，登记 V2-113 由用户批准。不修改生产存储以赶在本卡启用新事务。不修改旧 CoreTransactionV2 字节，不把旧输出块/草稿补写为内部历史，不从旧 Q 猜测从未保存的工作材料。部署升级须先备份，能力协商后才启用新分支；初始认知状态由真实已提交材料与明确模式转换建立，不能伪造过去思考。

回退恢复旧应用与其兼容备份；保留新对象原标签，不让旧解码器读成旧事务。恢复未决外部副作用须按 action_id 与真实回执核对，UNKNOWN 不盲重发。本卡提供编码/旧字节/类型隔离证据，不声称执行 SYS17 迁移、H/L/Q 原子性或真实持续认知；这些属于 P5-38、P5-39、P5-75 等后续卡。

## 备选与后果

不采用复用模式 C 草稿或给旧事件绑定塞假字段；不采用隐藏 worker 记忆、默认心理参数或预定义 P5-72。代价为新增引用和明确的消费者语义边界；协议真实性和运行体验仍须后续测量。用户批准本契约与独立交叉审查均为合并前提。
