# P5-04 事实视图（W19）— 进行中报告

## 结果

实现与首轮验收进行中；保持草稿。已从 origin/main 876cff7 创建 D:\Dev\nemu-wt\P5-04，分支 task/P5-04-fact-view，PR #65。

## 已有证据

- 事实视图正向白名单、四路召回、固定 RRF、独立用途预算、五种非成功状态、纠错与撤回链已经实现。
- 首轮临时库测试 4/5 通过；SYS05 首次失败是测试路径比较遗漏 Windows 规范化前缀，修正后定向复测通过。原始失败日志保留。
- 真实官方模型 CPU 初测：9 条合成事实，重启临时 EXPERIMENT 存储后，三种语言 9/9 查询的正确事实均为向量与合并排名第 1；向量启用后撤回的事实在 100 次相关查询中 0 次进入 c_task 或任何召回通道。cpu-eval-2.log。此为评测骨架，不是 P6-13 多年规模/完整回答的 90% 验收。
- 初测为 debug 构建，查询中位 346.60 ms，未达到 50 ms 原目标。正在用 release 构建核对生产编译方式下的开销；原数据保留，不改目标。
- Rust 全工作区、全部目标、可选 CPU 特性 clippy -D warnings 通过（clippy-2.log）。全部本地回归与最终 main 合并后的检查尚待完成。
- 所有开发测试库来自 TemporaryRoot；模型、Cargo/ORT/uv/npm/pnpm/临时与编译缓存在 D 盘。未打开排练库或正式库。

## core-store 扩展（用户专项授权，必审）

仅追加 core/crates/core-store/src/queries.rs 的 Reader::transactions_at 与 src/tests/reader.rs 测试。不改格式、表、写路径或已有接口行为。捕获持久化回执后建立固定只读快照，先按现有 Reader 路径做谱系头、恢复、索引与回执核验，再读取完整已知前缀；未知前缀、缺失索引和预算不足明确失败。Reader<L> 的谱系类型保留。P5-03 PR #64 的并行情况已先在 #65 PR 正文说明。

## 文档与模型

规格对照见 docs/traceability/P5-04.md。使用方法、预算、审计对象与运行时接入边界见 core/crates/fact-view/README.md。

官方 intfloat/multilingual-e5-small，MIT，固定修订 614241f622f53c4eeff9890bdc4f31cfecc418b3，SHA-256 清单见 fact-view/models/manifest.json。模型位于 D:\Dev\models\multilingual-e5-small\614241f622f53c4eeff9890bdc4f31cfecc418b3；所有文件已实际下载并核验。加载只读本地文件，无 GPU provider、无运行时网络或 Python 子进程。

新增 ADR 0020 登记可选 fastembed/ONNX CPU 运行时与集中 sqlite-vec FFI；新 ADR 按规则 needs-user，由用户合并批准，不自动合并。

## 下一步

完成 release 核对、全部临时库回归、最新 main 合并与重跑、CI，最后转为可审查并在 PR 写「请审查」。PR #57 尚未合并，目前使用 main 的生成 Query/QueryResult；如 #57 合并则 merge main 后对齐。Windows runner 当前离线，Windows CI 尚未运行，不能记通过。
