# P5-34 门控与已解决问题（2026-10-26，PR #95）

## B1 已解决：V2 新写入冲突与范围

用户批准的最小 core-store 连锁改动已完成：V3 新写、V2 历史只读、旧 V2 head 明确拒写。完整本地工作区通过（core-store 34/34），三个新增存储边界回归通过；首次夹具构造失败的原日志保留。范围和必然后果见 PROTECTED-SCOPE-20261026.md。没有自动迁移、默认 q/V 或重放回调；未来迁移由 P5-26 按 v2.0 §2.2 从 H 运行 G，不另开迁移卡。

## B2 已核验：supervisor 原失败保留，未发现本卡回归证据

旧 Windows pnpm test 在 supervisor.test.ts:199 观察到 PID 87292，原 579/1 结果不撤销。固定 main b4eb7fd 的三次正确完整运行均 580/580，本分支当前运行也 580/580，因此不能声称“main 同样失败”。源码、测试、配置和锁文件十个 Git blob 在失败 head/main/当前完全一致；该模块只依赖本地 supervisor 与 Node 内建 API，不调用本卡 IDL/Rust 存储。根因未定位，原日志与非复现证据均保留，未改 supervisor、阈值或设置。Ubuntu CI 会跑此测试，Windows desktop workflow 不跑根 Vitest。详情见 REPORT.md 与 supervisor-comparison.json。

## B3 尚待最终门控：同一 head CI、交叉审查与用户合并

本地验收完成后推送；在草稿阶段手动触发 Windows 全量 workflow，并核实 Ubuntu 与 Windows 都属于最终 head、实际执行且全部通过，跳过/取消不算通过。CI 未满足时保持草稿，在本文件记录真实原因并完成其他部分；满足后转可审查并写“请审查”。不得用旧提交的成功日志代替。

core-store 是受保护路径，在 REPORT.md、PROTECTED-SCOPE 与 PR 描述单列，随 ADR 0021、V2-62/63 一起交叉审查。needs-user 保留，由用户合并，不自行合并或领取下一卡。

完整 G 重放、AC2-08/19/21、I23、SYS17 的后续集成未由本卡签发。最新本地证据为 verification/protected-20261026/；旧 PR #72 与本次早期失败完整保留。

## B4 实测环境阻塞：Windows runner 离线

2026-10-26 11:53–11:56 UTC 核验，GitHub runner ID 42 / DESKTOP-8LBF84O 为 offline、busy=false；本机也无 Runner.Listener/Runner.Worker。head 7cb285a14bcc904abd18f2cd9ffc64947302cb66 已推送，手动 Windows 全量 run [40318429511](https://github.com/nanake-lab/nemu/actions/runs/40318429511) 已成功触发，full=true，job 121556830260 queued，尚未分配到 runner。

同 head 的草稿 PR 事件 run 40318254835 按现有守卫跳过 self-hosted job，汇总 gate failure；这不是执行了测试，也不算通过。Ubuntu run 40318254909 继续执行。原始检查快照见 verification/protected-20261026/ci-7cb285a-initial.json。

已完成所有不依赖该 runner 的本地验收、报告与推送，并通知用户相关 PR/head；未启动/更改 runner 服务或系统设置。选项：恢复现有 runner 后由已排队的 full=true 检查执行；若当前有用户安排的独占资源窗口，则保持排队/草稿，窗口结束再恢复。两种方式都要求实际结果通过，不能通过跳过、缩小范围或改变超时/测试阈值替代。


后续核验：同源码 head 7cb285a 的 Ubuntu run 40318254909 已 completed/success，含 core-store 34 项（604.08 秒）和本卡三语言契约。完整下载日志与 SHA-256 元数据已保存；.NET/status 按路径未涉及而跳过，不算执行过。2026-10-26 12:14 UTC 再次查询 runner 仍 offline。报告/证据补充提交不改变任何源码；当前 head 的最终门控以 PR 最新检查为准，恢复 runner 后需确保排队的手动运行指向该 head。
