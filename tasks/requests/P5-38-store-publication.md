# P5-38 权威存储与运行时事件生产点的最小改动请求

P5-38 卡面目录只覆盖运行时与任务层的 cognition 子目录。申请在本 PR 内修改下列路径，等待范围确认。

- core-store 的事务、存储格式、恢复、查询和相关测试。当前 stage 只接收 CoreTransactionV2，必须消费下一条 raw ingress；commits.position 唯一，恢复又逐条要求外部事件。独处期间的多次发布必须能在同一 ingress 游标上发生，继续使用唯一 writer、同一 SQLite 事务、对象可见性和崩溃恢复。旧标签按旧格式读取。需要以临时数据库验迁移、回退、混合历史、拒收审计以及提交前后故障。
- core-runtime/src/store.rs 的事件生产点。事件使用原 ProposalBindingV2 直接生产 CoreTransactionV4.event_binding，沿用原有绑定、顺序与快照检查。
- 读端按实际用途区分事件、内部提交与仅审计的尝试。已有事件消费者的查询仍须能得到正确的事件前缀；不可将内部记录伪装为 EventEnvelope。

另有一处需 P5-39 协调：现有三语言认知状态语义检查只允许旧记录作为 latest_step 与 origin_record。请求允许在本卡提前接通这两个读端对新记录标签的支持，并保留旧标签的正反例；或者由 P5-39 先行提供兼容读端。按派卡约定不改 core/contracts/。
