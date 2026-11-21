# P5-40 — Windows W18 的 SYS01 回执读取超时

## 实测

| 被测提交 | 检查 | 结果 |
| --- | --- | --- |
| `bd750680c0242e8a04c751c40854182de5931bcd` | [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/44700279851) | 通过，包括 Rust 工作区、存储、三端契约。 |
| 同一提交 | [Windows 手动作业，第 1 次](https://github.com/nanake-lab/nemu/actions/runs/44700278197/attempts/1) | 桌面 E2E、视觉及证据来源检查通过。上传桌面证据时 GitHub Actions 返回 ECONNRESET，W02/W18 跳过。 |
| 同一提交 | [Windows 手动作业，第 2 次](https://github.com/nanake-lab/nemu/actions/runs/44700278197/attempts/2) | 桌面、证据上传、W02 通过；W18 为 41 通过、1 失败。 |
| 开工基线 `c56363617d13191deb55a66c3193bee0d5ceaeda` | [main 的 Windows 作业](https://github.com/nanake-lab/nemu/actions/runs/44697435088) | W18 同一用例、同一读取位置超时，也是 41 通过、1 失败。 |

失败用例为 `sys01_authenticated_upload_dispatch_reconnect_gamma_and_reverse_receipts`。等待旧版 TerminalResponse 时，`wire.read` 返回 `gateway deadline exceeded`。[回执读取位置](../../core/crates/gateway/src/tests/service.rs#L316)

基线已出现相同故障。日志只能定位到等待回执超时，无法判定网关没发、发送太晚还是调度或环境所致。P5-40 的提交只涉及契约及其验证接口，W18 的读路径和测试期限没有改动。

## 解除条件与选项

以 PR **当前提交**的 Windows W18 真实运行通过为解除本项的条件。最终检查与私有证据索引在 [PR #235](https://github.com/nanake-lab/nemu/pull/235) 的交付评论登记。保留所有失败记录。交叉审查及用户批准仍独立进行。

- 若最终 W18 仍失败，建议给 gateway 所属目录单列诊断，记录接收、提交和回执发送时间，查清缺失或超期的位置；若出现代码修复，按所属卡验证后再回到本 PR。
- 用户也可指定安静的 Windows 时间段做受控对照，保留原期限、完整作业及失败日志。一次复测能给出当次结论，不能证明间歇故障已经修复。
