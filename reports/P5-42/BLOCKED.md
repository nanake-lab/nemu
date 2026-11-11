# P5-42 未完成项

1. **ADR / 契约批准与交叉审查**：ADR 0036、V2-151 为待用户确认；PR #164 保持 draft / needs-user。由用户安排独立高难度交叉审查，再由用户合并；执行者不自行批准。
2. **Windows CI**：当前分支的 `tools/scaffold/windows-evidence.ps1:34` 枚举全机进程，现行 Windows workflow 会上传该快照，违反 AGENTS §6 的隐私要求。未触发该工作流，也未转为 ready。修复 PR #157 尚未合并。

解除 Windows 阻塞的路径：等 #157 合并后，merge main 并核对工作流，再运行本卡所需 Windows 检查；或由负责该工作流的任务提供符合 §6 的替代验证方式。本卡不修改卡外 CI。已有本机 / Ubuntu 检查不能代替缺失的 Windows CI。

真实载体、资格报告、genesis 谱系迁移、生产发布入口不在本卡范围，详见规格对照中的 not_run；不因此把本卡的契约检查标成生产验收。
