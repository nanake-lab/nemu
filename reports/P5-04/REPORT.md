# P5-04 事实视图（W19）— 报告

## 当前状态

部分完成，PR #65 保持草稿 + needs-user。本轮本机及 Ubuntu 验收全通过，手动 Windows 39986857254 失败于隔离测试设置临时目录完整性标签（Access denied），未进入 W02/W19。当前代码 785a1bb 已合并 origin/main 0b5412e（含 #57、#61、#48），MEMORY 已对齐 QueryV2/QueryResultV2。按用户指示自动等主机空闲后完成本机检查，再手动触发 Windows；修复权限的最小方案见 BLOCKED.md，等待用户选择。验收通过后才转可审查、写「请审查」，由用户安排新 Codex 会话；不合并、不取下一张卡。

## 交付物

- core/crates/fact-view/：只读事实视图，精确实体/时间、FTS5 trigram、本地多语言向量四路召回，固定 RRF；正向白名单排除解释记录，分用途预算，五种非成功状态，纠错与撤回链，完整查询审计。
- core-store 的两文件只读历史前缀扩展与 core-types 的两文件不可变内置 codec 缓存：用户专项授权，均须审查，详见下文。
- docs/traceability/P5-04.md、ADR 0020、两份 tasks/requests/P5-04-* 请求：规格、依赖、授权边界。
- 模型 manifest、下载脚本及 reports/P5-04/：固定版本与 SHA-256、临时库测试、召回/撤回/耗时及 CI 原始证据。旧证据不覆盖。

## 验收

以下本机结果与 Ubuntu CI 均对应代码 785a1bb；目录 main-sync-20261024/ 记录本轮。旧原始证据另行保留，没有覆盖。

| 命令或检查 | 当前结果 | 证据 |
| --- | --- | --- |
| cargo fmt --all -- --check；cargo clippy --workspace --all-targets --features fact-view/cpu-embedding --locked -- -D warnings | 通过 | main-sync-20261024/fmt.log、clippy.log |
| cargo test --workspace --release --locked -- --test-threads=1 | 通过；core-store 32/32、fact-view 8/8、谱系 compile_fail；SYS05 五状态与成功例、结构隔离、100 次撤回、新 V2 面板用例 | main-sync-20261024/workspace-tests.log；SYS05/result-main-sync-20261024.json |
| cargo test -p fact-view --release --features cpu-embedding real_cpu_multilingual --locked -- --ignored --nocapture --test-threads=1 | 通过；真实临时库重启、三语 9/9 向量与合并排名第 1，100 次撤回 0 泄漏 | main-sync-20261024/cpu-model.log、cpu-result.json |
| 第一轮检索 50 ms 目标 | 完整九查询 p50 44.50 ms、max 45.91 ms；9/9 小于 50 ms；模型加载 2.220 s、索引重建 0.894 s | main-sync-20261024/cpu-result.json |
| core-types 全部合法/非法黄金向量冷/热/无缓存等价 | 445 合法、457 非法逐项相同；首次/后续公开接口、并发、独立可变 Schema 测试通过 | main-sync-20261024/workspace-tests.log |
| pnpm lint / typecheck / test | 通过；535 项测试 | main-sync-20261024/lint.log、typecheck.log、node-tests.log |
| pnpm test:contracts | 350 通过；语句/行/函数 100%，分支 97.46% | main-sync-20261024/contracts.log、coverage-summary.json |
| pnpm test:core-contracts | Rust/Python/TypeScript 各 10,766 例通过；0 stale、0 compatibility violations；官方 protoc 445 条通过 | main-sync-20261024/core-contracts.log |
| IDL / 生成文件 / 冻结字节相对 origin/main | 0 差异；schema SHA-256 c24393b63665dde9c29e9da2278430f7a45c0a75c0cb4d65ce350149c1933a88 | git diff origin/main -- core/contracts packages/contracts/src/generated workers/irl/generated/echo |
| Ubuntu CI | 785a1bb 全部成功 | main-sync-20261024/ubuntu-785a1bb.json；https://github.com/nanake-lab/nemu/actions/runs/39983818418 |
| Windows CI | 失败：隔离测试的临时目录完整性标签设置 Access denied；W02/W19 未运行，草稿自动跳过不算通过 | main-sync-20261024/windows-785a1bb.json、windows-39986857254-failure.log、windows-evidence/；https://github.com/nanake-lab/nemu/actions/runs/39986857254 |

