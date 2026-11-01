# 首次审查修复提交的 CI

源码与本地完整证据随修复一次提交；发布后在 PR 描述和最终「请审查」回复固定新 head、实际 checkout、main 父提交、run URL 及结果，不为仅回填 CI 结果再次提交触发测试。

- 当前 [PR #108 Checks](https://github.com/nanake-lab/nemu/pull/108/checks)：Ubuntu 完整 Rust/契约检查；Windows NTFS 存储、桌面隔离及上传汇总门禁。
- Ubuntu 新增 `P5-36 bounded writer lock lifecycle evidence (Linux)` 步骤：只运行三个短锁用例并输出 `/proc/locks`/fdinfo 和 128 次并发结果；不重跑整个 workspace，不安装新工具。
- 主报告不会把旧 head 的 CI、草稿 skip、cancel 或失败当作新提交通过。新失败必须原样保留并诊断，不增加无限重试。
- CI 完成后的原始日志、结果和 checkout 核对追加保存在本目录的 `ci/`，GitHub run 为远端可读原始证据。历史首次 Ubuntu 失败仍可从 `../storage/ci/` 和原 PR run 查阅。

本轮本地基线 B=`f24f1256259c998bc213a9dc93103fcfb9cfe89a`，发布前再次 fetch 的 M 相同。提交后/收尾前继续核对 M；源文件摘要见 `source-final-manifest.json`。
