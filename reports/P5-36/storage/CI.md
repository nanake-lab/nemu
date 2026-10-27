# P5-36 当前提交 CI 证据入口

- [PR #108 Checks](https://github.com/nanake-lab/nemu/pull/108/checks)：核对当前 head，Ubuntu static checks and unit tests、Windows self-hosted checks 及 Windows validation gate；skipped/cancelled 不算通过。
- [Windows 工作流](https://github.com/nanake-lab/nemu/actions/workflows/windows.yml)：包含 W02 NTFS 存储检查；完整 PR 的生成 TS 路径也会触发既定桌面检查。工作流不作修改。
- [CI 工作流](https://github.com/nanake-lab/nemu/actions/workflows/ci.yml)：完整 Rust 工作区/存储、可选 CPU fact-view 编译，以及整个 PR 影响到的三语言契约与 Node/Python 检查。

实现及本地完整验收的源文件摘要见 `source-manifest.json`，实际 main 基线 B 与发布前 M 见 `main-final-check.json`（均为 f24f1256，无增量）。本地完整 Rust 日志与 65 个真实存储返回状态已随实现提交。

按用户“不为报告更新反复提交触发 CI”的要求，在同一实现提交上完成必要 CI 后，将准确 head、run URL、结果和“请审查”写入 PR 描述/最终交付评论；不另提交仅记录 CI 状态的提交。报告及此入口不把旧提交或草稿阶段跳过的 Windows 检查冒充当前通过。首次独立完整审查由用户另开会话进行，本会话不合并。
