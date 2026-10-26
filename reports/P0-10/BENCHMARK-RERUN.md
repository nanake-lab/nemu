# P0-10 日志修正与 benchmark 补测（2026-10-26，Perth）

## 授权与原因

用户明确选 1，批准 server 日志级别 3→4，原判据不变，只补测两种量化的速度和 IRL，不重跑已完成注入；要求记录原因、参数、前后测量提交及所有异常。

前次实际运行源码为 **`072141f7112728ba05ebe70fe95de7468bb14221`**，成功注入/失败 server 的报告提交为 **`396fad7e92b3faf7c1d3541ae8601136c311feff`**。前次 server 健康就绪但无 CUDA/full-offload 消息，入口主动停止，正式 benchmark 样本 0；原始记录不删改。

固定 b11669 的 [common/log.cpp:515–530](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/common/log.cpp#L515-L530) 将底层 INFO 映射为 TRACE，而 TRACE=4；原 3 屏蔽了所需信息。此前是本卡准备脚本的日志配置问题，未据此判定引擎回落 CPU。

## 唯一运行参数修正

| 项目 | 前次 | 本次 |
| --- | --- | --- |
| server `--log-verbosity` | 3 | **4** |
| CUDA 后端 + `offloaded N/N layers to GPU` 证据 | 必须具备 | 原样保留 |
| 模型/修订、Q4_K_M/Q8_0、引擎二进制 | 固定库存/哈希 | 不变 |
| context/batch/threads/Flash Attention、cache、greedy 等 | 固定方案 | 不变 |
| jobs 与 schemas | full-v3 | full-v4 解压 jobs/schema **逐字节相同**；仅 http_probe.py 绑定摘要改变 |
| 注入实验 | 已完成 78 次 | **不重跑** |

新日志级别的日志开销包含在这次观测中，不能将跨量化/内核/观测配置的差异全部归因为引擎。

## 补测前检查与执行

14 个 Python 文件 Ruff/strict mypy 与 6 项既有 unittest 全通过。原始输出、输入一致性与重新准备计划见 [rerun-preparation-20261026/](rerun-preparation-20261026/)。没有新增依赖。

补测使用 `benchmark_native.py` 单独入口，按 Q4_K_M→Q8_0 串行，每量化 474 请求（含 4 条排除预热）。GPT-SoVITS V5 在限定缓存路径盘点仍未发现，按用户授权只测载体，无 TTS/渲染。GPU 前再次核实 Listener/Worker 均未运行；失败/异常保留，严重组件问题停止，不自行换方案。

## 本次实际提交与失败结果

实际运行提交：**`7ab742037a9203e9b5d671d8962598a3bdb0f311`**。前次实际运行提交为 **`072141f7112728ba05ebe70fe95de7468bb14221`**；前次结果记录在 `396fad7e92b3faf7c1d3541ae8601136c311feff`。完整启动绑定及本次原始记录见 [benchmark-run-02/](benchmark-run-02/)。

Perth 2026-10-26 **12:59:48–12:59:59**，无 Listener/Worker 后启动。日志确认 verbosity=4、`using device CUDA0 (NVIDIA GeForce RTX 3080)`、CUDA0 model/KV/compute buffers，以及 `offloaded 37/37 layers to GPU`；但没有原入口要求的 `loaded CUDA backend` 行，因此仍按原条件拒绝，不自行放行。父进程退出 1；server 由入口主动结束，退出 1；采样器退出 0。**正式速度/IRL 样本 0，Q8_0 未运行，注入未重跑。** 不能将本次描述为 CPU 回落或模型崩溃。

显存采样只有 1 行、匹配本进程 PID 为 0。离线函数的零值是空集合汇总，**不是 0 字节显存实测**；本次启动显存峰值未测得，不能用于显存验收。

进一步静态核对发现，命令行中 `-ngl 999` 在日志参数之前；[arg.cpp:2802–2816](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/common/arg.cpp#L2802-L2816) 的处理会调用 `llama_supports_gpu_offload`，[llama.cpp:95–102](https://github.com/ggml-org/llama.cpp/blob/620d2225cc436ab0b001e3200e590e3b286ef9ab/src/llama.cpp#L95-L102) 在后端尚未注册时会加载后端。CLI 按顺序处理，日志 4 尚未生效时该消息就可能被过滤。只修改值未覆盖此顺序，是本卡准备不完整；静态调用链与缺行现象相符，尚未实测调整顺序后的结果。

已备好 [仅调整顺序的补丁](benchmark-run-02/proposed-log-order.patch)：将现有 `--log-verbosity 4` 放到首个选项，保留所有参数值及全部条件判断。AST 比较确认参数 token 多重集、所有 if 条件均不变；该补丁**未应用**。按阻塞即停要求，不再自行尝试。
