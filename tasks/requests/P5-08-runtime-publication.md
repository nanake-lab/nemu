# P5-08 请求：运行时权威发布与接入接口

状态：待后继设计/实现；不修改契约、core-store 或其他 crate。基线 `02948dc11aa1b43dc0dfe67c9da2e61676427385`。

## 已确认的接口边界

- 使用已生成 `ProposalBindingV2`、`CoreTransactionV2`、`RSnapshotV3` 等，不增加手写 IPC 类型。
- W02 已提供 durable ingress、重复过滤、read ticket、prepare/stage/commit、committed reader、单 writer 与 crash recovery；本卡不重复实现这些机制。
- `CoreTransactionV2` 的 anchor/process 字段不能表达跨 boot/epoch 的可信恢复证据及 commit_wall_time。当前没有完整 L/U/Q 初始根的生成消息，不能以测试用 UserText 资源充当生产根。
- `Store::bind` 在派发前固定全绑定（含 accepted_z_digest）。真实异步 IRL 完成后如何形成、关联并重核后续 A/M 的绑定，需要所属边界明确；本卡只检验已形成绑定的同步可信适配器。

## 请求与归属

1. 契约 + core-store：定义时钟 checkpoint、boot/epoch、可信墙钟及不确定度、process_time 与 commit_wall_time 的关系；说明故障后的拒绝/恢复边界、与 H/x/R/θ/L/Q/U 原子发布及 replay 的关系。不用旁写文件替代原子提交。
2. 契约 + core-store / echoctl：明确排练初始状态与正式 genesis 的 L/Q/U 根、阶段能力、运行状态、暂停/复核标志、系统维护与停机记录；ADR 0028 的封存/恢复和正式单实例由所属能力证明，不接受可伪造布尔值证明封存。
3. 语义/permission 边界：提供 IRL → 已接受 z → A/M 的绑定演进、持久化内容、超时/worker 重启/旧结果的接纳或拒绝接口；每次都核对完整 binding 与 read set。M 的 ConceptRef 解析只能给概念定义，不能把 R 强度或 x/m 泄漏进去。
4. gateway + fact-view：接入持有唯一 commit capability 的 runtime handler；真正 user_text/结构化事件进入 D、事实投影从提交前缀构造、提交后检索；变化后推送已提交 Q 对应的 run_state。不能把本卡 W02 结构测试当作此链路完成。
5. P5-19/P5-29 及相关算子所有者：提供实际 F/G/C 实现与有限数值/证据账本约束，接通暂停期间记录、恢复补算。接口 mock 仅验证顺序与隔离，不构成数学实现验收。

## 建议验收

- 一次事务只捕获一次 process_time；延迟提交有不同 commit_wall_time，crash/restart 重放不重计时间。
- 未知停机时长、墙钟倒退/跳变显式故障，恢复不会通过 anchor+1、忽略异常或定时平复修补。
- 所有 STAGED 结果原子成为 COMMITTED；提交失败对 Reader 不可见；Q/通知与所读快照一致。
- 真正临时排练入口冒烟；正式/实验缺阶段拒绝；封存与正式单实例有效；禁止用生产/用户数据测试。

本请求不指定新的时间阈值、成长规律或她的身份选择。优先补冻结契约，再接运行时服务；保留 P5-08 部分实现直至全部验收。