旧 Windows 39901710241 与本轮 39986857254 均在隔离测试的 icacls 临时目录完整性标签设置处 Access denied。现场只读检查显示 D:\Dev\cache\tmp 继承 Authenticated Users 的 Modify，当前用户为该目录 owner，但没有当前用户的显式 FullControl；这是最小 ACL 修复方案的依据，尚未改 ACL 或验证该方案。没有切回 C 盘、跳过隔离测试或重试相同失败。

此前其他失败也保留：量化候选未通过固定正向召回；首次存储损坏夹具先触发外键错误，改为删除临时 snapshots 索引行后通过；本地契约验证器固定 core/target 与 D 盘编译路径不一致，创建本工作区 D→D 目录联接后通过。没有放宽阈值、修改冻结预期值或删除失败日志。

## core-store 扩展（必审）

仅 core/crates/core-store/src/queries.rs 与 src/tests/reader.rs，追加 Reader::transactions_at 及 TemporaryRoot 测试。不改格式、表、写路径或已有接口行为。捕获持久化回执后建立固定只读快照，先走现有谱系头、恢复、索引与回执核验，再取完整已知前缀；未知前缀、缺失索引、预算不足明确失败。Reader<L> 保留打开时的谱系类型，不能跨谱系调用。

必须取得旧事务的 ACCEPTED SemanticGraph 才能重建有来源的实体索引；fact-view 不能绕过 core-store 直接打开权威 SQLite。P5-03 PR #64 并行情况已在实现前写入 #65；2026-10-23 检查其 diff 未涉及这两个文件。专项授权见 tasks/requests/P5-04-core-store-prefix.md。

## core-types 扩展（必审）

仅 src/lib.rs、tests/canonical_boundaries.rs。先确认了公开 Schema 可复用，但 Reader::read 无解码器注入接口，上层持有描述符不能覆盖 Reader 内部 ObjectRef 编码、对象核验和 typed 解码，故采用授权的最小扩展。100 次原 WireMessage 编码 60.14 ms，复用描述符 0.16 ms，缓存后 WireMessage 1.14 ms，见 descriptor-profile-before/after.log。

OnceLock 一次初始化内置不可变描述符，键含类型标签、schema_version=1、生成 schema.json 的 SHA-256；初始化后无可变状态。全部 encode/decode/normalize、原文核验、限制、错误顺序照常执行，不缓存对象或核验结果。公开 Schema::builtin 仍返回独立可变实例。测试逐项比较全部合法/非法黄金向量在冷、热、无缓存下的结果、规范字节、摘要、String 错误；另测独立子进程公开接口第一次/后续等价、并发及外部可变实例隔离。

合并前同一 FP32 九查询 p50 91.13 → 43.33 ms；合并最新 main 后本轮 p50 44.50 ms、max 45.91 ms。召回条数、预算、模型、四线程和撤回次数未改。#57 已合并，本卡未修改其 contract.rs、生成文件或冻结字节，全部契约检查已重跑通过。专项授权见 tasks/requests/P5-04-core-types-codec-cache.md。

## 参数、模型与边界

计划 w19-rrf-v1、索引 w19-trigram-vec-v1；RRF k=60、每路前 64，固定实体/时间/FTS/向量通道次序和 source_id 平局规则。不按 p、m、倾向强度或过去评估重排。c_core 默认 16 条/16 KiB/100k 步，c_task 32/32 KiB/200k，user_view 50/64 KiB/300k；步骤含 Rust 工作和 SQLite VM，字节含投影与原始载荷。默认拒绝权限，各路在排序前过滤。

