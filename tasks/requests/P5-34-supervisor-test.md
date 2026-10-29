# P5-34 请求：桌面进程监督器退出确认

- 日期：2026-10-26；PR #95；状态：待所属目录维护者处理。
- 请求范围：apps/desktop/src/supervisor/，不在 P5-34 目录，本卡不修改。

在 Windows 上执行完整 pnpm test，579 项通过、1 项失败。supervisor.test.ts:199 的测试在 proc.stop() 后等待既定 100 ms，再通过 process.kill(pid, 0) 检查观测到的子进程；期望 []，实际为 [87292]。随后只读进程检查该 PID 已不存在。原始命令、时间与退出码见 reports/P5-34/verification/resumed-20261026/attempt-01/test.json，完整失败见同名 .log。

本卡与 origin/main 的 supervisor 源码和测试差异为零。尚不能确定根因，不能把后续退出解释为测试已通过，也没有 CI 作业冲突的证据。

期望所属维护者核实停止承诺与 Windows 进程树退出确认是否一致，保留该失败输入并做针对性诊断。若需要修复，只改既有生命周期逻辑并保持零孤儿进程断言，不以删除断言或放宽等待阈值交付。P5-34 不自行扩围、不自行重跑掩盖失败。

## 2026-10-26 用户要求的 main 与 CI 对照

在独立 worktree 固定 main b4eb7fd，以相同 Node 24.21.0/pnpm 12.11.2、原锁文件与 Vitest 配置执行预先确定的三次完整 pnpm test：每次 52 文件、580 项全通过，未复现原退出断言失败。不能写成“main 也失败”或由此证明根因。原失败 head da945d5、当前实现、main 的 supervisor 七文件及 package.json/pnpm-lock.yaml/vitest.config.ts 共十个 Git blob 相同；运行依赖为本地 supervisor 与 Node 内建模块，不引入本卡的 retention IDL 或 Rust 存储逻辑。

Ubuntu CI 的根 pnpm test 会包含这个测试；Windows workflow 跑 transport/isolation/e2e/visual 与 W02，不跑根 Vitest。原失败原样保留，不改 supervisor、断言、100 ms 等待或设置。当前没有发现本卡引入该失败的代码证据，根因仍未定位；后续维护者可独立调查 Windows 退出确认的波动。

证据为 reports/P5-34/verification/protected-20261026/supervisor-comparison.json 与 main-test-04/05/06 的 log/json。此前 main-test-01/02/03 是验证脚本单参数被拆分的调用错误（实际 vitest run e s t，仍跑完 580 项）；保留原日志，不作为正式对照。修正的是报告验证脚本的参数数组，未修改仓库测试或配置。
