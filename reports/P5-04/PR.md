为 P5-04 增加只读事实视图：从已提交历史前缀建立实体、时间、FTS5 trigram、本地 CPU 向量四路索引，用固定 RRF 合并；c_core、c_task、面板分别计预算。正向白名单隔离解释记录，五种非成功状态明确区分，纠错/撤回按引用生效，每次返回重读核验 H 与原始载荷，并产出可留存的查询审计。

当前代码 785a1bb 已 merge origin/main 0b5412e（含 #57、#61、#48）。MEMORY 使用生成 QueryV2/QueryResultV2，growth_entries 恒空；不承接其他服务的查询。相对 main 的 IDL、生成文件、冻结字节零差异。本机与 Ubuntu 验收全通过，手动 Windows 39986857254 在隔离测试临时目录 icacls 完整性标签设置处 Access denied，W02/W19 未开始；保持草稿，等待用户决定最小权限修复方案。

## core-store 扩展（必审）

用户专项授权仅 core/crates/core-store/src/queries.rs、src/tests/reader.rs：追加 Reader::transactions_at 与 TemporaryRoot 测试。重建旧 ACCEPTED SemanticGraph 的实体索引必须取得完整历史前缀，因此通过 Reader 扩展，避免 fact-view 直接打开权威 SQLite。

在固定只读快照中先走现有恢复、谱系头、索引、回执核验，回执在快照前捕获；前缀不完整、未知前缀或预算不足明确失败。Reader<L> 保留打开时的谱系类型。未改存储格式、表结构、写路径或已有接口行为；正常前缀、缺失索引和跨谱系 compile_fail 已通过。P5-03 PR #64 的并行情况已在实现前说明，2026-10-23 核对其 diff 未涉及这两个文件。请求见 tasks/requests/P5-04-core-store-prefix.md。

## core-types 扩展（必审）

用户专项授权仅 src/lib.rs、tests/canonical_boundaries.rs。公开 Schema 可复用，但 Reader::read 没有注入解码器的接口，fact-view 持有 Schema 不能覆盖 Reader 内部重复编译，故使用最小 codec 缓存。OnceLock 只复用内置不可变描述符；键含类型标签、schema_version=1、生成 schema SHA-256。初始化后无可变状态，不缓存对象、原文、权限或核验结果，所有验证、限制、错误行为保留；Schema::builtin 的独立可变实例行为不变。

合并后 445 合法/457 非法黄金向量在冷、热、无缓存下逐项相同，比较结果、规范字节、摘要、String 错误；独立子进程首次/后续公开调用等价、并发和外部 Schema 隔离通过。只修改上述两文件，未修改 #57 改动的文件，已 merge 最新 main 并重跑全部契约检查。

合并前同一固定 FP32 九查询 p50 91.13 → 43.33 ms；合并最新 main 后本轮 p50 44.50 ms、max 45.91 ms，9/9 小于 50 ms。查询、条数、预算、模型、四线程和撤回次数未改。描述符诊断 100 次 WireMessage 编码 60.14 → 1.14 ms。专项授权与证据见 tasks/requests/P5-04-core-types-codec-cache.md。

## 模型与验收

官方 intfloat/multilingual-e5-small，MIT，固定修订 614241f622f53c4eeff9890bdc4f31cfecc418b3，FP32、逐文件 SHA-256、显式 CPU 4 线程；模型/缓存/临时/编译目录均在 D 盘，不用 GPU 或运行时网络。量化候选未过固定用例，已移除运行入口，保留失败证据。重量级依赖登记于提议 ADR 0020，保留 needs-user。

- Rust fmt、全工作区全目标 CPU 特性 clippy -D warnings、完整 release 回归通过；core-store 32/32、fact-view 8/8、谱系 compile_fail 通过。
- SYS05 五种不同状态及成功例、解释记录结构隔离、V2 MEMORY 边界、纠错链通过；真实临时 EXPERIMENT 重启后三语 9/9 向量与合并排名第 1，100 次相关撤回查询 0 次进入 c_task。
- pnpm lint/typecheck、535 项 Node 测试、350 项契约单测通过；契约语句/行/函数覆盖 100%，分支 97.46%。
- 三语言各 10,766 例通过，生成 0 stale、兼容 0 违规，官方 protoc 445 条通过；冻结字节相对 main 零差异。
- Ubuntu 39983818418 全通过；手动 Windows 39986857254 失败于临时目录 icacls Access denied，旧 Windows 39901710241 同样失败；原始日志与 artifact 保留，W02/W19 未运行。方案见 BLOCKED.md、tasks/requests/P5-04-windows-temp-acl.md。更早的量化、损坏夹具外键、验证器 target 路径失败亦保留。

所有测试仅用临时库，未打开排练库或正式库。九事实/九查询只是 P6-13 前的评测骨架，不代表多年规模、完整回答 90% 或任意负载 SLO。详细证据在 reports/P5-04/REPORT.md 与 main-sync-20261024/。

全部机械验收通过后才转为可审查并写「请审查」，由用户安排 Codex 新会话审查 core-store/core-types，之后由用户批准 ADR 与合并。不联系其他执行者，不自动合并。
