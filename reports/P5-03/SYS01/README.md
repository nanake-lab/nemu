# SYS01 — W18 原生入口验收

入口全程使用 TerminalAuthenticate / TerminalRequest / TerminalResponse / ReceiptV2
和 core-store 公开 API。运行 `../post-P5-35/check.ps1`，每轮结果在其 runs/ 唯一目录，
子进程 stdout/stderr 在该轮 SYS01/ 下；旧初轮输出在 post-P5-35/native-initial-*，全部保留。

| 子项 | 测试/断言 |
| --- | --- |
| 认证/上传→D | acceptance::sys01_authenticated_upload_dispatch_reconnect_gamma_and_reverse_receipts；DPAPI create/load 后真实握手，上传完整 canonical payload/provenance，返回服务端 D 位置/接收时间 |
| 重投、冲突、Γ | D=e1,e2,e2,conflict；H=e1,e2；真实 commit 两次，processed=4、anchor=2、ledger=2；冲突仅追加独立 enum 审计，原事实不可覆盖 |
| 延迟/乱序回执 | 两条 D 在未提交时仅 RECEIVED_DURABLE；真实 Store.commit 的 e2/e1 逆序通知仍分别配对 r2/r1；无实际发布不能 CORE_COMMITTED |
| 并发/重连 | 另一认证连接在首连接尚有 D 时查询成功；重连产生新随机 session，原 source key 重投 FILTERED |
| SQL 前骤停 | 子进程 Fault::Crash(IngressBeforeSql) exit=86；新进程从 raw=0 接收/提交，无跳号 |
| SQL 后未见证骤停 | 子进程 Fault::Crash(IngressSqlCommitted) exit=86；新进程恢复已写 D，原键重试占 raw=1，Γ 仅允许一次提交 |
| 收到持久接收后未提交骤停 | 父进程确认 RECEIVED_DURABLE 后 TerminateProcess，子进程 exit=1；新进程重开并处理原 D，重试 FILTERED，H=1、anchor=1、ledger=1 |
| 伪造回执/背压 | storage_failures：假 CoreCommitResultV2 不得转发；实际 Store 条数/字节满额回 typed reason 且不消耗 D 序号，相同键容量恢复后成功 |

合成事务处理器仅在测试中持有实际 Store CommitCapability；它不实现/证明成长行为或 P5-08 业务编排。
崩溃子进程必须验证 D 盘 TEMP 下新建的父根、随机一次性 permit、case/phase 与进程内 LOCALAPPDATA；
只允许打开父根内新建的合成库。无真实用户库、系统账户或系统设置操作。

物理电源中断 not_run。进程骤停可验证 WAL/见证/重投逻辑，不能证明磁盘缓存/NTFS 在真实失电时的硬件行为；
是否据此完成本卡该项，由用户明确，不自动放宽。
