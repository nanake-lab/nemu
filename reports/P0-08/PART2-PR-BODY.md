## 续测范围

继续已合并但部分完成的 #29；旧分支不再推送。本 PR 从 origin/main 806e09b 新建 task/P0-08-part2。

- 隔离 CODEX_HOME 的 ChatGPT 登录、模型目录与同版类型确认已完成；没有读取认证文件或登录页面。
- 用户指定 gpt-6-luna / max。初始远程组完成，但工具能力审计暴露 code-mode 工具；旧记录保留为历史对照。
- 严格工具配置已完成三语各 50 条，新的 200 条远程 IRL 正在串行测量；完成后重新按卡片规则选后端。
- 本地选定 Qwen3-4B-Instruct-2507、m=0、NF4/native RMS 的 200 条已完成：Schema 192/200，来源检查 169/200，完成 P50 15647.8648 ms，P90 26168.0977 ms。8 条 malformed 输出和中止 v1 均保留。
- 用户授权的 GPU 安静窗口检查已完成，测量已结束，已通知恢复 runner。
- interrupt、steer、命令审批和文件审批均已实测；客户端拒绝全部审批，没有创建目标文件。
- 兼容 wire schema 与原始 schema 分开，最终输出仍按原始 schema 校验；不修改正式契约或配置。

证据和限制：reports/P0-08/PART2-STATUS.md、PART2-METHODS.md、part2/。

## 当前检查

中间态 pnpm lint/typecheck/test 通过（29 文件 / 443 测试）；Node 7 项、Python CPU 10 项、Ruff 与 strict mypy 通过。完整日志保留；技术测量完成后合并最新 origin/main 并重跑检查。

## 审查状态

当前保持草稿。#29 已有 unverified 标签及说明，第二部分经 Claude 审查后再移除。P0-09 延迟目标确认仍由用户完成；A/B/C 未测到的端到端项保持 null。正式 helpers.json 等继续按 tasks/requests/P0-08-config-scope.md 走 ADR。
