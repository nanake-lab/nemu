# P5-25 临时动力学资源与集成请求（编号：待定）

状态：待用户确认（阻塞），未改 `core/contracts/`，未声明新 wire message。

现有 `ThetaSnapshot.estimator_state` 是 ObjectRef；具体持久化格式尚空缺。`ThetaChange`、`ThetaValues`、`NatureVector`、`NatureScale` 直接使用已有生成类型。临时 Rust 结构留在 P5-25 范围内，不能直接被当成已冻结跨进程契约。

请求后继契约卡明确：

- 估计器资源版本、全状态的规范编码、schema/数值验证、完整恢复和迁移：权重/反应/平方和、σ、τ、已关闭时长和及质量、开放符号段、检测器均值/正负累积量/计数、h、上次事件、慢漂计数，绑定 law version 和参数摘要。
- 变点参考的 shift_sigma、threshold、n_h、时长先验质量 n_tau；检测器配置或 h 算法改变必须是新的规律版本并从有效 H 重算。本次不复用 PR #73 的 R 二元后验。
- 来源投影由 H/Γ 认证，m=0 的解释已存好；不能信任任意调用方自报的 bool/enum。外层保证历史 ID 唯一性、C 在 G 后执行、同一事务提交 ThetaSnapshot/ThetaChange/L；重放不重新调用模型。
- P5-26 的跳变/慢漂信号与原子 hold 暂存接口：held 时没有状态进展，必须记录复核触发并暂停，不吞事件、不跳过事件继续学习。保留原候选以供审查不是授权存储。
- 时长右删失、零残差、同时间事件、跨2^53纳秒的精度边界，以及慢漂绝对距离单位与连续计数解释。
- 若要接入统一 `core/tests/acceptance/<编号>/` 组织，由获准范围的后继提交登记；当前测试留在 dynamics 包内，所有结果按编号存于 reports/P5-25。

验收期望：全状态 canonical round trip、完整H重放逐位一致、截断/缺字段/错law版本拒绝、失败原子性、I13事务与P5-26回放集成。不能以当前纯函数单元测试替代这些未运行的集成测试。

数值 oracle 阻塞与选项见 [BLOCKED.md](../../reports/P5-25/BLOCKED.md)，方法见 [METHODS.md](../../reports/P5-25/METHODS.md)。新编号全部待定，不占用其他任务已登记编号。
