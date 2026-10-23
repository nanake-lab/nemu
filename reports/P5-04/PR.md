认领 P5-04，基于 origin/main 的 P5-02 core-store 实现事实视图。

## core-store 扩展（必审）

用户在本会话明确授权：只在 core/crates/core-store/src/queries.rs 和 src/tests/reader.rs 追加 Reader::transactions_at 及临时库测试。必须在固定只读快照里先做现有恢复核验、回执核验和谱系检查，前缀不完整即失败；不改表结构、存储格式、写路径或已有接口行为。用于从所有历史事务的 ACCEPTED SemanticGraph 重建实体召回，避免 fact-view 绕过 core-store 直接开权威 SQLite。

P5-03 为 PR #64；本扩展在此先行说明。只追加独立方法与测试，不改现有入口方法；若其修改这两个文件同一区域则暂停冲突部分。

全部验收完成前保持草稿。PR #57 合并后 merge 最新 main 对齐 user_view 消息。完成后在 PR 写「请审查」，由用户安排 Codex 新会话；不自动合并。
