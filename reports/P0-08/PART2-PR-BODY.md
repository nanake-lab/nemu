## 续测范围

继续已合并但部分完成的 #29；旧分支不再推送。本 PR 从 origin/main 806e09b 新建 task/P0-08-part2。

- 隔离 CODEX_HOME 的 ChatGPT 登录已完成，授权后仓库未出现凭据；没有读取认证文件或登录页面。
- 已取得模型目录，按用户新指示使用日常档 gpt-6-luna / max。
- 三语各 50 条短回答已测；日文原连接出现 9 个失败槽位，原记录保留，补测单列。见 reports/P0-08/PART2-STATUS.md 和 part2/remote-speech-all-summary.json。
- 200 条远程 IRL 正在测量；控制与审批探针待测。
- 本地 m=0 受约束 IRL 脚本与载体已准备。用户已安排远程完成后的 GPU 安静窗口；实际开始前检查 runner、Blender、P4-01/P4-02 进程。
- 第一部分 schema 的 oneOf 被严格接口拒绝；兼容 wire schema 和原始 schema 分开，输出仍按原始 schema 校验，不修改正式契约。

## 当前检查

中间态 pnpm lint、pnpm typecheck 退出 0；pnpm test 29 文件 / 443 测试通过。JS schema 4 项、Python CPU 3 项测试通过，Ruff 与 strict mypy 通过。完整日志保留在 reports/P0-08/part2/checks/；完成后合并最新 origin/main，再重跑检查。

## 审查状态

保持草稿直至第二部分技术测量完成。#29 已补 unverified 标签和说明，第二部分经 Claude 审查后再移除。P0-09 延迟目标确认仍由用户完成。helpers.json 等正式配置不在本卡修改范围；继续按 tasks/requests/P0-08-config-scope.md 走 ADR。
