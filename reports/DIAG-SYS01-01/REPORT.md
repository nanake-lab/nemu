轻档（最终差异为任务卡、报告和逐轮结果表）。

# DIAG-SYS01-01 · Windows 上的 SYS01 并发超时

**诊断完成。默认并发 10 轮全部在同一个 SYS01 用例读取回执时超时，单线程 10 轮全部通过。** 补充计时捕获的超时发生在第二个事件 `r2` 的 `CORE_COMMITTED` 回执。测试要求两次真实提交后倒序回执，服务在同一执行线程上同步完成提交和发布校验，并发时客户端的 10 秒读帧期限先到。两个 `sys01_pipe` 用例在全部 20 轮均通过。交付为诊断，修复后验证轮数为 0。

## Windows 复现

基线为 `c5636361`，运行时任务分支为 `a1643db6`，两者的 Rust 代码一致。本机为 Windows 10 x64 build 19045，16 个逻辑处理器，Cargo 和 rustc 均为 1.99.0，使用调试测试构建、NTFS 合成临时库。每轮单独启动 Cargo，默认组未设置 `RUST_TEST_THREADS`。下表耗时是 42 项库测试的耗时。

| 命令 | 整轮通过 | 42 项库测试结果 | 每轮耗时 |
| --- | ---: | --- | ---: |
| `cargo test -p gateway` | 0/10 | 第 1 轮 40 过、2 败，其余九轮均为 41 过、1 败 | 27.43–32.51 秒 |
| `cargo test -p gateway -- --test-threads=1` | 10/10 | 每轮 42 过，Cargo 均正常退出 | 123.33–124.78 秒 |

