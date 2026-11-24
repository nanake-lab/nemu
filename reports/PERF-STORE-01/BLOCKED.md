# PERF-STORE-01 基线测量阻塞

**WSL 启动阻塞已解除。** 2026-11-23 本机 Ubuntu 曾在创建虚拟机时失败，退出码 1，错误 `Wsl/Service/CreateInstance/CreateVm/HCS/0x800705aa`。经[审查决定](https://github.com/nanake-lab/nemu/pull/265#issuecomment-7106277996)，Ubuntu 在 GitHub 托管机计时，函数剖析在 Windows 进行。续做不需要修复 WSL。

| 基线 | 本次实况 | 可继续的条件 |
| --- | --- | --- |
| Windows / NTFS | 2026-11-24 开工检查时，缓存父目录可枚举、GPU 窗口锁不存在，发现 1 个 `Runner.Worker`。测试、编译与剖析均为 `not_run`。 | 另次派发时重新核对窗口锁、runner 空闲及其他重负载，再测基线并剖析最慢五项。 |
| Ubuntu / GitHub 托管机 | 默认分支为 `main`，核对的 `origin/main`（`4d5081e5`）尚无本工作流。手动调度与计时为 `not_run`。 | 由维护者安排工作流进入默认分支后再手动运行。首次可只测基线，有候选提交后在同一作业复测两版。见 [GitHub 手动运行条件](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)。 |

正式对照和剩余验收见[本卡报告](REPORT.md)。当前仅请求审查测量准备。
