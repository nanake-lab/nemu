# P5-22 当前阻塞：Windows CI runner 离线

日期：2026-10-23（Australia/Perth）。A1 代码修复及本机验收已完成；未合并。

## 问题与实测

- 修复提交：fc5f6dab4d53dd673b0906844ecb66e5d231eb0c。
- 首次复核时 Windows run 39910022507 为 queued，目标为该修复提交；GitHub runner API 返回 DESKTOP-8LBF84O status=offline、busy=false。最终文档提交对应的排队作业以 PR 回复中的链接为准。
- 用户明确回复「暂时无法启动」。执行者不操作浏览器/其他应用、不代用户启动 runner 或联系其他执行者。
- 本机 26 项检查均有成功结果（含迁移后重建环境的 Python 复跑）；全仓 481 测试、契约 350 测试、三语言各 10,766 用例通过。旧向量每语言 1,816 查询，0 字节/摘要差异。完整证据见 REVISION-2.md。
- 首次 Ubuntu run 39909990529 与 Windows 均针对上述修复提交；文档提交后 CI 的最终状态在 PR 交付回复记录。没有把 Windows 排队记作通过。

## 已做的处理

已检查 runner 状态、保留现有排队作业并通知用户；PR 恢复草稿，保留 needs-user。未跳过检查、改阈值或自动合并。

## 可选后续

1. 用户可以安排 runner 上线，让已有排队的 Windows 作业执行；全部 CI 与独立审查通过后，才转为可审查并等待用户批准合并。
2. 目前先由用户转交独立 Codex 会话审查代码，PR 保持草稿；runner 可用后补齐 Windows CI。本轮按用户指示在 PR 回复「请审查」后停下。

仅文档的后继提交不会改变 fc5f6da 的受测代码。继续任务时仍须重新 fetch；如 main 有更新则 merge 并按变化重验。
