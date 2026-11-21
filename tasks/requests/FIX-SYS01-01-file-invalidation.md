# FIX-SYS01-01 对象与见证文件失效检测范围请求

PR #240 要求在旧前缀的文件发生改写时回退完整核验。Windows 合成库实测中，翻转第一笔快照文件的一个字节后，写连接的 `data_version`、第二笔提交的父链与所有 SQL 派生表仍通过增量核对，完整核验拒绝同一库。[反例与备选方案](../../reports/FIX-SYS01-01/BLOCKED.md)

请求在本 PR 的后续修复中允许修改 `core/crates/core-store/src/platform/`、`objects.rs` 和 `witness.rs`，并允许在 `store.rs`、`recovery.rs`、`derived.rs`、`terminal_ingress.rs` 共享同一套完整与增量检查。目标接口应能针对本次 Store 打开，区分「已核验前缀仍有效」与「须全量重查」，把文件变更、信号不可用和丢失作为回退条件。先用临时库验证可观测范围与代价，再选择实现。涉及存储格式的方案另按 SYS17 处理。

审查意见已覆盖 progress、uploads 和 core-runtime 的读取入口。此请求针对证明文件前缀是否仍有效所需的存储层范围。
