# P0-10 阻塞：server 缺少 CUDA/全 offload 日志证据（2026-10-26，Perth）

本次完整窗口已停止，GPU 子进程与采样器均已结束，已通知用户恢复 runner。PR #87 仍为草稿。

已完成：Q4_K_M 注入配对进程与采样器退出 0，78 次请求、51 个输出位置的零调制/连续更新配对已离线整理，原始激活/logits/token 无损保留。尚未完成：速度、原/紧凑 IRL、Q8_0 及其显存。

阻塞：server 健康检查返回就绪，但固定 `--log-verbosity 3` 的日志没有 `loaded CUDA backend` 与 `offloaded N/N layers to GPU`，因此保守入口拒绝继续并退出 1。没有发正式 benchmark 请求；不能把缺日志推断成 CPU 回落或 CUDA 不支持。原始 server 日志、退出原因和显存时序全部保留。

已试：原定官方 b11669 server、Q4_K_M、全 offload 参数，失败一次后自动停止；没有改判据、重试或更换工具/模型。只读核对日志参数与已有源码，不运行引擎。

原因（静态核对）：本卡误用日志级别 3，而固定上游的 [common/log.cpp:515–530](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/common/log.cpp#L515-L530) 将底层 INFO 映射为 TRACE；[common/log.h:22–24](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/common/log.h#L22-L24) 中 TRACE 为 4，因而所需消息被屏蔽。建议仅改为 4、保留原 CUDA/全 offload 判据；修正后效果**未实测**。

具体待批准改动：[一行修正补丁](full-run-01/proposed-log-verbosity.patch)，尚未应用。其余参数、模型、引擎与判据不变；补测预计仍需约 1–2 小时安静窗口。

选项：①用户批准修正日志可观测配置，保持 CUDA/全 offload 判据，再另开安静窗口补测 benchmark（不重跑已完成注入）；②暂缓后续测量，保留当前结果。未经决定不改运行入口或重试，不能转为可审查。