默认组十次 SYS01 失败均在 `Peer::reply` 读取回执时返回 `gateway deadline exceeded`。第 1 轮另有 `upload_reservations_binding_expiry_and_disconnect_cleanup` 失败，预期 `Some(OBJECT_MISMATCH)`，实际为 `None`。这个用例设置了 100 毫秒上传预留期限；该轮没有记录服务端处理失配请求时的预留状态，无法给这次断言失败定因。[失败位置](../../core/crates/gateway/src/tests/service.rs#L316) · [上传预留断言](../../core/crates/gateway/src/tests/service/adversarial.rs#L214)

[逐轮结果](SYS01/runs.tsv)列有每轮用例数、耗时、退出码、失败用例及最后输出。首轮 Cargo 的 55.648 秒含编译。第 1 轮缺少独立的用例线程起点，SYS01 的 12.900 秒是相对套件开始的估算。第 2–10 轮可用 Windows 线程创建时间对齐失败输出，SYS01 耗时 12.024–16.145 秒。二十轮均未观测到活动的 Windows CI worker。其他本机 Rust 任务曾在部分轮次启动；默认组第 6、8、9、10 轮在采样期间都只有本轮 Cargo，仍复现同一超时。

同一基线的主分支 Windows CI 也以单线程在同一共用读取行超时，41 过、1 败，库测试耗时 131.92 秒。本机单线程的 10/10 表明它在本次条件下通过；CI 日志没有记录超时回执的请求号。[主分支 Windows CI](https://github.com/nanake-lab/nemu/actions/runs/44697435088)

## 回执为什么迟到

补充探针只在测试目录记录单调时钟。三个对照都保留原有期限、真实命名管道、两次存储提交、反序要求及全部断言。下表从客户端开始等待 `r2` 的提交回执计时。此时两条 `RECEIVED_DURABLE` 都已收到，第二条管道的认证查询也已成功。

| 对照（各 1 次） | 夹具向 gateway 交回两条提交结果 | 处理第二次提交的服务轮询交还执行权 | 客户端读取 `r2` |
| --- | ---: | ---: | --- |
| 整包默认并发 | 6.207 秒 | 10.374 秒 | 10.374 秒，超时 |
| 单独运行该用例 | 5.028 秒 | 8.953 秒 | 8.953 秒，收到 `CORE_COMMITTED`，随后收到 `r1` 的提交回执 |
| 整包默认并发，客户端独占另一线程和 Tokio 运行时 | 6.650 秒 | 11.229 秒 | 10.001 秒，超时 |

夹具扣住第一次真实提交的结果，直到第二次完成才按 `r2 → r1` 返回。接下来，`Gateway::progress` 在同一次轮询内同步核对已发布的 head，再为两条结果逐条打开 reader、核对事务并排入发送队列；管道写出由另一个异步任务执行。默认并发探针中，从夹具返回到这次轮询结束又经过 **4.167 秒**。独立客户端按时结束等待时，服务线程仍在这一轮询内。这解释了为什么单独给客户端分配线程仍会超时。该区间包含多种存储、校验及投递操作，本次只测了它们的合计。[反序夹具](../../core/crates/gateway/src/tests/service.rs#L94) · [发布核对与投递](../../core/crates/gateway/src/progress.rs#L35) · [读取时的存储核验](../../core/crates/core-store/src/reader.rs#L47) · [管道写出](../../core/crates/gateway/src/network.rs#L65)

## 共享资源排查

| 项目 | 结论与证据 |
| --- | --- |
| 固定名字、临时目录、端口 | 服务管道名含 SID、进程号和用例标签。`sys01_pipe` 用另一前缀和 `dedup`、`restart` 标签。每个用例单独创建随机临时根，两个 `"sys01"` 是各自根下的名字。传输走本地命名管道，无 TCP 端口。见 [服务夹具](../../core/crates/gateway/src/tests/service.rs#L181)、[结构测试](../../core/crates/gateway/src/tests/sys01_pipe.rs#L22)、[临时根](../../core/crates/core-store/src/lineage.rs#L34)。 |
| DPAPI、用户状态、全局数据 | 复用当前 Windows 用户的 DPAPI 身份，各用例创建自己的受保护令牌文件。开关、提交计数、反序队列属于各自的 `Core`。共享黄金向量初始化后只读。见 [令牌和夹具状态](../../core/crates/gateway/src/tests/service.rs#L190) 与 [黄金向量](../../core/crates/gateway/src/tests/mod.rs#L29)。 |
| 管道实例、ACL | 服务配置上限 4，同一管道名的系统创建上限设为 16，目标用例有两条已认证的连接。20 轮里逐实例 DACL 检查全部通过，计时对照也已完成第二连接查询。目标失败发生在上述连接成功后的回执读取。见 [管道创建](../../core/crates/gateway/src/windows.rs)、[ACL 用例](../../core/crates/gateway/src/tests/service/permissions.rs) 和 [第二连接查询](../../core/crates/gateway/src/tests/service/acceptance.rs#L96)。 |
| 计时与规格 | 客户端等待首字节即开始计算 10 秒期限；现有用例继续核对 durable/committed、反序回执、重投与冲突。对应 SYS01 的原始判据及既有映射见[施工总方案](../../docs/echo-core/echo-core-system-plan-v1.0.md#L643)与 [P5-03 映射](../../docs/traceability/P5-03.md#L16)。见[读帧实现](../../core/crates/gateway/src/framing.rs#L92)。 |

## 交付与后续

任务分支已移除临时探针，交付物为[任务卡原文](../../tasks/requests/DIAG-SYS01-01.md)、本报告和逐轮结果。未发现可通过命名或目录隔离消除的资源冲突。建议后续在 `gateway::progress` 和 `core-store::Reader::open_checked` 对发布核对、两次回执校验分别计时，评估在同一已发布状态下复用已校验 reader 的可行性，保留每条回执的真实性检查。本卡等待审查。

私有证据 `DIAG-SYS01-01/SYS01/windows-runs.zip` 收录 20 轮基线、3 次诊断对照、探针补丁、采集脚本和包内逐文件哈希。证据提交 `b467bf5367afbfa355dedb837d27f332dfe1ab61`，ZIP SHA-256 为 `e5c461bec631f6f684938da3a242356c0f6e1ed440e05be6d7bcd57173abeda9`。[私有原始证据](https://github.com/nanake-lab/nemu-evidence/blob/b467bf5367afbfa355dedb837d27f332dfe1ab61/DIAG-SYS01-01/SYS01/windows-runs.zip)
