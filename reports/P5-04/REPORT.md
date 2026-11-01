# P5-04 事实视图（W19）— 报告

## 当前状态

2026-10-27：两项 P2 已修复于 `918b8dc`；新增三项回归（含 45 个空响应组合）、CI scope 11 场景，以及 main 同步前后的 fmt/clippy/完整 release 工作区测试均通过。已 merge `origin/main@65052c0`（`c4162dec`），保留 needs-user，等待独立复审、当前提交的 CI 及 ADR 0020 批准，见 [BLOCKED.md](BLOCKED.md)。本轮止于推送与「请审查」，不等待新 CI，不改权限/系统设置，不合并、不取下一卡。

## 本轮两项审查修复

- 完整响应预算：`paging.rs` 检索前先核验包含原样 query_id 的空信封；最低字节已超预算时走既有 Error::Invalid，不返回超大消息、不裁短 ID。所有返回再次核验完整 canonical 长度；裁空时同步投影/记忆/审计返回项/续页，零进展不发 cursor。IDL、状态枚举、查询/索引版本、默认预算不变。
- CPU scope：为本 PR 新增的 W19 CPU clippy 补上 `if: steps.scope.outputs.rust == 'true'`，保留原路径判定、手动全量和失败时全量；11 个实际脚本逻辑场景通过，仅 git diff 结果模拟。
- 旧实现反向验证：新增测试在 092cb2fe 的 paging.rs 上于预算断言失败（exit 101，53 字节空信封在 0 字节预算下未拒绝）；固定源码按字节恢复后，新三项测试通过。原始失败日志保留。
- main 同步：合并 #87/#105/#106/#101 后本卡修复源码哈希不变；原样接收 main 的 Windows job 20→60 分钟调整（#106，blob a686643b9c6f7508e949157c36ddaf97e946e8fc）。未自行调整时限、网络或验收阈值，也没有恢复 TEMP/TMP 覆盖。

| 本轮检查 | 结果 | 证据 |
| --- | --- | --- |
| 新预算回归 | 3 通过；三种空状态的短/70000 ASCII/UTF-8 ID 与零/差一字节/精确边界/默认预算等 45 组合；裁空及大 ID 翻页保持原文、顺序与 cursor | [panel-budget-green.log](review-fix-20261026/panel-budget-green.log) |
| 旧实现反向验证 | 预期预算断言失败，exit 101；修复源码完整恢复，SHA 已记录 | [原始失败](review-fix-20261026/budget-regression-red.log)、[控制结果](review-fix-20261026/budget-regression-negative-control.json) |
| Rust fmt / workspace all-targets CPU clippy -D warnings | 同步 main 后再次通过，无警告 | [fmt-main.json](review-fix-20261026/fmt-main.json)、[clippy-main.log](review-fix-20261026/clippy-main.log) |
| 完整 release workspace / CPU 特性 | 同步 main 后通过：core-kernel 86、core-store 32、fact-view 18；谱系 compile_fail 通过。既有诊断/真实模型 ignore 保留，未新增跳过 | [workspace-tests-main.log](review-fix-20261026/workspace-tests-main.log)；[第一轮](review-fix-20261026/workspace-tests.log) |
| CI Rust scope | 11 场景通过，stage/desktop-only 跳过 CPU；Rust/共享文件/工作流/手动与 diff 失败保留覆盖 | [ci-scope-result.json](review-fix-20261026/ci-scope-result.json) |

精确命令、工作目录、起止时间、退出码与检查脚本见 [review-fix-20261026](review-fix-20261026/README.md)。同步后工作区检查在 918b8dc 上执行，后续仅记录报告/证据。本轮没有新依赖、IDL、冻结字节、存储格式、core-store/core-types 修改或性能测量；实际 CPU 模型证据仍属于下述旧恢复提交，不能外推为本轮重新测量。只读机械核验另确认初始和同步后退出码/统计、源码恢复 SHA，以及旧失败目录 14 个文件未变；不将该核验当作第二次运行或审查批准。

## 2026-10-26 Windows CI 失败定位

