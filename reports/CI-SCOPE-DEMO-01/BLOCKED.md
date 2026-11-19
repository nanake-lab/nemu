# CI-SCOPE-DEMO-01 的 Windows runner 连接记录

GitHub Actions 起初显示 Windows runner 离线。启动现有监听进程后，[首次手动全量](https://github.com/nanake-lab/nemu/actions/runs/44452750228) 运行到 W02 存储测试。GitHub 报告 runner 与服务端失联并判定作业失败。

runner 随后重新上线。以同一套检查重跑当前分支，结果见 [该分支的 Windows 工作流列表](https://github.com/nanake-lab/nemu/actions/workflows/windows.yml?query=branch%3Atask%2FCI-SCOPE-DEMO-01) 和 [PR 检查页](https://github.com/nanake-lab/nemu/pull/230/checks)。
