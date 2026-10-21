P4-02 提供三语 TTS 旁路服务、共享边界校验、离线降级、模型锁、本地/云端适配器。云端仅模拟，无真实请求或密钥。

**保持草稿和 needs-user；不合并。正式验收仍未完成。**

本次优化只改 services/voice-sidecar/ 与 reports/P4-02/。自有 bounded CUDA graph 使用静态 KV、与上游相同的注意力长度/连续布局，保留原采样/精度；predictor 全部单步、talker 每请求前 16 帧/总 48 个位置图、256-token 优化缓存，超范围继续上游 eager，生成上限仍 512。独立线程/持久 CUDA stream 解码，首块 2 帧，其余 4 帧，左上下文 25 帧不变。未新增 faster-qwen3-tts 依赖（其当前 0.5.4 与锁定栈不兼容）。候选显式启用，默认旧路径保留。

分段计时包括 prefill、talker、predictor 每步、每次解码、冷捕获、引擎/实际 WS 发送首末块。实时率仅作报告指标。两个变体的三语有界贪心 codec 与上游全部一致，同 2 帧分块的 PCM 逐样本相同；0.6B 511 帧完整序列 SHA-256 也相同。所有失败和满卡停滞证据保留。

| 开发短测（不是正式 p50） | 0.6B | 1.7B |
| --- | --- | --- |
| 中/英/日首块 ms | 233.205 / 196.403 / 223.672 | 269.548 / 249.451 / 210.670 |
| 首块到末块实时率 | 1.280 / 1.222 / 1.490 | 1.497 / 1.110 / 1.158 |
| ADR 0012 Nemu 合计显存 | 7.816 GB | 9.654 GB，超 8 GB |

另一个 0.6B 40.88 秒完整输出峰值 **8.089 GB，仍超线**；不能只报告短句符合。接缝数值/原始波形已归档，听感未签发。

用户批准的后续压力判据已登记且实现：torch allocated 首末不变、20–30 分钟 RSS 斜率 ≤12 MiB/小时；不满足延长到总 2 小时，报告趋势交用户判断，不追溯旧数据。优化后正式各语言 20 句 p50、WS 取消和压力仍需另约安静窗口，载体/渲染照常驻留，原阈值不变。

已 merge origin/main 806e09b（合并 d585926），随后 lint/typecheck、454 TS 测试、schema、ruff/mypy、35 Python 测试、Python 构建及原样 Electron e2e 3 项通过；1 GPU smoke 按默认条件明确跳过，另有真实开发测量。当前提交的远端 CI 状态以检查页为准，历史成功不冒充本次。

[REPORT.md](https://github.com/nanake-lab/nemu/blob/task/P4-02-tts-sidecar/reports/P4-02/REPORT.md) / [BLOCKED.md](https://github.com/nanake-lab/nemu/blob/task/P4-02-tts-sidecar/reports/P4-02/BLOCKED.md) / [开发摘要](https://github.com/nanake-lab/nemu/blob/task/P4-02-tts-sidecar/reports/P4-02/optimization-summary.json)。无合并，无下一张卡。
