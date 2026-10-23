# P0-08 第二部分实测

从 origin/main 806e09b 开始，接续已合并但部分完成的 PR #29。

- 重新测量模型目录、150 次语音、200 次远程 IRL、控制与审批及额度。
- 本地使用 P0-07 / ADR 0012 选定的 Qwen3-4B-Instruct-2507，同配置 m=0，200 条受约束 IRL。
- GPU 测量须用户安排安静窗口，先核对 Runner.Listener、Runner.Worker、Blender 和 P4-01/P4-02 测量。
- helpers 与正式配置遵循 tasks/requests/P0-08-config-scope.md，仅更新候选。
- 完成前保持 draft；完成后 merge 最新 main、重跑检查、等待 Claude 审查。
- #29 的 unverified 标签在第二部分经 Claude 审查后再移除。
