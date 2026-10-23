# P5-04 事实视图（W19）— 报告

## 结果

实现与验收进行中，尚不能记完成；PR #65 保持草稿。已从 origin/main 876cff7 创建 D:\Dev\nemu-wt\P5-04，分支 task/P5-04-fact-view。

## 已有证据

- 事实视图正向白名单、四路召回、固定 RRF、独立用途预算、五种非成功状态、纠错与撤回链已经实现。
- 首轮临时库测试 4/5 通过；SYS05 首次失败是测试路径比较遗漏 Windows 规范化前缀，修正后定向复测通过。原始失败日志保留。
- 真实官方模型 CPU 初测：9 条合成事实，重启临时 EXPERIMENT 存储后，三种语言 9/9 查询的正确事实均为向量与合并排名第 1；向量启用后撤回的事实在 100 次相关查询中 0 次进入 c_task 或任何召回通道。cpu-eval-2.log。此为评测骨架，不是 P6-13 多年规模/完整回答的 90% 验收。
- FP32 初测 debug 查询中位 346.60 ms；release 中位 87.12 ms、最大 103.60 ms，仍未达到 50 ms 原目标。正在核对同一官方修订的量化权重；固定事实、问题、预算、四线程与撤回次数不变。原数据保留，不改目标。
- Rust 全工作区、全部目标、可选 CPU 特性 clippy -D warnings 通过（clippy-2.log）。全部本地回归与最终 main 合并后的检查尚待完成。
- 所有开发测试库来自 TemporaryRoot；模型、Cargo/ORT/uv/npm/pnpm/临时与编译缓存在 D 盘。未打开排练库或正式库。
- 首轮 Ubuntu CI 和本地完整回归均为 core-store 31/32 通过。唯一失败是新增测试的损坏注入先违反外键约束，未进入预期的恢复核验断言；修复为删除临时库的 snapshots 索引行，不关闭外键、不改生产代码。失败日志 ci-first-failure.log、workspace-tests-final.log 保留，最新提交等待完整复测。

## core-store 扩展（用户专项授权，必审）

仅追加 core/crates/core-store/src/queries.rs 的 Reader::transactions_at 与 src/tests/reader.rs 测试。不改格式、表、写路径或已有接口行为。捕获持久化回执后建立固定只读快照，先按现有 Reader 路径做谱系头、恢复、索引与回执核验，再读取完整已知前缀；未知前缀、缺失索引和预算不足明确失败。Reader<L> 的谱系类型保留。P5-03 PR #64 的并行情况已先在 #65 PR 正文说明。

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
| ort（可选）/ ONNX Runtime | 2.0.0-rc.15 / 运行时构建见评测 JSON | MIT | 显式 CPUExecutionProvider |
| rusqlite（工作区已有，启用 hooks/limits） | 0.40.2 | MIT | FTS5/内存索引/步骤预算 |
| serde / serde_json | 1.0.229 / 1.0.154 | MIT / Apache-2.0 | 版本化审计编码 |
| sha2 | 0.11.1 | MIT / Apache-2.0 | 模型 SHA-256 核验 |
| time | 0.3.44 | MIT / Apache-2.0 | 面板 UTC 日期与 RFC3339 过滤 |
| tempfile（仅测试） | 3.27.0 | MIT / Apache-2.0 | 损坏模型的临时夹具 |

核心本地依赖仍为 core-types、core-store。传递依赖锁在 core/Cargo.lock；tokenizers 0.23.2 为 Apache-2.0。模型 MIT、可分发，来源、修订、长度与逐文件 SHA-256 全部见 manifest。

## 待用户确认

- [ ] （阻塞）新增重量级 CPU 依赖 ADR 0020，由用户合并批准。
- [ ] （阻塞）core-store 两文件扩展按用户授权属于必审存储范围，由用户安排新 Codex 会话一并审查。
- [ ] （阻塞）当前 50 ms 目标尚未满足；量化与完整复测结束前不改阈值、不转 ready。

## 下一步

完成量化核对、全部临时库回归、最新 main 合并与重跑、CI。PR #57 尚未合并，目前使用 main 的生成 Query/QueryResult；如 #57 合并则 merge main 后对齐。用户已启动 Windows runner；当前先执行 main 的 W02，#65 最新手动作业 39898374730 排队中。仅在验收完成后转为可审查并在 PR 写「请审查」，由用户安排审查与合并，不认领下一张卡。
