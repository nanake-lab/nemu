档位：重档；修改核心数值读出与权威存储的派生一致性边界。

# FIX-STORE-01 — 数值与派生一致性修复（交付线）

实现与本机验收完成，等待交叉审查及当前提交 CI。Windows runner 离线，PR 保持草稿，合并受阻，见 [BLOCKED.md](BLOCKED.md)。账本按权威事务的追加序返回；派生表与权威记录不一致时拒绝恢复。空存储初始化中断留下的缺失 reject 目录可以补建并同步；有历史的存储缺目录时明确拒绝。只用临时合成存储验证，完成后请求审查并停止。

## 变更与依据

- `core-kernel/law/recurrence.rs`：E/C 改为先除二再加减，防止中间溢出。差异登记 [V2-178](../../docs/traceability/diff-register.md)，[规格对照](../../docs/traceability/FIX-STORE-01.md)。正常范围逐位比较，极大值有限性及次正规边界分别验证。
- `core-store/derived.rs`：从 D 的规范事件和已提交事务重建内存投影，与存量表逐行、含重复行比较。覆盖 object_refs、visible_objects、evidence_groups、evidence_index、snapshot_objects、accepted_results、runtime_records、outbox、dependency_index、fact_corrections，以及既有 facts、snapshots、ledger_index、ingress_decisions、branch_head。不把派生索引作为重建输入，不写回真实数据库。旧 R/EvidenceGroup V2 只读投影保留。
- `store.rs`：GC 先完整核验，再由权威根的闭包取得保留集合，合并 prepare pin 保护；不以存量引用表决定删除。
- `queries.rs`：ledger_at 按提交序读取规范 staged_ledger，保持事务内追加序及完整结果预算错误。
- `witness.rs`、`reader.rs`：format-2 恢复准入要求三个回执目录；仅无 D、commit、拒绝审计和引用，且既存回执为空的初始化残留允许 writer 补 reject 并同步。其他缺失明确提示保留文件及显式恢复。Reader 不负责修复。format-1 继续允许缺 reject，writer 仍需既有显式迁移。
- `tests/consistency.rs`：四项审计诊断成为仓库回归，另加全表重建比较及非空存储缺回执目录拒绝。

## 兼容与范围

无 schema、H 格式、IDL、黄金向量或运行时依赖变化，无需迁移数据。已有完整存储仍沿现有入口读取；原先静默接受的损坏索引现在报错，本卡不新增磁盘索引修复命令。`scope_index` 当前仅 schema 定义，无写入或生产消费者，留待其接线时实现重建与核对；其他审计列举表均纳入检查。

新增机制三问：不能省略权威与派生的核对，否则损坏仍会静默传播；一个人核对账本会以原始记录为依据；内存投影不持久化、不新增第二权威或不可撤销格式。复用现有发布投影逻辑，避免另写一套派生语义。

## 验证与失败

依据 [PR #193 审计及通过的复核](https://github.com/nanake-lab/nemu/pull/193)。验证源提交 `b5f5d58f03d5c7907e7fb00549e3ed542dc23f9a`，main 基线 `ce115abe6ced3ec8cc301bea2a290f47ace4807a`；收尾只改报告。原始日志、哈希和私有仓提交见 [证据索引](EVIDENCE.md)。

| 检查（Windows / Rust 1.99.0，core/ 下） | 结果 |
| --- | --- |
| `cargo +1.99.0 fmt --all -- --check` | exit 0 |
| `cargo +1.99.0 clippy --workspace --all-targets --offline --locked -- -D warnings` | exit 0，无警告 |
| `cargo +1.99.0 test --workspace --offline --locked` | exit 0；顶层汇总 476 passed、0 failed、1 既有 ignored（排除子进程重复摘要） |
| core-kernel | 274 passed；1 项原有 V2-93 Python 舍入诊断保持 ignored；290.25 s |
| core-store | 58 passed，含 7 项新增回归与原有全部测试；533.78 s；另 3 项文档测试通过 |
| fact-view | 18 passed；模块总耗时 378.46 s，不是单次查询延迟 |
| 文档与差异 | `git diff --check`、新增 Markdown 相对链接存在性检查通过 |
| 远端 CI | 最终提交推送后执行；Windows runner 离线，当前 Windows CI 未取得通过，不以本机结果替代 |

首轮存储回归 5 项失败：内存投影库默认开启外键，缺少故意未复制的权威表而拒绝插入。已显式关闭这一个一次性内存连接的外键，产品存储外键保持开启；原始失败保留。一次中间版本串行工作区运行因实现继续更新而主动中止（exit 0xffffffff），不是完整通过；旧提交 Ubuntu CI 同样取消。最终固定实现的全工作区测试从头完成，结果如上。

## 限制

正常数值范围的逐位相同不外推到整个 binary64 域：V=A=2^-1074 的新 E 为 0，旧 E 为 2^-1074。当前该辅助函数只在测试中调用，不改变 G 的 A/V 更新。未运行真实存储、模型、GPU、物理断电测试。恢复重建的成本随历史增长，现有崩溃和并发测试会覆盖正确性，但不宣称已完成大历史性能验收。

## 待审查

- V2-178 运算次序修正及上述兼容边界。
- 当前状态为草稿；请审查实现与证据。Windows runner 启动并取得当前提交 CI 通过前不可合并；本任务不自动合并。

## 代码组织

`store.rs` 为 404 行，保留既有存储生命周期、类型入口及通用 SQL 辅助函数的组织，本次只追加准入/GC 检查，不为四行超限拆出无独立职责的模块。新增派生重建与核对集中在独立的 `derived.rs`。
