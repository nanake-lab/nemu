# P5-06：缺少的正式语义边界

基线：origin/main `f24f1256259c998bc213a9dc93103fcfb9cfe89a`；2026-10-28。
仅提出接口请求，不在本卡修改 contracts/types/store/gateway，不为 #108/#64 代做实现。

| 缺口 | 当前正式位置 | 所需边界（字段与编码由契约卡裁决，本请求不定义新协议） | 本卡处理 |
| --- | --- | --- | --- |
| G01 来源与跨度 | `state.proto: SourceMetadata.source_span`、`EventEnvelope.provenance` 仅为 ObjectRef；`FactItem` 只有对象与来源 ID | 对象的正式类型、跨度单位与范围、与原始 payload 的绑定；可信身份、派生根、撤回/权限有效性；已提交 H 与本事务 staged event 的可读证明 | 只检查已提供来源集合与标签；没有 span 越界验收或 provenance 认证，绝不进入 ACCEPTED |
| G02 概念注册表 | `ConceptRef`、`ConceptProposalV2` 已有，signature/grounding/relations 仍为 ObjectRef；Ω.registry 也是 ObjectRef | 不可变 ID 的正式记录、生命周期/版本/三语标签/修订链，alias/equivalent/canonical 的版本语义，核心分配及暂存的类型边界 | 只做既有 ConceptRef 精确成员检查与独立有向 alias 算法；PROVISIONAL 返回 MissingContract，不分配 ID、不写 registry |
| G03 Claim 与确定性适配 | `Claim.value/modality/time` 为 ObjectRef，`Act.interaction_kind`、role/edge 为字符串 | 正式 operand 对象及其约束；触摸/桌面/执行/会话/维护到已注册 predicate、角色、span 的版本映射资源 | 保留并核对现有结构化 payload 的规范字节/摘要/执行 action_id，不制造 Claim 或默认概念 |
| G04 解释记录联结与采纳阶段 | `records.proto` 有各记录，`SemanticCandidateV2` 与图已分型；没有正式的 IRL 阶段结果/注册表暂存输出类型 | 已验证 actual_output、read_set、目标事件及重解释新旧记录的类型/目标/生产者关系；事务原子发布的阶段输出 | 完整 binding 比较、producer/version 与目标事件核对及已知 ID 检查只算 preflight；尚不能证明重解释引用同类同目标记录、记录或语义被采纳 |
| G05 属性规则完整生产 | `EventAttributesRecord` 具备输出字段，缺少输入 provenance 与规则表资源类型 | 将已认证 payload/事件因果来源映射到外源/自身叙述/可核实/人的规则；EV9 判定依据，停留会话和缺席事实正式后继类型 | 只检查 V2-71/EV3 已确定的外源/叙述分区与显式账号 alias；不猜 verifiable，不把当前 Desktop IDLE 当已定义的缺席事实 |

## 完成接口后的连接方向

由契约任务提供正式对象与黄金向量，来源/存储拥有者提供只读可信快照边界；semantic-core 消费这些类型完成 validator/resolver/canonicalizer。事务、服务、调度、提交仍由后继编排卡完成。不得把本卡公开的临时只读集合当成可反序列化的认证凭证。

## 验收目录

Echo Core 技能要求 `core/tests/acceptance/<ID>/`，超出本卡目录。当前单元测试在本包，报告按原编号记录 **not_run** 的完整验收。本请求同时请后续安排验收注册范围；不为通过验收扩目录或重命名原验收。
