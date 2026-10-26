# P5-04 事实视图（W19）— 报告

## 当前状态

本卡本机验收完成，等待交叉审查与用户合并（ADR 0020）。2026-10-26 已解除暂停，分支以 merge 合入 `origin/main@86779b1`（合并提交 `7ec2502`）；本轮只继续事实视图目录及追加文档/请求/报告。Windows 工作流与 main 完全一致，旧 TEMP/TMP 权限阻塞已解除。按用户要求止于推送、转可审查并写「请审查」，不等待或报告 CI，不合并、不取下一卡。

## 交付物

- `core/crates/fact-view/`：已提交前缀、实体/时间/FTS5 trigram/本地多语言向量四路召回、固定 RRF、用途与预算隔离、纠错/撤回、原始对象核验及查询审计。
- V2-88：c_task/MEMORY 可读六类 ACCEPTED 历史解释，显式标记「这是她当时的反应」。保留原始 ObjectRef、事件 ID、原文语义、精确纳秒及补解释的独立记录时刻；不把 ProposalBinding 的状态引用放进模型文本。c_core 四路都排除反应，FTS 语料与 embedding 输入独立，不构造任务展示数据。
- V2-76：`FactQuery.after`、`Execution.next_page`、`user_view_page` 提供无会话续页，绑定查询/前缀/权限/预算/模型；记忆召回取消每路前 64 的总量截断，仍限制每次条数、字节与步骤。75 条同一精确纳秒的记录经 11 页完整返回，无重漏；重建后可继续。
- NO_MATCH、DENIED、INDEX_LAGGING 及 OBJECT_MISSING、BUDGET_EXHAUSTED 分别返回。撤回连同关联反应过滤；补解释同时检查原事件和记录事件的来源权限；历史有新提交时旧索引/续页明确滞后。
- 查询/索引版本为 `w19-rrf-v2` / `w19-trigram-vec-v2`，RRF k=60、通道和稳定平局规则固定；c_core 保留首轮每路前 64。反应向量复用被解释经历的向量，自身语义字段仍可全文检索。审计扩展数据版本为 2，记录标记、实际结果与续页；权威 IDL 和既有字节不变。
- Windows 工作流 blob 与 main 均为 `65b29f2a8a73a7bb08c44492646f3952d8a17435`；本机 env.ps1 也不再覆盖 TEMP/TMP。临时目录继承当前进程配置，未授权或修复任何共享目录 ACL、用户或系统设置。

## 验收

