重档。本卡涉及 core-store 的已发布历史读取及 Windows SYS01 验收。

# FIX-SYS01-01 网关提交回执

网关现在把已发布 head 的核验、逐笔提交与对象闭包核对放到阻塞线程，另一条连接可以在此期间认证和读取。原生管道回归在完整核验已固定 SQL 快照时暂停工作线程，第二条连接在原期限内读到查询回复。恢复核验后第一条连接才收到提交回执。新 head 的增量候选漏检了一处旧快照文件改写，未进入交付分支。默认并发第 2 轮仍然超时，PR 保持草稿并等待决定。[阻塞明细](BLOCKED.md)

## 发送 CORE_COMMITTED 的条件

《系统总方案》§29 SYS01 要求区分输入持久化和核心提交。§30 允许在热路径执行局部充分检查。I18 要求缓存命中与否不改变权威结果，I21 要求读取不回写 R 或 θ。网关须证明该事务属于本 Store、本分支的已发布前缀，快照、H 记录、事件和对象引用的内容摘要吻合，绑定指向实际观察到的提交，且请求号、事件和稳定键仍与等待中的连接对应。持久输入回执可先返回，核心提交回执须等上述证明完成。[原始规格](../../docs/echo-core/echo-core-system-plan-v1.0.md) · [P5-03 映射](../../docs/traceability/P5-03.md) · [P5-02 读取快照与见证次序](../../docs/traceability/P5-02.md)

同一时刻最多运行一个发布核验任务。Store 写者留在原处。工作线程持有只读句柄和已捕获的发布身份，核验完成后，发送方再次比对 Store 打开身份、head 摘要、处理位置、锚点和写连接的 `data_version`。如果检查结束时发布身份已变化，网关不发本批回执并返回错误。只有仍在等待的连接能收到回执。[读取作业](../../core/crates/core-store/src/reader.rs) · [核验与转发](../../core/crates/gateway/src/progress.rs) · [服务循环](../../core/crates/gateway/src/gateway.rs)

## 新 head 及各读取入口

| 入口 | 当前分支的行为 |
| --- | --- |
| gateway/progress | 同一已核验 head 复用 reader。新 head 的完整核验和逐回执验证在 `spawn_blocking` 中执行 |
| gateway/uploads | 当前连接已完成的上传从内存读取。已发布对象的回退读取仍调用 `store.reader()` 并做完整核验，接入增量缓存待解阻 |
| core-runtime/read_state | 仍通过 `store.reader()` 完整核验后读取 x、R、θ，接入增量缓存待解阻 |
| 独立 Reader、Store 打开与重开 | 保留各自的完整恢复、见证检查 |

增量实验保存 D 与提交前缀位置、父提交、锚点、快照、稳定键及其指纹、审计和索引计数，再检查新增记录和派生表。历史合法时首笔完整核验通过。正常追加后只改旧快照的一个字节，候选接受、全量拒绝。该缓存缺少可观测的文件失效条件。把它用于另外两个读取入口会扩大漏检范围。[实测与范围请求](../../tasks/requests/FIX-SYS01-01-file-invalidation.md)

## Windows 验收

| 检查 | 结果 |
| --- | --- |
| `cargo test -p gateway` 默认并发，要求 20/20 | 首轮 43/43。第 2 轮 42/43，SYS01 倒序回执用例在共用的读帧处超时。按失败停组，余下 18 轮未运行 |
| `cargo test -p gateway -- --test-threads=1`，要求 5/5 | 5/5，每轮 43/43 |
| `cargo test -p core-store tests::reader::` | 8 过、0 败。包含旧/新对象损坏，并检查后台结果返回前的另一连接写入、换 head、同路径重开 |
| 两包全目标 Clippy、Rustfmt 与 Git 差异检查 | 通过。Clippy 使用 `--all-targets -- -D warnings` |
| 当前 head Ubuntu CI 与 Windows W18 | 远端结果按提交核对，见 [PR 检查](https://github.com/nanake-lab/nemu/pull/240/checks)和该 head 的 PR 评论 |

每轮使用独立临时合成库、原预算和原断言。库测试及 Cargo 墙钟逐轮见[本轮结果](SYS01/review-runs.tsv)，[前轮结果](SYS01/runs.tsv)保留。DIAG-SYS01-01 的默认并发为 0/10，单线程为 10/10。[初始诊断](../DIAG-SYS01-01/REPORT.md)

## 请求延迟

一次带计时探针的默认并发整包中，`r2` 从开始读取到收到 `CORE_COMMITTED` 为 **9.364 秒**，距离原 10 秒期限剩 **0.636 秒**。同机初始修复前的记录为 13.149 秒并超时，上一版同 head 复用 reader 时为 9.458 秒。各次都是独立样本。本轮正式第 2 轮没有逐请求计时，无法从共用读帧行判定请求号。[前一版测量与阶段分解](https://github.com/nanake-lab/nemu/blob/53965ca1e5fca973e9879ffc94eedd91f949b4ef/reports/FIX-SYS01-01/REPORT.md)

上传探针在同步 main 后、回执线程改动前复现了原断言的失配。竞争请求进入时，活动预留年龄 94.477 毫秒。服务端于 228.013 毫秒清除它。改声明的请求到达时活动表中已没有该预留。该请求按新 ID 的偏移 0 分块受理。正式 100 毫秒的期限与断言都保留。[上传预留诊断](BLOCKED.md)

## 规模测量

在同一个 Windows 临时库内逐笔追加。每个测点先核验第 n−1 笔，再测新 head。以空上传缓存读取同一个已发布的 UserText，强制走 uploads 的存储回退入口。每项各一次。夹具构造位于计时区间外。

| 提交数 | 新 head 核验（毫秒） | 读取已发布上传对象（毫秒） | 两次 integrity_check（毫秒） |
| ---: | ---: | ---: | ---: |
| 2 | 1,440 | 1,387 | 0.511 / 0.308 |
| 8 | 6,028 | 6,018 | 0.644 / 0.674 |
| 32 | 23,345 | 23,496 | 1.909 / 1.940 |
| 128 | 97,865 | 96,793 | 8.457 / 7.669 |

两处在当前分支均执行完整恢复，实测耗时随历史增长。这些数值是当前保守实现的基线。SQLite 页面完整性检查仍按原强度执行。

## 限制与证据

此轮的全量恢复仍包含 `PRAGMA integrity_check`、外键检查、提交链、对象闭包、派生索引及独立见证核对。缓存失效探测、增量结果与完整结果在所有损坏用例上的一致性，以及三个入口不随历史增长的耗时，仍是交付阻塞。当前主线为 V2，V4 随 [P5-38](https://github.com/nanake-lab/nemu/pull/238) 集成。没有新增生产依赖、消息或存储格式。

本轮原始日志、前缀损坏、上传预留与耗时探针、规模夹具和逐文件 SHA-256 已归档。文件为 `FIX-SYS01-01/SYS01/review-windows.zip`，SHA-256 为 `19e60ccc145c3eade33ee8626b53cb216205fa406d147712b4fca734fbdd345c`，证据提交为 `f61902b63c2fc02f8336a3a58635172a45c63c74`。[私有证据归档](https://github.com/nanake-lab/nemu-evidence/blob/f61902b63c2fc02f8336a3a58635172a45c63c74/FIX-SYS01-01/SYS01/review-windows.zip)
