为 P5-04 增加基于已提交历史前缀的事实视图。只经 core-store Reader 读取原始经历，实体/时间/FTS5 trigram/本地 CPU 向量四路召回，用固定 RRF 合并；c_core、c_task、面板各自计预算。

正向类型白名单阻止解释记录进入 c_core/c_task。五种非成功状态分别返回；纠错、撤回与旧索引失效确定性处理，memory 隐藏所有不能进入 c_task 的内容。实际查询、模型、向量与返回生成 AuditExtension，供外层事务留审计。IPC 只用 main 的生成类型，未改 IDL。

## core-store 扩展（必审）

用户在本会话明确授权：只在 core/crates/core-store/src/queries.rs 和 src/tests/reader.rs 追加 Reader::transactions_at 及临时库测试。必须在固定只读快照里先做现有恢复核验、回执核验和谱系检查，前缀不完整即失败；不改表结构、存储格式、写路径或已有接口行为。用于从所有历史事务的 ACCEPTED SemanticGraph 重建实体召回，避免 fact-view 绕过 core-store 直接开权威 SQLite。

P5-03 为 PR #64；本扩展已在实现前于此说明。2026-10-23 核对 #64 diff 没有修改上述两个文件，只与 Cargo.lock、Windows CI 等共享文件相交；若其修改这两个文件同一区域则暂停冲突部分。

## core-types 扩展（用户专项授权，必审）

只改 src/lib.rs、tests/canonical_boundaries.rs。WireMessage 的四个默认 codec 方法用 OnceLock 复用编译内置的不可变描述符；缓存键包括类型标签、schema_version=1 和生成 schema.json 的 SHA-256。继续执行全部 encode/decode/normalize、原文核验及限制检查；不缓存对象或核验结果。Schema::builtin 仍返回独立可变实例，公开接口行为不变。fact-view 能持有公开 Schema，但 Reader 内部没有注入该解码器的接口，所以上层局部持有不能覆盖这些重复构建。

396 条合法、409 条非法黄金向量冷/热/无缓存的结果、规范字节、摘要、Err(String) 完全相同；新增独立进程首次/后续调用等价、并发与外部 Schema 修改隔离测试。固定 FP32 完整查询中位 91.13 → 42.16 ms、最大 46.54 ms，召回与 0/100 撤回结果不变。只缓存描述符使同一读取/核验路径满足原目标，没有缩减返回或预算。

用户授权了与 #57 同目录不同文件的例外；这两个文件未与 #57 当前 diff 相交。IDL、生成文件、黄金字节、contract.rs 和存储均不改；后合并者 merge main 后重跑全部契约检查。授权与证据见 tasks/requests/P5-04-core-types-codec-cache.md。

## 模型与当前验收

官方 intfloat/multilingual-e5-small，MIT，固定修订 614241f622f53c4eeff9890bdc4f31cfecc418b3；使用 FP32，逐文件 SHA-256 登记，模型/缓存/编译/临时目录均在 D 盘，显式 CPU provider。量化候选未过固定用例，已移除运行入口，保留失败证据与来源清单。新增重量级依赖见提议 ADR 0020，保留 needs-user。

真实 FP32 release：临时 EXPERIMENT 存储重启后，三语 9/9 正确事实向量与合并排名第 1，100 次相关撤回查询 0 次进入 c_task。缓存优化后本轮 9 个完整查询均小于 50 ms；这是 P6-13 前的固定小规模评测骨架，不代表多年规模/完整回答验收或任意负载的 SLO。

修复后的完整回归已通过，core-store 32/32；首次夹具损坏注入违反外键的失败日志保留。最终代码的完整回归、契约检查和 CI 正在重跑。所有测试只用 TemporaryRoot，不开排练库或正式库。证据与细节见 reports/P5-04/REPORT.md。

全部验收完成前保持草稿。#57 尚未合并，合并后再 merge main 对齐 user_view。完成后按用户要求在 PR 写审查请求，由用户安排 Codex 新会话；不联系其他执行者，不自动合并。