精确命令、工作目录、起止时间、退出码与原始输出均在 [resume-20261026](resume-20261026/) 的同名 JSON/日志中。全部 Rust 检查使用锁定的 Rust 1.99.0；重负载串行。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --manifest-path core/Cargo.toml --all -- --check` | 通过 | [fmt-final.json](resume-20261026/fmt-final.json) |
| `cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets --features fact-view/cpu-embedding --locked -- -D warnings` | 通过 | [clippy-final.log](resume-20261026/clippy-final.log) |
| `cargo test --manifest-path core/Cargo.toml --workspace --release --features fact-view/cpu-embedding --locked -- --test-threads=1` | 通过；core-store 32 项、fact-view 15 项及谱系 compile_fail 通过 | [workspace-tests-final.log](resume-20261026/workspace-tests-final.log) |
| SYS05；V2-76、V2-88；100 次撤回 | 五种状态、成功对照、75 条/11 页、原文与精确时间、标记、权限/续页绑定、对象缺失清空、c_core 排名/步骤独立、撤回传播通过 | [SYS05 结果](SYS05/result-resume-20261026.json)；上述工作区日志 |
| `cargo test --manifest-path core/Cargo.toml -p fact-view --release --features cpu-embedding real_cpu_multilingual --locked -- --ignored --nocapture --test-threads=1` | 真实临时 EXPERIMENT 库重启；三语 9/9 向量与 RRF 排名第 1；100 次撤回零泄漏 | [cpu-model-core-projection.log](resume-20261026/cpu-model-core-projection.log)、[cpu-result.json](resume-20261026/cpu-result.json) |
| ADR 0012 第一轮 50 ms 目标 | c_core 九查询 p50 **36.52 ms**、max **38.40 ms**；9/9 低于 50 ms。含历史反应的完整 c_task 页另计：p50 **104.54 ms**、max **109.27 ms** | 同上；仅合成骨架观测，不是任意规模 SLO |
| 原 Windows 隔离脚本 | 通过：Low 令牌、Medium 目录拒写、正向写入对照；只操作它新建并清理的测试子目录 | [windows-isolation.log](resume-20261026/windows-isolation.log) |
| `pnpm lint`、`pnpm typecheck` | 通过 | [lint.log](resume-20261026/lint.log)、[typecheck.log](resume-20261026/typecheck.log) |
| `pnpm test` | 首次 579 通过/1 失败；main 独立基线及本分支原命令复测均 580/580 通过，失败保留 | [首次](resume-20261026/node-tests.log)、[main](resume-20261026/main-node-tests.log)、[复测](resume-20261026/node-tests-recheck.log) |
| `pnpm test:contracts` | 354 通过；语句/行/函数 100%，分支 97.46% | [contracts.log](resume-20261026/contracts.log)、[coverage-summary.json](resume-20261026/coverage-summary.json) |
| `pnpm test:core-contracts` | Rust/Python/TS 各 10,766 例；生成 0 stale、兼容 0 违规；protoc 445 条通过 | [core-contracts.log](resume-20261026/core-contracts.log) |
| 冻结 IDL/生成物相对 main | 0 差异；本轮未再修改 core-store/core-types | [verification.json](resume-20261026/verification.json) |

工作区默认保留的核心数值诊断、CPU 描述符诊断与真实模型显式测试仍按其原有 ignore 分类；真实模型项已另行按原命令执行，未删除或跳过断言。完整多年规模与回答正确率验收仍属于 P6-13；未用结构夹具冒充业务结论。

## 失败与中止证据

- [cpu-model.log](resume-20261026/cpu-model.log)：直接 embedding 结构化反应元数据导致英文正向用例的原事实排名为 10，原断言失败。改为来源经历向量召回后原断言通过；问题、事实、模型、预算、排名要求和 100 次撤回均未放宽。
- [node-tests.log](resume-20261026/node-tests.log)：`supervisor.test.ts:182` 观察到 2 个活进程，要求至多 1。该测试、相关实现、Vitest 配置及锁文件与 main 零差异；main@86779b1 的独立完整运行及本分支原命令复测通过，未在 main 复现这次失败，未修改测试、阈值、supervisor 或设置。保留此波动，不据一次复测声称根因已修复。
- [fact-view-first.log](resume-20261026/fact-view-first.log) 与 [fact-view-final.log](resume-20261026/fact-view-final.log) 为主动中止，退出码为 -1：分别因漏看已运行的另一任务而恢复串行窗口、以及消除合成夹具反复完整恢复的搭建开销。中止原因单独记录；75 条、11 页和断言保持，后续完整运行通过。
- 旧量化候选、夹具外键、验证器路径、两次 Windows CI 权限失败及 artifact 仍在原目录，未删改为通过。旧 ACL 请求的候选操作已作废；本轮不改共享权限。中间测量同样保留：`cpu-model-final.log` 的 c_core p50 52.27 ms 不覆盖最终结果，`cpu-model-core-projection.log` 对应去除无关任务展示开销后的实现。

## 受保护路径（交叉审查一并审）

本轮没有新增这些路径的改动；以下为 PR 先前用户专项授权的既有扩展，原请求与证据保留：

- `core-store/src/queries.rs`、`src/tests/reader.rs`：只读 `Reader::transactions_at`，固定快照中核验恢复、谱系、索引和先捕获的回执，返回完整已知前缀；未知/不完整/预算不足明确失败。不改格式、表结构或写路径。见 `tasks/requests/P5-04-core-store-prefix.md`。
- `core-types/src/lib.rs`、`tests/canonical_boundaries.rs`：仅内置不可变 codec 描述符缓存，键绑定类型标签、schema 版本和生成 SHA-256；不缓存对象或核验结果，保留独立可变 Schema。冷/热/无缓存黄金向量等价、并发和独立进程测试保留，本轮完整契约检查通过。见 `tasks/requests/P5-04-core-types-codec-cache.md`。

## 规格对照

完整表见 [docs/traceability/P5-04.md](../../docs/traceability/P5-04.md) 的 2026-10-26 追加。

| 条款/不变量 | 实现与核验 |
| --- | --- |
| V2-76；S §11 / SYS05 | 明确结果状态、有界单页、完整范围续页、原文与十进制纳秒 |
| V2-88、V2-102；EV5/EV6 | c_core 的四路与语料隔离；Task/MEMORY 的类型、来源与反应标记 |
| E10 / V2-07 | 原文不删；撤回原事件及其历史反应不进入 c_task；审计保留 |
| I02、I21 | 只读 Reader，无 CommitCapability；重建/查询不发布权威状态 |
| I18、I23 | 版本化可重建索引、无会话 cursor、实际查询/结果/标记审计；重放不重跑模型 |
| I29 / 黄金规则 14 | 所有库均新建 TemporaryRoot EXPERIMENT；未打开排练或正式存储；谱系 compile_fail |

未新增 IDL、黄金向量或规格差异登记。全部本轮 Rust 文件不超过 400 行。

## 新增依赖

本轮没有新增依赖。PR 原有依赖及 CPU 模型由待用户合并的 ADR 0020 登记：

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| sqlite-vec | 0.1.9 | MIT / Apache-2.0 | 向量距离 |
| fastembed（可选） | 7.3.0 | Apache-2.0 | 本地 CPU embedding |
| ort / ONNX Runtime（可选） | 2.0.0-rc.15 / 1.31.0（402979a） | MIT | CPU 推理 |
| rusqlite（已有，增 hooks/limits） | 0.40.2 | MIT | FTS / 步骤预算 |
| serde / serde_json | 1.0.229 / 1.0.154 | MIT / Apache-2.0 | 审计 |
| sha2 | 0.11.1 | MIT / Apache-2.0 | 文件核验 |
| time | 0.3.44 | MIT / Apache-2.0 | 日期过滤 |
| tempfile（测试） | 3.27.0 | MIT / Apache-2.0 | 临时夹具 |

模型仍为官方 intfloat/multilingual-e5-small，固定修订 `614241f622f53c4eeff9890bdc4f31cfecc418b3`，MIT，FP32、384 维、512 token、mean/L2、CPU 4 线程，加载前逐文件 SHA-256 核验。无 GPU provider 或运行时网络。模型、编译与依赖缓存在 D 盘；临时目录按用户要求继承，不再覆盖到共享 D 盘缓存。

## 待用户确认

- [ ] （合并前阻塞）交叉审查本 PR，包括既有 core-store/core-types 扩展；由用户安排，不联系其他执行者。
- [ ] （合并前阻塞）用户合并批准 ADR 0020 的重量级 CPU 依赖，保留 needs-user。

## 后续建议（不在本任务范围内）

- 终端冻结 QueryV2/QueryResultV2 没有 cursor 字段：本卡只提供进程内续页，不把 cursor 隐藏在 text/时间/query_id。正式传输接线由所属契约/集成卡完成，见 [请求](../../tasks/requests/P5-04-memory-page-transport.md)。
- P5-08/P5-19/P5-29 接入运行时查询审计、纠错与在途输入撤回；P6-13 完成年尺度及完整回答评测。
