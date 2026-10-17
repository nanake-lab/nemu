未选定载体：真实权重与测量尚未完成；当前仅试验 attention 残差后、下一归一化前的单点注入，rank=8，量化方式待实测。

本记录不是选型批准。候选清单见 `workers/model-adapter/probe/manifest.json`，测量状态见 `benchmark.md`。尚不修改 `config/models.lock.json`。

没有满足任务书全部条件的证据。CPU 结构夹具通过和硬件渲染就绪不能替代真实载体、TTS、渲染并载及三语观察。下载链路失败也不能用作“模型不满足”的结论。

## 2026-10-17 追加决定状态

仍未选定载体。完整十项矩阵已测，全部二十字夹具 p50 >600 ms；三款 4B 级最快为 Qwen3-4B int4（662.2309000013047 ms）。候选通路使用中间层 attention 残差后、下一归一化前的注入，rank=8；这不是正式选型。config/models.lock.json 已按用户授权登记候选，保持 candidates-only。历史“尚不修改”描述仅代表当时状态。

详细数据、显存归因和数值容差限制见 benchmark.md、REPORT.md、BLOCKED.md；方向观察继续执行，后续路线等待用户决定。
