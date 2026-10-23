# P5-04 事实视图（W19）— 报告

## 结果

实现、本地验收与最终代码 Ubuntu CI 完成；Windows CI 尚未完成，当前外部阻塞见 BLOCKED.md，PR #65 保持草稿。从 origin/main 876cff7 创建 D:\Dev\nemu-wt\P5-04，分支 task/P5-04-fact-view；最终代码为 954918fae3f9db245fc9a92ca646c090d655510f。已再次 fetch 并 merge main（无新提交），在此之后重跑以下验收。

## 已有证据

- 事实视图正向白名单、四路召回、固定 RRF、独立用途预算、五种非成功状态、纠错与撤回链已经实现。
- 最终事实视图 7/7 测试通过，包含 SYS05 五种状态、解释记录结构白名单、纠错链、权限与预算、短 Unicode/FTS 字面量、旧索引阻断。谱系编译失败用例通过。SYS05 首次路径比较遗漏 Windows 规范化前缀的失败日志保留。
- 真实官方 FP32 CPU：9 条合成事实，重启临时 EXPERIMENT 后，三语 9/9 查询的正确事实均为向量与合并排名第 1；向量启用后撤回事实在 100 次相关查询中 0 次进入 c_task 或任何召回通道。见 recall/result.json、cpu-eval-acceptance.log。
- 原目标 50 ms 未改：最终 release 九个完整查询中位 43.33 ms、最大 45.18 ms，九个均小于 50 ms；模型加载/核验 1.676 s，索引重建 0.889 s。CPU Ryzen 7 3700X、显式 CPU 4 线程；查询包含原始 H 与载荷重读核验。此为固定小规模观测，不代表多年规模/完整回答 90% 验收或任意负载 SLO。
- 优化前同一 FP32 对照中位 91.13 ms，embedding 多数仅 5–6 ms。用户授权的纯描述符缓存后，固定问题、条数、预算、四线程和撤回次数不变。本次选择最终复测 43.33 ms，而非挑选较快的初步 42.16 ms。更早 debug 346.60 ms/release 87.12 ms 与量化候选失败日志一并保留；量化入口已移除。
- Rust 全工作区、全部目标、可选 CPU 特性 clippy -D warnings 和完整 release 工作区回归通过；core-store 32/32。全部合法/非法黄金向量冷/热缓存相同，三语言各 9,744 例通过、兼容 0 违规，312 项契约单测通过。
- 所有开发测试库来自 TemporaryRoot；模型、Cargo/ORT/uv/npm/pnpm/临时与编译缓存在 D 盘。未打开排练库或正式库。
- 首轮 Ubuntu CI 和本地完整回归均为 core-store 31/32。唯一失败是新增测试的损坏注入先违反外键约束；修复为删除临时库 snapshots 索引行，不关闭外键、不改生产代码。修复后全部通过，失败日志 ci-first-failure.log、workspace-tests-final.log 保留。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| cargo fmt --all -- --check | 通过 | fmt-acceptance.log |
| cargo clippy --workspace --all-targets --features fact-view/cpu-embedding --locked -- -D warnings | 通过 | clippy-acceptance.log |
| cargo test --workspace --release --locked -- --test-threads=1 | 通过；含完整故障注入矩阵、32 项 core-store、7 项 fact-view、谱系 compile_fail | workspace-tests-acceptance.log |
| SYS05 面板五种不同结果及成功正向例 | 通过 | SYS05/result.json |
| c_core/c_task 解释记录结构检查 | 通过 | structural_allowlist_excludes_all_explanations_even_when_wrapped_as_event_payload |
| 真实 CPU 模型、三语召回骨架、100 次撤回 | 9/9 第 1；0/100 泄漏；九查询均 <50 ms | recall/result.json、recall/latency.json |
| core-types 全部黄金向量冷/热/无缓存 | 合法 396、非法 409；结果/规范字节/摘要/Err(String) 逐项相同 | core-types-cache-tests.log |
| pnpm test:core-contracts | Rust/Python/TypeScript 各 9,744 通过；0 stale、0 compatibility violations；官方 protoc 396 通过 | core-contracts-acceptance-2.log |
| pnpm test:contracts | 312 通过；语句/行覆盖 100%，分支 97.1% | contracts-acceptance.log、coverage-summary.json |
| git diff origin/main -- core/contracts packages/contracts/src/generated workers/irl/generated/echo | 0 差异 | 生成资源与黄金字节未改；schema SHA-256 为 4fd66832d88181b45262912dbdee6925a63328a778491645a03995fe2a13bc9a |
| Ubuntu CI，最终代码 | 全部通过 | ci/ubuntu-954918f.json；https://github.com/nanake-lab/nemu/actions/runs/39901232250 |
| Windows CI，最终代码 | 排队；runner 当前离线 | https://github.com/nanake-lab/nemu/actions/runs/39901710241 |