- [run 40323615588 / self-hosted job 121574544222](https://github.com/nanake-lab/nemu/actions/runs/40323615588/job/121574544222) 的第 19 步 `W02 NTFS storage checks` 在 Cargo 更新 crates.io 索引时失败，步骤退出码 1。`ppv-lite86` 源数据读取反复出现 `transfer too slow`，30 秒 0 字节；错误依赖链为 fact-view → fastembed 7.3.0 → tokenizers 0.23.2 → rand 0.9.5 → rand_chacha 0.9.0 → ppv-lite86 0.2.21。属于此次 runner 构建环境的依赖源网络超时；更下层的网络原因未确定，没有因本次日志而修改实现或依赖。
- 已读原始上传 ZIP 的两个文件（toolchain.log、clippy.log），SHA-256 `20c648f230b27a177cf8182d3488e35cd7e07d93a4c6db065c9cc6b8f5de1dec` 与 GitHub 一致。没有 tests.log，存储测试尚未开始；无编译诊断或测试失败证据。`desktop=false / storage=true`，隔离/E2E/visual 都是 skipped，不能计为本次通过。另一汇总 job 仅传播上游 failure。
- 本次证据成功写入 `D:\actions-runner\_work\_temp\w02-40323615588-1` 并上传；工作流 blob 与 main 一致，未恢复 TEMP/TMP 覆盖，未改权限、系统/用户/全局设置、runner、超时或阈值，未主动重跑 CI。仅核验证据、哈希、代码与工作流未变化，不用旧的本机成功替代失败 job。
- [诊断与完整证据](windows-failure-fbcb7cc-20261026/README.md) 保留为 fbcb7cc 的失败；[交叉审查](https://github.com/nanake-lab/nemu/pull/65#issuecomment-6419219224) 的两项代码问题已按上节修复并待复审，与原网络失败分别记账。此节的零代码变化、工作流状态及未重跑检查描述只针对 092cb2fe 的诊断提交。

## 交付物

- `core/crates/fact-view/`：已提交前缀、实体/时间/FTS5 trigram/本地多语言向量四路召回、固定 RRF、用途与预算隔离、纠错/撤回、原始对象核验及查询审计。
- V2-88：c_task/MEMORY 可读六类 ACCEPTED 历史解释，显式标记「这是她当时的反应」。保留原始 ObjectRef、事件 ID、原文语义、精确纳秒及补解释的独立记录时刻；不把 ProposalBinding 的状态引用放进模型文本。c_core 四路都排除反应，FTS 语料与 embedding 输入独立，不构造任务展示数据。
- V2-76：`FactQuery.after`、`Execution.next_page`、`user_view_page` 提供无会话续页，绑定查询/前缀/权限/预算/模型；记忆召回取消每路前 64 的总量截断，仍限制每次条数、字节与步骤。75 条同一精确纳秒的记录经 11 页完整返回，无重漏；重建后可继续。
- NO_MATCH、DENIED、INDEX_LAGGING 及 OBJECT_MISSING、BUDGET_EXHAUSTED 分别返回。撤回连同关联反应过滤；补解释同时检查原事件和记录事件的来源权限；历史有新提交时旧索引/续页明确滞后。
- 查询/索引版本为 `w19-rrf-v2` / `w19-trigram-vec-v2`，RRF k=60、通道和稳定平局规则固定；c_core 保留首轮每路前 64。反应向量复用被解释经历的向量，自身语义字段仍可全文检索。审计扩展数据版本为 2，记录标记、实际结果与续页；权威 IDL 和既有字节不变。
- Windows 工作流当前 blob 与 main 均为 `a686643b9c6f7508e949157c36ddaf97e946e8fc`；本机 env.ps1 不覆盖 TEMP/TMP。临时目录继承当前进程配置，未授权或修复任何共享目录 ACL、用户或系统设置。

## 验收

以下为 2026-10-26 恢复阶段的既有结果；本轮修复与同步后的检查单独在上表及 review-fix-20261026 记录，不混用提交或性能数据。

精确命令、工作目录、起止时间、退出码与原始输出均在 [resume-20261026](resume-20261026/) 的同名 JSON/日志中。全部 Rust 检查使用锁定的 Rust 1.99.0；重负载串行。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --manifest-path core/Cargo.toml --all -- --check` | 通过；最终 main 同步后复验通过 | [fmt.json](main-sync-20261026/fmt.json) |
| `cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets --features fact-view/cpu-embedding --locked -- -D warnings` | 通过；最终 main 同步后复验通过 | [clippy.log](main-sync-20261026/clippy.log) |
| `cargo test --manifest-path core/Cargo.toml --workspace --release --features fact-view/cpu-embedding --locked -- --test-threads=1` | 通过；core-store 32 项、fact-view 15 项及谱系 compile_fail 通过。之后仅文档同步，运行字节零差异，事实视图 15 项及 compile_fail 再通过 | [workspace-tests-final.log](resume-20261026/workspace-tests-final.log)、[同步后](main-sync-20261026/fact-view.log) |
| SYS05；V2-76、V2-88；100 次撤回 | 五种状态、成功对照、75 条/11 页、原文与精确时间、标记、权限/续页绑定、对象缺失清空、c_core 排名/步骤独立、撤回传播通过 | [SYS05 结果](SYS05/result-resume-20261026.json)；上述工作区日志 |
| `cargo test --manifest-path core/Cargo.toml -p fact-view --release --features cpu-embedding real_cpu_multilingual --locked -- --ignored --nocapture --test-threads=1` | 最终同步后真实临时 EXPERIMENT 库重启；三语 9/9 向量与 RRF 排名第 1；100 次撤回零泄漏 | [cpu-model.log](main-sync-20261026/cpu-model.log)、[cpu-result.json](main-sync-20261026/cpu-result.json) |
| ADR 0012 第一轮 50 ms 目标 | 最终 c_core 九查询 p50 **34.54 ms**、max **35.16 ms**；9/9 低于 50 ms。含历史反应的完整 c_task 页另计：p50 **98.42 ms**、max **101.55 ms** | 同上；仅合成骨架观测，不是任意规模 SLO；同步前的 36.52/38.40 ms 等数据仍保留 |
| 原 Windows 隔离脚本 | 通过：Low 令牌、Medium 目录拒写、正向写入对照；只操作它新建并清理的测试子目录 | [windows-isolation.log](resume-20261026/windows-isolation.log) |
| `pnpm lint`、`pnpm typecheck` | 通过 | [lint.log](resume-20261026/lint.log)、[typecheck.log](resume-20261026/typecheck.log) |
| `pnpm test` | 首次 579 通过/1 失败；main 独立基线及本分支原命令复测均 580/580 通过，失败保留 | [首次](resume-20261026/node-tests.log)、[main](resume-20261026/main-node-tests.log)、[复测](resume-20261026/node-tests-recheck.log) |
| `pnpm test:contracts` | 354 通过；语句/行/函数 100%，分支 97.46% | [contracts.log](resume-20261026/contracts.log)、[coverage-summary.json](resume-20261026/coverage-summary.json) |
| `pnpm test:core-contracts` | Rust/Python/TS 各 10,766 例；生成 0 stale、兼容 0 违规；protoc 445 条通过 | [core-contracts.log](resume-20261026/core-contracts.log) |
| 冻结 IDL/生成物相对 main | 0 差异；本轮未再修改 core-store/core-types；最终文档同步前后运行字节完全相同 | [verification.json](resume-20261026/verification.json)、[最终同步](main-sync-20261026/verification.json) |

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

- [ ] （合并前阻塞）当前提交的 Windows/其他 CI 通过，按提交核对，不用旧成功或旧失败代替；本轮不等待新 CI，见 BLOCKED.md。
- [ ] （合并前阻塞）独立复审两项已修复的 P2，不把本机回归当作审查批准。
- [ ] （合并前阻塞）交叉审查本 PR，包括既有 core-store/core-types 扩展；由用户安排，不联系其他执行者。
- [ ] （合并前阻塞）用户合并批准 ADR 0020 的重量级 CPU 依赖，保留 needs-user。

## 后续建议（不在本任务范围内）

- 终端冻结 QueryV2/QueryResultV2 没有 cursor 字段：本卡只提供进程内续页，不把 cursor 隐藏在 text/时间/query_id。正式传输接线由所属契约/集成卡完成，见 [请求](../../tasks/requests/P5-04-memory-page-transport.md)。
- P5-08/P5-19/P5-29 接入运行时查询审计、纠错与在途输入撤回；P6-13 完成年尺度及完整回答评测。
