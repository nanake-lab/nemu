# P5-34 当前阻塞（2026-10-26，PR #95）

## B1：卡外 core-store 仍生产 V2

本卡要求旧消息标为取代；当前 main 的 `core-store/src/objects.rs::prepare` 拒绝 `T::SUPERSEDED`，而 `store.rs::prepare_r`、证据组生产者与查询仍使用 V2。V3 的类型和校验已提供，但消费者未切换。

实测：完整 Rust 工作区退出 101。`tests::objects::p532_evidence_versions_and_ledger_prefix_are_complete` 在 `core-store/src/tests/objects.rs:172:49` 通过通用 prepare 写 EvidenceGroupV2，收到 `Rejected("new writes must use the active successor message")`；core-store 为 30 通过、1 失败、0 忽略，耗时 1077.61 秒。原始日志和命令/退出码位于 `verification/resumed-20261026/attempt-01/workspace.log`、`.json`。该失败没有重跑来替代诊断。

Ubuntu CI run 40271343235（head b84d0bf）也复现相同拒绝，退出 101，30 通过/1 失败；原始错误摘录及 job 链接见本轮 ci-b84d0bf.json。因此该消费者冲突并非只在本机发生。

旧 PR #72 的 core-store 扩围实现已按用户本次范围要求撤回；不能使用它的旧成功日志证明当前消费者兼容。没有取消 V2 取代标记、放宽新写入拒绝、改测试或默认补 q/V。已更新 `tasks/requests/P5-34-core-store-v3.md`，继续执行契约及其他不受影响的检查。

可选处理：

1. 安排有权修改 core-store 的独立迁移工作，协调依赖/目录占用后接入，再合入 P5-34 并完整复验。
2. 用户修订本卡目录并安排串行窗口后，恢复并审查旧分支中的存储适配，按最新 main 重新验证。

P5-35 已依赖 P5-34，不能假定直接等 P5-35 就能消除这个合并前阻塞。当前不扩大代码范围。

## B2：卡外 Windows 进程监督器测试失败

`pnpm test` 退出 1：52 个文件中 51 通过，580 项中 579 通过、1 失败。`apps/desktop/src/supervisor/supervisor.test.ts:199` 的 hung-process 清理测试期望空 PID 数组，实际为 `[87292]`。原始日志见 `verification/resumed-20261026/attempt-01/test.log`。随后只读检查该 PID 已不存在；这不撤销断言时的失败，也不足以判断根因。本次与 origin/main 的 supervisor 代码差异为零，没有证据把它归因于 CI 作业冲突。

未改 apps/desktop、等待时长或断言，未用重跑成功覆盖失败。后续可由桌面模块维护者诊断停止/退出确认的时序，修复后在本分支复验；也可在独立授权的排查中确定环境因素。P5-34 继续保持该检查失败。

## B3：契约交叉审查与用户批准

ADR 0021、V2-62/63 与具体 q/V 后继契约仍待交叉审查及用户批准；PR 标 `needs-user`，由用户合并。ADR 0019、V2-55–59 已由 V2-84 确认采纳，不重复列为待决。实现者不自行审查、不另开审查聊天。

真实 G 重放、生产迁移、AC2-08/19/21、I23、SYS17 属于后续集成范围；本次未访问排练或正式存储，也未签发这些验收。历史失败、原始日志和旧范围记录保留，当前结果以 REPORT.md 和 `verification/resumed-20261026/` 为准。
