# P5-04 Windows 权限阻塞

2026-10-24：代码 785a1bb 已合并 origin/main 0b5412e。#57 MEMORY 已对齐 QueryV2/QueryResultV2，本轮本机检查及 Ubuntu 39983818418 全通过；真实 CPU 三语 9/9 第 1、撤回 0/100、九查询 p50 44.50 ms / max 45.91 ms。全部本地库仅 TemporaryRoot。

自动等空闲并完成本机检查后，手动 Windows 39986857254 失败于 tools/scaffold/test-windows-isolation.ps1:7：D:\Dev\cache\tmp\nemu-isolation-* 执行 icacls /setintegritylevel 返回 Access denied；W02/W19 未开始。旧运行 39901710241 同样失败；原始日志和 artifact 保留于 main-sync-20261024/。D:\Dev\cache\tmp 的只读 ACL 显示当前用户没有显式 FullControl，尚未改权限或重试替代方案。

AGENTS.md 第 3 节第 13 条要求失败后列选项、等待用户决定。方案详见 tasks/requests/P5-04-windows-temp-acl.md：

1. 允许建立任务独占 D:\Dev\cache\tmp\P5-04-ci，仅给 runner 当前用户该子目录及后代 FullControl，并把本分支 Windows TEMP/TMP 指向它，再跑同一隔离检查和完整 Windows CI。保持 SYSTEM/Administrators 权限，保留现有检查和阈值；需一处 CI 配置与该私有目录 ACL 变更。
2. 保留草稿，用户修复 runner 的 D 盘临时目录权限后通知，再重跑；代理不改 ACL，等待时间由环境修复决定。

PR #65 保持 draft + needs-user，未请求审查。机械验收通过后才交给用户安排的新 Codex 会话审查 core-store/core-types；不合并、不取下一卡。
