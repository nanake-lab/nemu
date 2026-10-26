# P5-34 请求：桌面进程监督器退出确认

- 日期：2026-10-26；PR #95；状态：待所属目录维护者处理。
- 请求范围：apps/desktop/src/supervisor/，不在 P5-34 目录，本卡不修改。

在 Windows 上执行完整 pnpm test，579 项通过、1 项失败。supervisor.test.ts:199 的测试在 proc.stop() 后等待既定 100 ms，再通过 process.kill(pid, 0) 检查观测到的子进程；期望 []，实际为 [87292]。随后只读进程检查该 PID 已不存在。原始命令、时间与退出码见 reports/P5-34/verification/resumed-20261026/attempt-01/test.json，完整失败见同名 .log。

本卡与 origin/main 的 supervisor 源码和测试差异为零。尚不能确定根因，不能把后续退出解释为测试已通过，也没有 CI 作业冲突的证据。

期望所属维护者核实停止承诺与 Windows 进程树退出确认是否一致，保留该失败输入并做针对性诊断。若需要修复，只改既有生命周期逻辑并保持零孤儿进程断言，不以删除断言或放宽等待阈值交付。P5-34 不自行扩围、不自行重跑掩盖失败。
