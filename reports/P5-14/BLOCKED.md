# P5-14 尚待验证的部分

| 项目 | 已有证据 | 接续条件 |
| --- | --- | --- |
| P5-40 主线基线 | 本分支从 #235 的 `59a79fab4ac2f62836a67965f2ea4ad61aacf0cc` 开工。 | [#235](https://github.com/nanake-lab/nemu/pull/235) 合并后，merge 最新 origin/main 并重跑全部检查。 |
| SQLite 派生 outbox 行 | #235 的发布器只为根类型为单个 ActionLifecycle 的 Q 产生此表的行。P5-14 提交完整 CognitionState，其 outbox 记录随事务审计引用持久化。缺行或过时的索引提示可回退到 Q，并发出诊断。若手工改坏物理 SQLite 行，W02 读者会在读取 Q 之前拒绝整库校验；本卡未绕开该校验，物理损坏后的继续派发未验证。 | 按用户裁决，物理索引组合检查为 **not_run**，由与 [#238](https://github.com/nanake-lab/nemu/pull/238) 后合并的一方负责。[追溯表和运行命令](../../docs/traceability/P5-14.md) |
| Windows CI | Windows 本机全工作区 release 测试通过。旧提交 82593109 的手动作业已结束，W18 的 SYS01 网关用例失败。 | 待 #235 合并，把含 #240 修复的主线合入，重跑 Windows 工作流。 |
| 生产认知来源与执行链 | 正向 W02 测试用 #235 支持的本地来源，云端 V2 来源拒绝已单测。 | #238 提供新版 Q 来源组合验证。排练版外层事务到真实启动服务依赖 P5-08 的后续接线。P5-41 承接完整 SYS09、真实执行器 I26 与双层隔离。 |

本卡提交的是草稿，等 Claude 交叉审查。
