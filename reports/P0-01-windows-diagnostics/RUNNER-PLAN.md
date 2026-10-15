# Windows runner 运行顺序

请保持 runner 与本地 Electron 测试互斥。此次本地测试已结束；代理不会自行启动或重新注册 runner。

1. 首先运行独立诊断 PR 的 Windows desktop and isolation 作业；对应提交以本 PR 头 SHA 为准（通知用户时提供完整 SHA）。Ubuntu 同时由托管 runner 检查。
2. 两个平台均通过后代理复核并 squash 合并诊断 PR。合并 main 的 push 会自动产生另一套 CI；其 Windows 作业运行合并生成的 SHA，当时核对记录。该 SHA 在 squash 合并前尚不存在，不能预填。
3. 诊断分支合并后，代理对历史 run 38584387442 的失败 Windows 作业重新执行一次。PR #4 头必须仍是 1654ee85a00f36396658bba85f3d68f8adecbee0；历史实际 checkout 为 82c65bb162f6fbceab327aec99386c319bb2b0b4。此为间歇性验证，仍使用旧工作流/旧证据路径，不能把它称为新诊断覆盖的测试。只重跑该失败 Windows 作业，不重新执行已绿的 Ubuntu。

以上按顺序进行，Windows runner 串行执行。当前尚未触发第 3 项。若失败则保留证据，不连续盲目重试。PR #4 即使历史复跑转绿，仍须满足 AGENTS.md 的最新 main、更新后测试及 CI 条件；后续更新会产生新 SHA，届时需另行明确运行提交。本次不启动下一张任务卡。
