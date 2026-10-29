# P5-08 未解除项

状态：部分实现；无人在场期间继续完成独立工作，不等待回答，不自行发布新规律。

| 问题与证据 | 选项与代价 | 本次采用 / 建议 |
| --- | --- | --- |
| 已合并 IDL 的 `CoreTransactionV2` / `FastStateSnapshotV2` 没有 boot、clock epoch、可信墙钟证据及独立 commit_wall_time 的原子发布位置；初始 L/U/Q 也没有可供运行时构造的完整状态契约。 | ① 在运行时旁写自定义持久记录（破坏唯一契约与原子性）；② 由契约/存储负责人补充冻结契约，再接正式服务。 | 采用②；只提供显式输入的时钟计算及已有 head 的无成长准备接口，不伪造初始根、不启动正式服务。见 runtime-publication 请求。 |
| ADR 0032 D11 的观察时间、可信来源与资格规则尚未定稿；`EventEnvelope` 只有 event_time / ingest_time，后者不能冒充观察时间。 | ① 继续旧 τ 过期丢弃规则；② 保留已有时间，不增加资格规则，等后继设计补 observation / commit 语义。 | 采用②；本卡不按队列时延丢弃感受，不加入 D09/D10 的定时强制平复。见 result-time-semantics 请求，**待后继设计**。 |
| `ProposalBindingV2` 要求派发前已给出 projected / accepted_z / payload digest；真实异步 IRL 完成后的绑定转换、完整 Q 状态/通知与 gateway → runtime → fact-view 适配尚未提供。 | ① 用模拟根与同步桩声称完整冒烟；② 将同步可信适配器、存储测试与真实终端验收分开。 | 采用②；本地结构测试不是卡内真实入口冒烟，不认领 SYS14 全项、X06 全事务或 SYS23 持久重启通过。 |

用户决定涉及她的时间资格、观察归属及持续认知规律；本次不代选。工程接口可由后继契约/存储卡先提案。详见 REPORT 规格矩阵与 `tasks/requests/P5-08-*.md`。全卡验收未完成，按 AGENTS §2 保持 draft；不领取下一卡。
