# P5-34 当前待完成项（2026-10-26，PR #95）

## B1：最小存储适配的最终验证

原 full workspace 在 EvidenceGroupV2 新写入处失败，原始证据保留于 verification/resumed-20261026/attempt-01/workspace.log，Ubuntu 40271343235/40274213446 也复现。用户已批准最小 core-store 连锁改动及分阶段方案，详细范围与原因见 PROTECTED-SCOPE-20261026.md。当前实现改为 V3 新写、V2 只读、旧 V2 head 拒写；不恢复旧 PR 的迁移回调。

待执行新边界测试与完整工作区验收。当前其他任务占用本机 Rust 重负载窗口，先完成轻量静态检查及文档，不并行启动重负载。处理方式：窗口空闲后按 verify-protected.ps1 串行执行；若出现实现失败，保留日志并在授权范围内修复。完整 G/H 迁移不再是本卡的待授权项，未来由 P5-26 完成，不另开迁移卡。

## B2：supervisor 的 main 对照

旧 pnpm test 为 579 通过/1 失败，apps/desktop/src/supervisor/supervisor.test.ts:199 在 hung-process 退出清理断言观察到 PID 87292；原日志保留。随后 PID 已消失，但不把原失败改判成功。

supervisor 七个文件、根 Vitest 配置、package.json 与 pnpm-lock.yaml 在本分支和 main b4eb7fd 的 Git blob 相同。Ubuntu CI 的根 pnpm test 包含 supervisor；Windows desktop workflow 不跑根 Vitest（跑 transport/isolation/e2e/visual 和 storage）。还需在独立 main worktree 执行预先声明的三次完整 pnpm test，保留全部结果，以区别本卡回归与既有波动。测试/设置/阈值不改。

若 main 复现，则按用户指示原样保留、记录证据；若证实本卡引起，则写明原因并向用户报告。不用后续成功覆盖旧失败。

## B3：最终 head CI 与交叉审查

完成本地验收、报告与推送后，检查同一最终 head 的 Ubuntu 和手动 Windows 全量 CI；草稿条件跳过不算通过。全部 CI 确认通过后才转为可审查并写“请审查”。core-store 受保护路径在报告和 PR 描述单列，随契约交叉审查；ADR 0021、V2-62/63 仍由用户最终合并批准，保持 needs-user。

完整 G 重放、AC2-08/19/21、I23、SYS17 不由本卡的结构/存储测试签发。新证据统一放 verification/protected-20261026/；旧 PR #72 与本轮此前失败全部保留。