官方 intfloat/multilingual-e5-small，MIT，固定修订 614241f622f53c4eeff9890bdc4f31cfecc418b3，FP32、384 维、512 token、mean/L2、query:/passage:、显式 CPU 4 线程。逐文件 SHA-256 见 fact-view/models/manifest.json；模型在 D:\Dev\models\multilingual-e5-small\614241f622f53c4eeff9890bdc4f31cfecc418b3。模型、uv/npm/pnpm/Cargo/ORT/临时和编译缓存均在 D 盘，无 GPU provider、运行时网络或 Python 子进程。全部开发库来自 TemporaryRoot，未打开排练或正式库。

每次返回重读并核验 H 与原始载荷。撤回控制事件及 note 不索引；c_task/MEMORY 排除撤回，c_core 保留经历，审计可看原文；旧索引遇新提交返回 INDEX_LAGGING。实际查询、权限范围、模型、向量位模式、各路排名和结果产出生成的 AuditExtension，由外层运行时 stage/pin 至 audit_refs；本 crate 无提交能力。接入编排属于 P5-08/P5-19/P5-29。

不启用可选小模型重排或抽象摘要；原始段落及 H 为来源。九事实/九查询是 P6-13 前评测骨架，不代表多年规模、完整回答 90% 或任意负载的 50 ms SLO。retrieval.rs 约 406 行，预算进度钩子、统一退出清理、四路查询与面板适配共用执行路径，故保留在一个模块；唯一 unsafe 为 sqlite_vec.rs 静态 FFI 注册，其余模块 deny unsafe。

## 规格对照

完整映射见 docs/traceability/P5-04.md。

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| W19 / SYS05；CONTRACTS v2.2 §4.3 | query.rs、retrieval.rs；生成 QueryV2/QueryResultV2，仅 MEMORY、growth_entries 恒空 | SYS05 五状态及成功例、V2 边界回归 |
| EV5/EV6 / V2-14 | source.rs 正向白名单、仅接受图的身份及原文 span | 解释载荷包装为事件仍被排除 |
| E10 / V2-07 | source.rs 撤回与纠错链 | Task 100 次零泄漏、Core/Audit 保留经历 |
| ADR 0012 §7 | index.rs、retrieval.rs、embedding/cpu.rs | 固定 RRF、三语真实 CPU 正向召回 |
| I02/I29、黄金规则 14、#58 A1/B2 | 只读 Reader<L>、固定快照完整前缀、TemporaryRoot | 正常/损坏前缀、谱系 compile_fail |
| I18/I23 | 可重建版本化索引、查询/返回审计 | trace 确定性；重放不重跑模型 |

未新增或修改 IDL、黄金向量、权威编码、状态算子或差异登记；只继承最新 main 的契约。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| sqlite-vec | 0.1.9 | MIT / Apache-2.0 | SQLite 向量距离 |
| fastembed（可选） | 7.3.0 | Apache-2.0 | CPU tokenizer/pooling/embedding |
| ort（可选）/ ONNX Runtime | 2.0.0-rc.15 / 1.31.0，git 402979a | MIT | CPU provider |
| rusqlite（已有，增 hooks/limits） | 0.40.2 | MIT | FTS/步骤预算 |
| serde / serde_json | 1.0.229 / 1.0.154 | MIT / Apache-2.0 | 审计编码 |
| sha2 | 0.11.1 | MIT / Apache-2.0 | 模型核验 |
| time | 0.3.44 | MIT / Apache-2.0 | 面板日期过滤 |
| tempfile（测试） | 3.27.0 | MIT / Apache-2.0 | 损坏模型临时夹具 |

## 待用户确认

- [ ] （阻塞）ADR 0020 新增重量级 CPU 依赖，由用户合并批准，保留 needs-user。
- [ ] （阻塞）core-store、core-types 扩展须由用户安排新 Codex 会话一并审查。
- [ ] （阻塞）最终手动 Windows CI 的 D 盘临时目录权限失败；等待用户选择任务独占目录 ACL 最小修复或自行修复 runner 环境，见 BLOCKED.md、tasks/requests/P5-04-windows-temp-acl.md。通过前保持草稿。

## 后续建议（不在本任务范围内）

- P5-08/P5-19/P5-29 接入运行时审计与纠错；P6-13 运行多年规模/完整回答评测。
