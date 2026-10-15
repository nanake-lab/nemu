# PR #7 用户评审修改

本记录替代 REPORT.md 中“启动后立即异步采集”的描述；其余验收判据不变。

- 将原窗口计数断言放入 try，进程快照放入 finally。成功或失败都在原断言结束后才启动 pwsh，并等待该快照结束，然后继续后续测试或传播原断言错误。5 秒等待、2 窗口断言、其余 expect 调用以及总测试时限均不变。诊断子进程仍有原来的 10 秒上限。
- 快照标签改为 electron-after-window-wait；window-wait-ended 时间戳位于启动 pwsh 之前。单独检查实际源码中的 try/finally，模拟成功和失败，确认调用顺序及原错误不被吞掉。
- CI 完整快照仍写入按 run/attempt 隔离的临时目录并上传产物。仓库现有快照只保留 Electron、Node/NodeJS、Runner.* 进程，以及这些 PID 对应的端口。
- tools/scaffold/filter-evidence.mjs 用于筛选准备入库的副本；不得对 CI 原始产物执行。入库 manifest 标记 filtered derivative，重新计算入库字节的 SHA-256/长度，另保留原始哈希供核对，不冒充原始完整快照。
- 为避免完整快照留在本 PR 的早期提交中，未合并分支重建为一个筛选后的提交并使用明确旧 SHA 的 force-with-lease 推送。PR #4 不变。

验证：lint、typecheck、原桌面 E2E、证据 Verify；新增顺序测试覆盖成功/失败，并检查入库快照白名单及衍生文件哈希。证据见 window-wait-order.log、after-wait-e2e.log。

Runner 任务仍按 RUNNER-PLAN.md：先诊断 PR 的双平台 CI；合并后 main 自动 CI；再对 PR #4 原 1654ee8 的失败 Windows 作业复跑一次。Windows runner 需在这些任务之间保持在线，串行执行；完成后即可停止。任何失败先查证据，不连续盲目重试。PR #4 与下一张卡继续保持原限制。
