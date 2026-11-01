# P5-08 请求：真实 F 输入与 θ 的规律资源绑定

状态：待后继接入设计；基线 main `db8802f317b5b4dd5fe57699ef724fbac0d940f1`。本次只改 `core-runtime`，不改契约、core-store、算子或预设。对应 [PR #120 审查 R1/R2](https://github.com/nanake-lab/nemu/pull/120#issuecomment-6514177758)。

## 已修复的窄边界

R⁺ 视图保留生成类型 `TendencyV3.typical_magnitude`，F 可区分审查中的两个幅度；没有给 F 增加完整 R、概念定义、证据或审计文本。运行时输入、C 返回和维护输入均经 canonical 编码与 `core_types::semantic::check_theta`，λ/σ/τ 为零的情况拒绝，不产生 STAGED 或维护投影。这里没有新增数值阈值，也没有恢复 ADR 0032 D09/D10 已取消的定时强制平复。

## 尚未具备的输入及职责

| 缺口 | 已有类型 / 接口 | 请求与责任边界 |
| --- | --- | --- |
| F 的事件类别、权威内部计数与发起轮次关联 | `core_kernel::fast::EventInput` / `EventClass`、生成事件记录；当前 `Pulse` 没有这些输入 | 由持续认知/Q、事件入口和 F 负责人明确持久来源及绑定；运行时只传经核验的数值视图。不得一律当新外源事件、默认 n=0，或从 z/原文猜类别。既有 kernel 的旧计数实现不是 ADR 0032 后继规则已获准接线的证明。 |
| 涉及人的关系权重 | 现有 `EventInput.empathy_weight` / `empathy_weight()`；A 的 persons 只有身份列表，不能证明关系值 | 由关系规律与 F 负责人明确 ADR 0032 D05 等决定后的权威来源及适配方式。不得在 runtime 默认 w=1、从 R⁺ 反推所有关系，或为接线新增关系规律。 |
| law 资源及 θ 范围 | 生成 `LawVersion` / `LawParameters`、`ThetaSnapshot.law_version`、环境 law 引用、`check_theta_within` 已存在 | 生产 C/F 适配层负责解析同一 W02 read ticket / environment 所绑定的精确资源，先核对引用、版本与来源，再用 `check_theta_within` 检查 θ 输入/输出的 g、λ 范围；runtime 负责绑定一致性及在不合格时拒绝推进。当前只存在同步可信 trait 与引用，尚未实现这条可核验资源解析链，不能把严格正数检查当成完整 law 范围验证。 |
| 结果的发生/观察/提交时间与资格 | 现有 event/ingest/process 字段不能替代 observation/commit 语义 | 延续 [D11 请求](P5-08-result-time-semantics.md)，**待后继设计**；不接入旧 τ 排队过期丢弃规则。 |

优先由现有生成资源、Store/Reader/read ticket 完成适配；只有确认现有接口不足后，才由契约/存储负责人提接口变更。本卡不另造 IPC 消息或旁写权威状态。

## 选项与建议

1. 在 runtime 填默认类别、计数、关系值和 law：会隐藏未经批准的规律与来源，拒绝采用。
2. 保留已修复的部分阶段接口，完整 F/C 接线等待上述来源和绑定明确：本次采用，建议后继卡按既有资源优先完成。

后继验收须覆盖：相同数值但不同 law 引用/版本被拒绝；正数但超出所绑定 law 范围被拒绝；不同类别/权威轮次/关系输入进入正确的 F 路径；D11 三种时间可追溯且队列延迟不被当作丢弃感受的理由。需要用户决定的持续认知、关系和时间资格规则只提案，不在接口适配时代选。

## 2026-11-01 R3 接续：完整 θ 后继类型已在 main

位置 / 依据 / 新读法：上文 θ 类型及范围检查职责 / #108、ADR 0029、main `67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e` / 运行时现直接携带 `ThetaSnapshotV2` 的完整参数、摘要及估计器，用 `check_theta_snapshot_v2` 检查向量长度、计数、时间、正数、内嵌 g/λ 范围、参数摘要和引用版本自洽；不再把旧 `check_theta` 或旧 `LawParameters` 当成新 θ 的完整参数接口。

这解除旧 θ 标签的 W02 消费者缺口，无需另改契约或 core-store。内嵌参数自洽仍不证明它们就是独立认证的权威 law/fuses，也不证明对应 H 前缀与停机来源；生产 C/F 适配层仍须解析同一 read ticket/environment 的精确资源并核对完整引用与参数，参照已有 kernel `restore_snapshot` 的可信输入边界。F 的类别/计数/关系输入与 D11 设计仍待后继接入。

旧 θ 缺少完整估计器，继续只读并拒绝 writer；本卡不从旧摘要补零或猜造迁移。既有 W02 旧头只读测试负责检查 DB/WAL/历史字节不变，运行时只接受可写 Store 中的 V2 head。