本地三语言首次运行的验证器固定寻找 core/target，而编译产物在 D:\Dev\cache\cargo-target\P5-04，导致 FileNotFoundError。失败日志 core-contracts-acceptance.log 保留；只在本工作区创建 D 盘目录联接 core/target 指向该缓存，未改验证器，随后重跑通过。没有生成或改写冻结预期值。

## core-store 扩展（用户专项授权，必审）

仅追加 core/crates/core-store/src/queries.rs 的 Reader::transactions_at 与 src/tests/reader.rs 测试。不改格式、表、写路径或已有接口行为。捕获持久化回执后建立固定只读快照，先按现有 Reader 路径做谱系头、恢复、索引与回执核验，再读取完整已知前缀；未知前缀、缺失索引和预算不足明确失败。Reader<L> 的谱系类型保留。P5-03 PR #64 的并行情况已先在 #65 PR 正文说明。

## core-types 扩展（用户专项授权，必审）

仅 src/lib.rs 与 tests/canonical_boundaries.rs。public Schema 可复用，但 Reader::read 没有注入解码器的接口，fact-view 内持有描述符不能覆盖它内部的 ObjectRef 编码/对象核验/typed 解码；因此按用户条件采用两文件扩展。100 次原 WireMessage 编码 60.14 ms，复用原始描述符 0.16 ms；缓存后 WireMessage 1.14 ms，见 descriptor-profile-before/after.log。

OnceLock 一次性建立不可变描述符图，键为类型标签、支持的 schema_version=1、生成 schema 的 SHA-256。所有 encode/decode/normalize、未知版本、尺寸/深度/条数限制照常执行；不缓存对象、原文、权限或核验结果，除初始化外没有可变缓存状态。Schema::builtin 的公开签名与独立可变实例行为保留，外部修改不影响 codec。

逐条新建局部 OnceLock 走同一个生产 accessor，测全部 396 合法/409 非法黄金向量的冷缓存、热缓存、无缓存结果；值、规范字节、摘要和 String 错误完全相同。另有独立子进程的公开接口首次/后续等价测试及并发、预算和可变实例隔离。未改 #57 当前 diff 中的文件；后合并者 merge main 后重跑全部契约检查。详见 tasks/requests/P5-04-core-types-codec-cache.md。

## 文档与模型

规格对照见 docs/traceability/P5-04.md。使用方法、预算、审计对象与运行时接入边界见 core/crates/fact-view/README.md。

官方 intfloat/multilingual-e5-small，MIT，固定修订 614241f622f53c4eeff9890bdc4f31cfecc418b3，SHA-256 清单见 fact-view/models/manifest.json。模型位于 D:\Dev\models\multilingual-e5-small\614241f622f53c4eeff9890bdc4f31cfecc418b3；所有文件已实际下载并核验。加载只读本地文件，无 GPU provider、无运行时网络或 Python 子进程。

新增 ADR 0020 登记可选 fastembed/ONNX CPU 运行时与集中 sqlite-vec FFI；新 ADR 按规则 needs-user，由用户合并批准，不自动合并。

## 查询参数与交付物

