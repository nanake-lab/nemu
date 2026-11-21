# FIX-SYS01-01 网关回执与存储增量核验

## 结果

新 head、已发布上传对象和运行时读取已接入同一套增量核验。Windows 正式验收默认并发 20/20 轮、单线程 5/5 轮通过，每轮 47 项。远端验收按本次推送的提交核对，运行链接登记在 [PR #240](https://github.com/nanake-lab/nemu/pull/240)。

## 规格与实现

《系统总方案》§29 SYS01 区分输入持久化与核心提交。§30 允许在热路径做局部充分检查。发出 `CORE_COMMITTED` 前，网关核对实际观察到的提交 ID、已发布快照、事件、分支和原始输入位置，再将其与等待中的连接、请求及稳定键配对。对象可见性本身不能证明事务已提交。I18 要求缓存不改变权威结果，I21 要求读取不回写 R 或 θ。[系统总方案](../../docs/echo-core/echo-core-system-plan-v1.0.md) · [P5-03 映射](../../docs/traceability/P5-03.md) · [交付检查](../../core/crates/gateway/src/progress.rs)

| 入口 | 核验与失效条件 |
| --- | --- |
| 网关 progress、uploads 的已发布对象回退、core-runtime 的 read_state | 三者共用当前 Store 打开实例的前缀。写连接的 `data_version`、head 摘要、位置和锚点一致时复用。运行时从同一个 reader 读取快照与 x、R、θ |
| 新 head | 从已核验的 D、提交和拒绝审计位置向后重放。沿用父提交、锚点、快照和已见稳定键指纹。读取并核对新增事务及其传递引用和账本对象的实际字节，重建新增派生行，与 SQL 索引和新增见证核对。保留 SQLite 完整性和外键检查 |
| 独立 Reader、打开或重开 Store、另一连接写入、无法证明直接延伸 | 完整检查提交链、对象及派生表和见证。每个 Store 持有独立缓存。工作线程返回后，写者重新核对打开身份、发布位置和写连接版本，才接纳结果 |
| 服务启动与定期检查 | 启动时完整核验一次。此后用单调时钟累计服务运行时间，默认满 24 小时后，在没有待处理 D 及待发回执时调度完整核验。用户可在启动前设置环境变量 `NEMU_STORE_VERIFY_INTERVAL_SECONDS` 为正整数秒数。例如，在 PowerShell 执行 `$env:NEMU_STORE_VERIFY_INTERVAL_SECONDS = "21600"` 可改为六小时。嵌入方也可使用 `Gateway::set_verification_interval`。全量和增量共用阻塞线程池，每次只跑一项。检测到损坏时服务返回存储错误、关闭连接并停止发回执 |

已核验前缀的派生投影保存在内存中。每次扩展都从规范对象重建新行，并比较两端各自上一次校验之后追加的行。可变的单行 head 每次都完整比较。服务循环处理认证和查询，需要核验前缀的事件按连接保持顺序。可以独立完成的查询照常返回。确实要读取待核前缀的查询随核验结果继续。每次对象读取仍核对内容摘要。[Store 读取](../../core/crates/core-store/src/reader.rs) · [派生投影](../../core/crates/core-store/src/derived.rs) · [恢复检查](../../core/crates/core-store/src/recovery.rs) · [事件循环](../../core/crates/gateway/src/gateway.rs)

## 反例与测试边界

- 破坏新提交引用的状态对象，增量核验拒绝。另一连接提交 SQL 写入，即使表与业务无关，也触发完整核验。破坏仅属于旧提交的快照后，新提交仍能独立验证。读取旧快照、用旧快照验证回执以及下一次完整核验均失败。另有原生命名管道测试验证后台检查发现损坏后服务终止。[存储测试](../../core/crates/core-store/src/tests/reader.rs) · [空闲核验测试](../../core/crates/gateway/src/tests/service/idle.rs)
- 原“旧对象篡改必须在下一个 head 时发现”测试已按审查更正替换。§30 要求证明本次事务所用的对象。冷历史由读时的摘要检查和定期完整核验覆盖，审查方已撤回文件监视范围请求。[审查更正](https://github.com/nanake-lab/nemu/pull/240#issuecomment-7058721543)
- 上传冲突和预算测试使用 10 秒夹具生命周期，使预算不足和对象声明冲突各自在活动预留上发生。独立过期测试保留 100 毫秒的生命周期，等待 150 毫秒后断言完整预留可重新使用。100 毫秒由夹具指定，拆分保留了原有预算、冲突、断连、释放断言及产品期限。[上传测试](../../core/crates/gateway/src/tests/service/adversarial.rs)
- 启动完整核验暂停期间，另一条管道可以认证、查询。在新提交核验暂停期间，提交回执须继续等待，另一个客户端仍可读到查询结果。[管道并发测试](../../core/crates/gateway/src/tests/service/acceptance.rs)

## Windows 验收

Windows 10 x64，AMD Ryzen 7 3700X，Rust 1.99.0。全部用例使用临时合成谱系。下表的耗时为 Cargo 报告的整包测试时间，每轮墙钟与断言结果另有逐轮记录。[25 轮明细](SYS01/incremental-runs.tsv)

| 检查 | 结果 |
| --- | --- |
| `cargo test -p gateway`，默认并发 | 20/20 轮，每轮 47/47，用时 19.90–21.24 秒，中位 20.32 秒。DIAG 基线 0/10 |
| `cargo test -p gateway -- --test-threads=1` | 5/5 轮，每轮 47/47，用时 97.29–102.93 秒，中位 99.82 秒。DIAG 基线 10/10 |
| 存储与运行时 | 开发期间全量 core-store 库测试 65 项通过，reader 专项 9 项通过。core-runtime 库测试通过 |
| 最终静态检查 | 三个受影响包的 `cargo clippy --all-targets -- -D warnings`、工作区 Rustfmt、Git 差异检查通过 |
| 远端当前提交 | Ubuntu CI 覆盖工作区，Windows 手动工作流在任务分支提交上运行 W02 与 W18。运行号和结论见 [PR 检查与交付记录](https://github.com/nanake-lab/nemu/pull/240) |

带临时计时探针的默认并发整包中，从等待倒序请求 `r2` 到收到 `CORE_COMMITTED` 实测 **8.539627 秒**，距原 10 秒期限剩 **1.460373 秒**。修复前整包探针为 13.149 秒并超时，上轮为 9.364 秒。本次探针在正式 25 轮前移除。这三个数是各版本的单次观测。[原始诊断](../DIAG-SYS01-01/REPORT.md) · [上轮报告](https://github.com/nanake-lab/nemu/blob/42fc9658fc57ab9e9f52b6cf859ef5c28fe16361/reports/FIX-SYS01-01/REPORT.md)

## 规模与耗时

每个规模单独计时一次。在同一临时库中先核验第 n−1 笔，再写入第 n 个 D 并提交。使用独立的长期 reader 构造夹具。计时段分别走真实新 head 回执入口、空上传缓存的已发布 UserText 回退入口，以及 release 的强制完整核验入口。下表单位为毫秒。Debug 测量与另一项 Windows CI 作业有重叠，release 测量于该作业启动前结束。

| 已提交笔数 | Debug 新 head | Debug 上传回退 | Release 新 head | Release 上传回退 |
| ---: | ---: | ---: | ---: | ---: |
| 2 | 813.812 | 11.457 | 141.354 | 2.533 |
| 8 | 795.065 | 11.864 | 155.281 | 2.584 |
| 32 | 910.217 | 12.511 | 149.013 | 2.838 |
| 128 | 791.633 | 11.386 | 149.324 | 2.685 |

上一版 debug 的 128 笔新 head 和上传回退分别耗时 97,865 和 96,793 毫秒。本版四个测点中，release 新 head 为 141–155 毫秒，上传回退为 2.5–2.9 毫秒。增量核验仍会执行 SQLite 页面与外键检查，新增事务引用的对象数量也会影响耗时。

### Release 完整核验分段

| 笔数 | 总计 ms | integrity_check | 对象文件读取 | canonical 解码 | 见证 | 内置 schema 解析 | 其余 | 每笔 ms |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 249.26 | 0.22 | 67.35 | 35.12 | 9.93 | 106.08 | 30.56 | 124.63 |
| 8 | 1,134.45 | 0.42 | 316.63 | 163.39 | 28.53 | 493.51 | 131.97 | 141.81 |
| 32 | 4,440.98 | 1.25 | 1,246.71 | 651.72 | 112.99 | 1,921.45 | 506.86 | 138.78 |
| 128 | 17,689.45 | 4.70 | 4,960.68 | 2,613.17 | 423.13 | 7,682.55 | 2,005.21 | 138.20 |

文件读取段包含路径安全检查、元数据与文件读取。canonical 解码只计最外层。见证段包含目录检查、捕获与核对，其中嵌套的读取和解码归入见证。内置 schema 段计独立执行的 `Schema::builtin()` 解析。余量包含其它 SQL、投影、摘要、语义检查和探针开销。五个分段按墙钟去重。

**后续项**　四个 release 测点的完整核验均超过 50 毫秒/笔。128 笔时，独立解析内置 schema 7,795 次，用时 7.68 秒（43.4%），对象文件读取 4.96 秒（28.0%），canonical 解码 2.61 秒（14.8%）。启动核验会随历史长度增长。后续可先测共享已解析的只读 schema 能节省多少启动时间，保持对象摘要、SQLite 和见证检查强度。

## 证据与交付范围

原始日志、测量用例、计时脚本和逐文件 SHA-256 已归档。归档 SHA-256 为 `1a7d9f818e8135ad931b6fdc48ad5f78f99dd956d178d9d25cebe64976ffdc92`。首次 release 夹具因探针模块声明位置编译失败，该次日志与修正后两个构建的成功记录均已保留。规模测量的分段探针仅写入缓存中的源码副本。[私有证据归档](https://github.com/nanake-lab/nemu-evidence/blob/6b0c6a96cf0276a6b55a7c40cf3b00fb5d9e50f8/FIX-SYS01-01/SYS01/incremental-windows.zip)

本轮使用 Windows 临时合成谱系，没有新增运行时依赖或改动存储格式。`store.rs` 原本超过 400 行，本轮在原有 Store 所有者上保存验证前缀。V4 及其三个新读取入口随 #238 集成。[P5-38](https://github.com/nanake-lab/nemu/pull/238) · [上一轮报告与探针](https://github.com/nanake-lab/nemu/blob/42fc9658fc57ab9e9f52b6cf859ef5c28fe16361/reports/FIX-SYS01-01/REPORT.md)
