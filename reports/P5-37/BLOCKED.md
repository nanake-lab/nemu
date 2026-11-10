# P5-37 未解除的验收阻塞

Windows runner 离线，尚未执行本次提交的 Windows 工作流。PR 保持草稿；自动 Windows 门禁明确拒绝把 draft 导致的 skipped 算作通过。已通知用户恢复 runner，未修改工作流或把 PR 转为 ready 来规避门禁。

本机 Windows 的 Rust/NTFS 测试不能替代工作流的桌面、隔离与 gateway 检查。原始状态和日志见 [EVIDENCE-INDEX](EVIDENCE-INDEX.md)。

可选后续：

1. 恢复 runner 在线、确保无其他重负载后，在当前代码分支手动运行 `gh workflow run windows.yml --ref task/P5-37-cognition-idl`，记录实际 head 与完整结果。
2. 本次保持草稿与待验收状态，安排后续验证窗口；独立代码审查可先进行，但不得据此合并或签发 Windows 通过。

独立交叉审查及 ADR 0030 / V2-110–114 的用户批准另见 [报告](REPORT.md)。
