## 第二部分结果

续测部分完成后合并的 #29；只推送 task/P0-08-part2，旧分支未再推送。已合并最新 origin/main bba68cc（merge b1900b6），P0-08 卡和载体锁定配置未变。

- 远程模型为用户指定 gpt-6-luna / max；最终严格工具配置完整运行三语各 50 条与 200 条 IRL。350 条均无观测到的 raw 工具调用/模型改道。
- IRL 本地完成 P50/P90：15647.8648 / 26168.0977 ms；Schema 192/200，来源检查 169/200。8 条 JSON 解析失败原文保留。
- IRL 远程完成 P50/P90：15924.60 / 23776.16 ms；Schema 200/200，来源检查 195/200。
- 两者都未达到 400 ms。按卡片回退规则选完成 P50 较低的本地，截止时间草稿 26168.09769999236 ms；不是 400 ms 验收通过，也不证明总体性能显著更好。
- interrupt/steer、命令/文件审批实测完成，审批全拒绝且目标文件均未创建。额度开始/100/整组结束快照齐全。
- 登录、载体与 GPU 前置阻塞已解除；GPU 测量已结束并通知恢复 runner。没有读取认证文件或登录页面，没有把凭据放进仓库。
- A/B/C 未测到的端到端 P50/P90 保持 null。正式 helpers/models.lock/部署 profile 均未改，继续按 tasks/requests/P0-08-config-scope.md 走 ADR。

原始组、日文连接失败/补测、解码器 v1 中止和工具暴露审计均保留为历史证据，不混入最终组。数字、绑定与限制见 reports/P0-08/PART2-REPORT.md、PART2-METHODS.md、benchmark.md、decision.md、part2/analysis.json。

## 检查

最新 main 合并后 pnpm lint/typecheck/test 通过（29 文件/443 项），Node 7 项与语法检查通过；Python 10 项、Ruff、strict mypy 4 文件通过。原始日志在 reports/P0-08/part2/checks/final-after-main/ 和 final-python-native/；初次 Python CMD 启动器失败日志保留，原生 PowerShell 补跑没有改实现或测试。post-merge analysis 退出 0，已重校候选、来源、覆盖与源码哈希。CI 以本 PR 最新 checks 为准。

## 审查与剩余门槛

按用户本次明确要求交 Claude 审查，不自行合并。P0-09 用户确认仍为阻塞；#29 的 unverified 在第二部分审查通过前保留。技术测量完成不等于所有用户验收已完成。

最新 main 的 AGENTS.md 第 2、6 节要求部分完成/等用户决定时保持草稿；P0-09 仍待决定，当前因此保持草稿，技术证据可供 Claude 先审。用户若确认按本次原第 7 步先转可审查，再变更状态。
