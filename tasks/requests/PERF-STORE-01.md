# PERF-STORE-01：core-store 测试为什么慢，从产品代码上提速

交付线。前提：#238（P5-38）已合并进 main。

- **分支：** `task/PERF-STORE-01`，从 `origin/main` 新建。
- **目录：**
  - `core/crates/core-store/src/`（产品代码）；
  - `core/crates/core-types/src/contract.rs`（只为缓存 `Schema::builtin()`）；
  - `core/crates/core-store/examples/`（计时探针，可新建）；
  - `tasks/requests/PERF-STORE-01.md`（本卡原文）；
  - `reports/PERF-STORE-01/`。
- **不碰：** 测试的断言、期限和阈值。不删测试，不加 `#[ignore]`，不缩小测试数据。不动契约 `.proto`，不加新的第三方包。
- **预授权：** 目录内的实现层决定由你自己做，在 PR 描述里逐条写「决定 / 理由」。如果某项提速必须改测试本身，或者必须改变存储格式、降低核验强度，停下来问。

**背景：**
- Windows 的 W02 组（core-store 测试，NTFS）每次要跑约 21 分钟：65 个库测试共 1418 秒，编译只占约 2%（见 #255 的报告）。Ubuntu 上的 core-store 测试也要 9 分钟左右，有时撞到 35 分钟的作业时限。
- 已知的一个热点：#240 实测发现，全量核验耗时里约 43% 花在反复调用 `Schema::builtin()`（`core-types/src/contract.rs`）重新解析 schema 上。

**1. 先测。**
1. 工具链是稳定版 1.99.0，不能用 `-Z` 参数。先用 `cargo test -p core-store --locked -- --list` 列出全部测试，再逐个按名称、`--exact` 运行并计时。在 Ubuntu 和 Windows 各做一次，列出最慢的 15 个测试和各自耗时。
2. 对最慢的 5 个做剖析：Windows 用 ETW，或者已装好的 `cargo flamegraph`；Ubuntu 用 `perf`。按函数列出时间占比，标明时间主要花在 CPU（解析、哈希、编解码）、fsync/磁盘，还是 SQLite。
3. 结论写进报告，作为改动前的基线。

**2. 再改**，只改产品代码，按测量结果取舍。候选如下，测量证明没用的就不做，在报告里写明原因：
1. `Schema::builtin()` 只解析一次，比如用 `std::sync::OnceLock` 缓存，之后共享只读结果；
2. 同一次核验里重复做的解码或哈希，结果能复用的就复用；
3. 测量中发现的其他热点。

不降低核验强度，不跳过任何检查，不改动磁盘上的格式。每项改动都要说明，为什么它不改变任何可观察的行为。

**3. 验收。**
- core-store 全部测试在 Ubuntu 和 Windows 上都通过，断言一条不改。
- 改前改后对照：最慢 15 个测试的耗时、整组总耗时、W02 步骤耗时，以及 #240 的两项计时（每个新头 149 ms、全量核验每个提交 138 ms）。
- 在 `examples/` 下写一个计时探针，覆盖各个热点，不用新的第三方包，方便以后发现性能回退。
- `cargo clippy`、`cargo fmt --check` 通过。

**交付：** 开 PR，正文按代码卡写：开头白话三行，然后不超过十行。前后对照表放在 `reports/PERF-STORE-01/REPORT.md`（含正式测量）。写「请审查」后停下，卡住就写 `BLOCKED.md` 然后停下。不要为了等别的 PR、CI 或回复让会话一直跑。

Windows 和 Ubuntu 各做独立前后对照，平台、修订、工具链与测试数明确标注，同平台比较须维持条件一致；若授权 Windows 上的现有 WSL 或项目既有 CI 无法提供正式 Ubuntu 测量，不自行改工作流、不改配额、不假称通过，照卡写 BLOCKED.md 后停。#249 的 69 测试 W02 1,321.81 秒与 #255 的 65 测试 1,418 秒是不同修订与集合，不能用来声称提速；以本卡同基线实测为准。#249 仍待审，不要在它或 Q3 的分支上工作，不越过卡列目录、不运行 GPU。
