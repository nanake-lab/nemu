# 2026-10-26 最小载体实验

仅完成 Q4_K_M 的最小可行性实验，不是完整 P0-10 验收。用户明确确认停 runner；入口和每次请求/解码前检查均未见 Listener/Worker。有效 GPU 运行结束于 Perth 00:28:52，进程与采样器均退出 0，随后已通知用户恢复 runner。

## 固定配置与覆盖

- 模型修订 `cdbee75f17c01a7cc42f958dc650907174af0554`，Q4_K_M SHA-256 `972e3afefe3abcfbc5b38c4dd1e5dcf7244bc3824203b67167b44f4468f129f2`。
- 官方 llama.cpp b11669 / `620d2225cc436ab0b001e3200e590e3b286ef9ab`，CUDA 12.4 包；RTX 3080 10 GB，驱动 610.88。日志确认 CUDA 后端和全部 37/37 层卸载；上下文 256，batch/ubatch 256，CPU 线程 4，Flash Attention 关闭。
- 三条固定合成问候输入，各 7 个模式，共 21 请求。每次最多生成 8 token，实际各为 3 token（含 EOS）。不读真实对话、排练库或正式存储。
- native 零起算第 17 层整层输出 control vector；A/B 为单位基向量的 +1/−1，不是学得概念方向。prefill 按上游广播，未实现旧探针的最后位置掩码。
- 同一已加载模型；每种语言的前 6 请求复用同一上下文，顺序 plain→zero→A→B→clear→A；最后新建上下文运行 fresh-A。每次请求显式清空 KV。

## 实测

| 语言 | 零向量/清除向量与 plain 的末个 prefill logits | greedy token | A 重复 / fresh-A 的最大差 | A−plain 最大绝对差 | B−A 最大绝对差 |
| --- | --- | --- | --- | --- | --- |
| 中 | 两者逐字节相同，最大差 0 | 两者一致，含 EOS 共 3 token | 0 / 0 | 0.6668210029602051 | 0.6772985458374023 |
| 英 | 两者逐字节相同，最大差 0 | 两者一致，含 EOS 共 3 token | 0 / 0 | 0.6424937248229980 | 0.5328326225280762 |
| 日 | 两者逐字节相同，最大差 0 | 两者一致，含 EOS 共 3 token | 0 / 0 | 0.7970752716064453 | 0.6693656444549561 |

18 次向量 API 设置/清除的返回码均为 0。这个配置下，**显式清 KV 后，库接口可逐请求更新向量且与对应 fresh 上下文一致**。没有声称引擎会自动识别 m 的变化或自动清 KV，也没有测并发请求、server 动态配置或热切模型。

零调制结论只覆盖这里的三条短输入、末个 prefill 的完整 logits 与实际生成的 3 token。每个 prefill 位置、每个 decode 位置的完整 logits、长序列和 Q8_0 **未实测**。

只载体的 WDDM 观测：38 行采样，33 行匹配 native PID；专用峰值 3,196,669,952 bytes、共享峰值 100,663,296 bytes，各自峰值相加为 **3,297,333,248 bytes（3.297 GB）**。这是短输入、至多两个小上下文的采样上界，不是连续峰值，也不是 GPT-SoVITS/渲染同驻或完整 Nemu 显存通过证明。没有把第三方约 3.5 GB 估值相加。

请求计时仅为本探针的 prefill wall，含同步/CPU 读回/有限性检查；输入和观测器不同于 P0-07/P0-08，不能当作公平速度对比或完整首句延迟。

## 原始证据与失败保留

- [有效 GPU 数据](smoke-gpu-03/raw/)、[启动绑定与 WDDM 时序](smoke-gpu-03/metadata/)、[重算摘要](smoke-gpu-03/summary.json)、[机械哈希复核](smoke-gpu-03/raw-recheck.json)。21 份 raw float32 logits 每份 607,744 bytes；全部哈希匹配。
- [run-01 CPU fallback](smoke-cpu-fallback-01/)：探针未配置进程内 DLL 搜索目录，实际仅 CPU 后端，不能算 GPU 结果；原始数值和源码全部保留。
- [run-02 绑定失败](smoke-api-binding-02/)：将 `ggml_backend_dev_type/name` 绑定到错误 DLL，native 退出 1；离线导出表核对后修正为 ggml-base.dll。该轮未加载模型，不作引擎性能失败结论。
- [编译过程记录](preparation/smoke-build/)：第一次命令行引用错误保留；同一 MSVC 19.44 工具链以 `/W4 /WX /O2 /utf-8` 编译通过。没有换模型、库或放宽验收。
- 两处探针修正仅在本卡目录，DLL 搜索配置只作用于当前进程。没有改系统/用户设置。

## 尚未实测

1. 同一引擎、同向量、同掩码下的旧 attention 残差位置 vs 新整层位置；目前只有源码位置差的证据。
2. 新位置的方向重提取及三语行为/能力验证。
3. 两量化同一批输入的 prefill、TTFT、decode token/s；200 条原 IRL schema 和紧凑 schema。
4. Q8_0 的注入路径、长上下文，以及 TTS/渲染同驻。