- core/crates/fact-view/：仅经 Reader 读取已提交历史；私有内存 SQLite 派生索引，不开权威数据库路径。Rust 谱系类型贯穿 FactView 与 Reader。
- 计划 w19-rrf-v1，索引 w19-trigram-vec-v1；实体、时间、FTS5 trigram、sqlite-vec cosine 四路，固定 RRF k=60、每路前 64，固定通道次序与 source_id 平局规则。不用 p、m、倾向强度或过去评估重排。
- c_core 默认 16 条/16 KiB/100k 步；c_task 32 条/32 KiB/200k 步；user_view 50 条/64 KiB/300k 步。步骤覆盖 Rust 检索工作与 SQLite VM；字节包含投影及原始载荷，面板另核验响应字节。拒绝权限为默认值。
- 解释记录用正向载荷白名单隔离；已接受语义图只提取有来源的身份。撤回控制事件及 note 不索引；c_task/MEMORY 排除撤回，c_core 保留经历，审计可读原文。旧索引遇新提交明确 INDEX_LAGGING。
- 每次执行返回原 IDL FactProjection 与实际查询、许可范围、模型、向量位模式、各路排名、结果的 RetrievalTrace；audit_record 产出生成的 AuditExtension。外层运行时负责 stage/pin 与写入 audit_refs，见 README；本 crate 没有提交能力。编排接入属于 P5-08/P5-19/P5-29。
- 不启用可选小模型重排和抽象段落摘要；保留可重建的原始段落与 H 来源。P6-13 的多年规模/完整回答 90% 验收未运行，本卡仅交付评测骨架。

## 规格对照与静态边界

完整映射见 docs/traceability/P5-04.md：SYS05、E10/V2-07、EV5/EV6/V2-14、ADR 0012 §7、I29、PR #58 A1/B2。I18 通过可丢弃重建索引与显式审计资源版本保证；I23 不在重放时重跑模型；I02/I29 由借用只读 Reader、谱系类型和临时库测试约束。IDL 与黄金向量没有改动，没有新增差异登记条目。

retrieval.rs 为 405 行：预算进度钩子、统一退出清理和四路检索紧邻，面板适配复用同一执行路径；其余 Rust 模块均小于 400 行。唯一 unsafe 在 sqlite_vec.rs 的静态 FFI 注册，每处有说明；其他模块 deny unsafe。

## 新增直接依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| sqlite-vec | 0.1.9 | MIT / Apache-2.0 | 静态 SQLite 向量距离 |
| fastembed（可选） | 7.3.0 | Apache-2.0 | CPU embedding、tokenizer、pooling |
| ort（可选）/ ONNX Runtime | 2.0.0-rc.15 / 1.31.0，git 402979a | MIT | 显式 CPUExecutionProvider；完整构建标识见评测 JSON |
| rusqlite（工作区已有，启用 hooks/limits） | 0.40.2 | MIT | FTS5/内存索引/步骤预算 |
| serde / serde_json | 1.0.229 / 1.0.154 | MIT / Apache-2.0 | 版本化审计编码 |
| sha2 | 0.11.1 | MIT / Apache-2.0 | 模型 SHA-256 核验 |
| time | 0.3.44 | MIT / Apache-2.0 | 面板 UTC 日期与 RFC3339 过滤 |
| tempfile（仅测试） | 3.27.0 | MIT / Apache-2.0 | 损坏模型的临时夹具 |

核心本地依赖仍为 core-types、core-store。传递依赖锁在 core/Cargo.lock；tokenizers 0.23.2 为 Apache-2.0。模型 MIT、可分发，来源、修订、长度与逐文件 SHA-256 全部见 manifest。

## 待用户确认

- [ ] （阻塞）新增重量级 CPU 依赖 ADR 0020，由用户合并批准。
- [ ] （阻塞）core-store 与 core-types 两处授权扩展均属于必审范围，由用户安排新 Codex 会话一并审查。
- [ ] （阻塞）最终 Windows CI 尚未运行完成；当前 runner 离线，等待用户恢复。Ubuntu CI 全通过。机械验收全通过前保持草稿。

## 下一步

等待最终 CI；#57 仍未合并，目前使用 main 的生成 Query/QueryResult，如 #57 合并则 merge main 后对齐。先前 main Windows 作业成功后 runner 再次离线，#65 最终手动作业 39901710241 排队中；已通知用户。仅在全部机械验收通过、最终 main 核对后转为可审查并在 PR 写「请审查」，由用户安排新 Codex 会话审查与合并，不认领下一张卡。
