# P6-01 剩余接线与验收

SYS14 卡面冲突已解除。[#250 最新确认](https://github.com/nanake-lab/nemu/pull/250#issuecomment-7059859384) 采用已批准的 [V2-170](../../docs/traceability/diff-register.md#L743)。卡面已在本 PR 修订，用户合并时批准文字改动。

| 未完成项 | 当前证据与恢复条件 |
| --- | --- |
| 权限与真正发送 | P5-40 [#235](https://github.com/nanake-lab/nemu/pull/235) 的 ADR 0047 / permission.proto 仍在草稿。依用户确认等待其合并，再绑定真实范围、凭证与撤销版本。 |
| SYS08 / SYS13 | P5-14 [#249](https://github.com/nanake-lab/nemu/pull/249) 尚未合并。替身能计数、拒绝过期绑定、验证无行动时不登记。真实并发观察持久化、发送中断、已发范围、UNKNOWN、恢复重放与不重发均为 not_run。依赖合并后换用真实 outbox 跑原判据。 |
| 外层事务、网关与生成器 | task-layer 已提供有类型的输入、运行绑定、回合状态机与查询入口。当前 main 的外层事务复用 old.staged_q，网关默认拒绝查询。宿主需在授权目录内写入 Q 和审计、推送故障、恢复结束的回合、接入真实载体。详见[接口请求](../../tasks/requests/P6-01-runtime-consumers.md)。当前桌面体验与生产生成回执为 not_run。 |
| 协议定稿与样例 | 三语 p v0 留在候选目录，N2/P6-12 由用户阅读签字。D10 的选择接口需由后继实现绑定已签字的样例库，整库签字及 JT01/JT02 实测仍属 P5-17/P6-04。 |

可以先审查任务层与卡面，依赖合入后继续接线和真实验收。也可待依赖合入后一次审查，代价是推迟已实现部分的反馈。
