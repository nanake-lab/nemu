# FIX-SYS01-01 默认并发第 9 轮阻塞

Windows 去除探针后的正式测试连续通过 8 轮，第 9 轮退出码 101，库测试 28.62 秒、40 过 2 败。任务要求默认并发 20/20，因此本次已经无法通过。默认组余下 11 轮、单线程 5 轮和 W18 均未运行。PR 保持草稿。[逐轮耗时](SYS01/runs.tsv)

| 第 9 轮失败 | 观测 |
| --- | --- |
| `sys01_authenticated_upload_dispatch_reconnect_gamma_and_reverse_receipts` | 共用读帧处报告 `gateway deadline exceeded`。本轮没有记录请求号和阶段，无法确认是哪次等待。[service.rs 第 316 行](../../core/crates/gateway/src/tests/service.rs#L316) |
| `upload_reservations_binding_expiry_and_disconnect_cleanup` | 更改同一上传预留的资源版本后，期望 `Some(OBJECT_MISMATCH)`，实际 `None`。用例设置预留期 100 毫秒，本轮未采集请求到达时的预留状态。[adversarial.rs 第 214 行](../../core/crates/gateway/src/tests/service/adversarial.rs#L214) |

同一 head 重复整体核验已消除。修前、修后各一次带计时的整包对照中，`r2` 分别为 13.149 秒超时、9.458 秒成功。新 head 的首次核验仍有历史扫描，修复后规模对照从 2 条时的 1.458 秒增至 8 条时的 5.789 秒。计数、重开、对象损坏、提交删除及前缀外拒绝均通过。正式第 9 轮原日志为归档内 `raw/default-09.log`。[报告及原始证据](REPORT.md)

## 待用户决定

1. **建议先在当前代码定位两项失败。** 保留原期限、断言和并发，在默认并发下记录每次等候的请求、处理阶段和上传预留的剩余时间，直至捕获失败。用一次新的 Windows 诊断窗口，区分剩余的提交或 head 核验耗时与其他等待。再次触发所需轮数未知。
2. 将上传预留单列为诊断卡，本卡只追 SYS01。这样两条路径可以独立定位，但整包 20/20 仍取决于两者，且涉及同一工作区的重负载需要串行。
