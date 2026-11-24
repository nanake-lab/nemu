重档。本卡改动 core-store 对象核验，并要求 GitHub 托管 Ubuntu 和 Windows 的正式对照。

# PERF-STORE-01 对象核验与计时

对象存储现在只解析一次内置 schema，随后从四处共用同一份只读描述符。每次读取仍从磁盘取对象并核验。两次托管运行均已启动，尚无本轮加速幅度或验收结论。PR 保持草稿，交由 dot 跟进结果。[审查要求](https://github.com/nanake-lab/nemu/pull/265#issuecomment-7141513818)

## 被测版本和口径

| 运行 | 实际产品修订 | 用途 |
| --- | --- | --- |
| [仅基线 45485885498](https://github.com/nanake-lab/nemu/actions/runs/45485885498) | `6734adc2bfe8b145e8062a2fc669dd2024dd2ba6` | main 手动触发，candidate 留空。 |
| [同机对照 45486419758](https://github.com/nanake-lab/nemu/actions/runs/45486419758) | 上述基线 → `3952ffc82cb9d64f90ea4312276d37111b539901` | 每个平台在同一个作业内依次测两个修订，各三轮。提速应由这一运行内的同平台中位数计算。 |

两个运行使用 main 的 `2740b9e147a18e2b3c7734cee9a1527223f14164` 调度。工作流在每次 checkout 后核对实际 SHA、环境和 Rust 1.99.0；预编译后核对整包、库及文档测试名，逐项检查退出码、通过名称和数量，发现重编译即拒绝样本。整包与逐项均记录框架秒数和进程墙钟秒数。Ubuntu 对每个修订按框架耗时选最慢五项，用 perf 记录 CPU 栈。等待磁盘或 SQLite 锁的时间不能直接从 CPU 样本比例推算。[#269 工作流](../../.github/workflows/perf-store.yml)

基线和计时候选之间，`core/` 只有两个产品文件和一个 example 有差异。库测试目录的 Git tree 同为 `1424d339e714106bc0c693bd2bd7c164b0df0edf`，契约、黄金向量、依赖锁和工具链相同。Cargo 元数据中 example 的 `test` 与 `doctest` 均为 false。实际枚举结果由托管工作流在两版相等时才接受。[候选差异](https://github.com/nanake-lab/nemu/compare/6734adc2bfe8b145e8062a2fc669dd2024dd2ba6...3952ffc82cb9d64f90ea4312276d37111b539901)

main 自基线以来引入了存储发布、生成 schema、依赖锁等变化。先推送单独可测的优化提交，再 merge main。集成后两个产品文件和探针与计时候选的 blob 相同，工作流与 #269 的 blob 相同（`5141c63eaded8395013719a09b56fa24672d10b7`）。集成版本的正确性另看本 PR 当前提交的 CI。[PR 检查](https://github.com/nanake-lab/nemu/pull/265/checks)

## 改动与行为核对

| 路径 | 实现与核验 |
| --- | --- |
| [core-types](../../core/crates/core-types/src/contract.rs) | `Schema::shared_builtin` 用进程内 OnceLock 保存同一份编入二进制的 schema 的解析结果。失败时传回同样的错误文本。已有 typed codec 的缓存仍服务于 WireMessage。两条路径各自最多持有一份解析图。返回独立可修改描述符的 `Schema::builtin` 保留给现有调用方及[冷、热、无缓存的黄金对照](../../core/crates/core-types/src/lib.rs)。 |
| [对象核验](../../core/crates/core-store/src/objects.rs) | 替换四处取 schema 的调用：构造引用、识别 NSI1 索引、验证重组或普通对象、遍历引用闭包。对象字节、摘要、解码结果和闭包均逐次计算，原有读盘、长度、深度、类型、摘要、语义、同步和拒绝顺序相同。I16/I18/I21 对应既有[存储映射](../../docs/traceability/P5-03.md)及[损坏对象反例](../../core/crates/core-store/src/tests/reader.rs)。 |
| [计时探针](../../core/crates/core-store/examples/object_hotspots.rs) | 通过 `TemporaryRoot` 新建合成谱系，从公开入口 `Store::prepare` 和 `prepare_r` 分别计时首次写入与复用。带实际引用的 NSI1 索引覆盖索引识别、对象摘要检查和闭包遍历。输出逐样本纳秒及对象引用一致性检查。探针只使用已有依赖。 |

在托管机的 `core/` 下，探针入口为 `cargo run --release --locked -p core-store --example object_hotspots`。做探针对照时两版须使用同一份 example 源码；它独立于新 accessor。正式工作流测量完整测试与单测，未调用此 example。

## 验收现状

| 项目 | 基线 | 候选 |
| --- | --- | --- |
| Ubuntu 全部通过数、整包及最慢 15 项中位数 | 已调度，待结果 | 已调度，待结果 |
| Windows 全部通过数、整包及最慢 15 项中位数 | 已调度，待结果 | 已调度，待结果 |
| Ubuntu 最慢 5 项的函数剖析与火焰图 | 已调度，待结果 | 已调度，待结果 |
| Windows W02 独立步骤耗时 | not_run | not_run |
| #240 的 128 笔 release 新 head 核验、完整核验每提交耗时 | not_run | not_run |
| 对象探针实跑 | not_run | not_run |

历史 Windows 夹具中，独立 schema 解析占完整核验的 43.4%，新 head 为 149.324 ms，完整核验为 138.20 ms/笔。这些值只用于选择本轮改动位置。托管串行整包耗时与 Windows W02 步骤的命令及计时边界也有区别。[#240 原始口径](../FIX-SYS01-01/REPORT.md)

本机轻检查通过：Rust 1.99.0 工作区 `cargo fmt --all -- --check`（候选及合入 main 后）、`cargo metadata --locked --offline --no-deps`、差异空白及 Markdown 相对链接检查。未在本机或自托管机测量。当前 head 的 clippy、运行时黄金对照、core-store 测试结果及探针运行尚未核实。待两次 Actions 产生样本后，由 dot 核对实际修订、清单、平台及三轮完整性，补正式结果、失败和证据仓库索引。Windows 若实际超时，保留部分样本并记录停止点。
