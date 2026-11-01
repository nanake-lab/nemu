# P0-12 阻塞：官方权重下载中断

步骤 1 空间充足；步骤 2 下载失败，按用户本次“任一步失败即停，不自行换工具、引擎或方法重试”停止全部后续执行。

- 固定修订：`Qwen/Qwen3.5-4B@851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a`。
- 2026-11-01 23:26:41（Australia/Perth），curl 退出 56：`schannel: server closed abruptly (missing close_notify)`；见 [原始日志](preparation/download.log)。不据此推断服务器故障或本机 TLS 故障的根因。
- 已试：系统 curl 单次顺序下载，TLS 校验开启，无自动重试。6 个小文件完成上游摘要核验。
- 第一分片预期 5,329,398,688 bytes，仅收到 2,418,060,864 bytes；partial 位于 `D:\Dev\cache\P0-12\models\Qwen--Qwen3.5-4B\851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a\model.safetensors-00001-of-00002.safetensors.partial`。本地 SHA-256 只标识残片，不能认证完整权重，见 [manifest](manifest.json)。第二分片未下载。
- 转换、量化、mmproj、合成输入、注入测试与七项 GPU 测量均未执行。GPU 时长 0，已通知用户恢复 runner。

供用户决定，均未执行：

1. 授权用同一 curl、同一修订和同一 URL 尝试 Range 接续；须先验证服务器响应范围与现有残片一致，完整后再核对上游 SHA-256。预计剩余权重流量约 6.90 GB；可能仍遇网络中断。需要适配现有“目录存在即拒绝”的入口，不自动覆盖。
2. 授权在新缓存目录重新完整下载同一修订，保留本次失败残片；约 9.32 GB 权重流量，另多占约 2.42 GB 残片空间。
3. 暂停本卡，保留证据与缓存，待用户确认网络/下载方案后再安排。无需现在保留 GPU 安静窗口。

不提议或自动下载 Qwen3-VL-4B-Instruct；当前是下载阻塞，没有证据证明 Qwen3.5 在 b11669 上存在转换或运行硬性问题。
