# P0-07 测量记录（进行中）

尚未完成真实载体矩阵。以下工程夹具结果不能用于载体选型。

## 已执行的准备验证

| 检查 | 原始结果 | 范围与证据 |
| --- | --- | --- |
| CPU 机制单元测试 | 3 passed | `test_injection.py`；掩码、非零注入、非法输入、请求内冻结、缓存键所有分量失效 |
| 小型随机 Qwen3：零调制 prefill logits 最大差 | 0 | `preparation/tiny-cpu-path-v1.json`；仅 CPU float32 随机夹具 |
| 同上：零调制 decode logits 最大差 | 0 | 同上 |
| 同上：m 变化后旧 KV 清除 | true | 同上 |
| 同上：失效后与全新 prefill 的 logits 最大差 | 0 | 同上 |
| 同上：prefill 注入最大误差 | 3.725290298461914e-09 | 同上；保存实际 before/after/delta，未冻结生产容差 |
| 同上：decode 注入最大误差 | 1.30385160446167e-08 | 同上 |
| 渲染占位 GPU | NVIDIA GeForce RTX 3080 / D3D11 | `preparation/hardware-render-v2.json` |
| 渲染连续性 | 帧计数 1 → 13 | 同上；1 秒等待窗口的帧计数观测，不是 FPS 性能验收 |

探针当前结构夹具配置：单层 attention 残差相加后、post_attention_layernorm 前；rank=8；U 为单位矩阵前 8 列，算子范数恰为 1；试验 a=(0.5,0,…,0)，beta=1。这些是通路调试参数，不是 genesis 参数或最终载体配置。实际模型层号预设为总层数整除 2 减 1（零起算），必须随结果记录。prefill 只注入最后 token，后续每次单 token decode 注入；不支持批量或分块 prefill。

## 真实模型矩阵

| 模型 | bf16 | 8 bit | 4 bit | TTS + 渲染并载 | 三语方向检查 |
| --- | --- | --- | --- | --- | --- |
| Qwen3-4B | not_run | not_run | not_run | not_run | not_run |
| Qwen3-4B-Instruct-2507 | not_run | not_run | not_run | not_run | not_run |
| Qwen3-8B 对照 | 不属任务要求 | 不属任务要求 | not_run | not_run | 不替代 4B 候选检查 |

完整权重尚未就绪。原生 Hugging Face 下载出现 TLS unexpected EOF；元数据和 tokenizer 成功不代表权重下载成功。系统 curl 的 1 KB、8 MB Range 请求成功，现采用最多 4 路、每段 8 MiB 的续传，校验响应 Content-Range/长度，完整文件再核对固定修订的 LFS SHA-256。失败或部分文件不会改名为最终权重。含临时签名链接的下载日志保留在忽略的 `.cache/logs/`，不提交仓库。

## 尚待完成

真实模型通路、量化矩阵、TTS 常驻 GPU、并载总显存、20 字首句延迟、tokens/s、三语各 20 次方向观察，以及一条命令复现全矩阵。600 ms 和 9 GB 门槛不变；不跳过或替代验收。

## 2026-10-16 候选范围追加

新增 microsoft/Phi-4-mini-instruct（3.8B，Phi3ForCausalLM，MIT）为第三个 4B 级候选。bf16、8 bit、4 bit、通路、TTS/渲染并载和三语方向检查全部为 not_run；当前 Qwen3 包装尚不能直接用于 Phi。核查来源、修订和其他候选筛选依据见 REPORT 的“候选架构扩展”。TTS Base 仅用于并载显存测量，正式音色和变体由 P4-03 决定。

## 2026-10-17 首组真实并载观察

| 配置 | 原始样本 | 首句 p50 | 实际字数 | 结论 |
| --- | --- | --- | --- | --- |
| Qwen3-4B int4 + TTS Base GPU 常驻 + 舞台硬件渲染 | preparation/qwen4b-int4-concurrent-v1.json | 800.3448 ms | 每条 14 汉字 | 非二十字验收；未选定 |

一次预热后 20 次 greedy 重复。原始文件另含 TTFT、token/s、CUDA allocator 峰值与全 GPU 离散样本；全 GPU 样本不是 Nemu 归因峰值。

## 2026-10-17 矩阵 v1 已执行（追加，不替代历史）

原始数据在 `matrix-v1/<模型-精度>/path.json`、`concurrent.json`、`render.json` 及同目录日志。每配置独立进程，真实 TTS Base 常驻与舞台硬件渲染并载；三个 4B 级候选各三种精度，另有 8B int4 对照。十项执行/清理退出码均为 0，不等于验收通过。

本轮使用固定二十汉字复制夹具、greedy、eager、一次预热后 20 个独立 KV 请求。它是受控生成性能测量，不代表自由回答质量。失败/超时样本不删除，p50 使用全部非预热样本。

| 模型 | 精度 | 全部非预热首句/停止耗时 p50 (ms) | 完成二十字并抵达句末 / 20 |
| --- | --- | ---: | ---: |
| Qwen3-4B | bf16 | 2913.529250 | 19 |
| Qwen3-4B | int8 | 2698.864300 | 20 |
| Qwen3-4B | int4 | 662.230900 | 20 |
| Qwen3-4B-Instruct-2507 | bf16 | 5274.476600 | 19 |
| Qwen3-4B-Instruct-2507 | int8 | 9473.569100 | 20 |
| Qwen3-4B-Instruct-2507 | int4 | 666.255450 | 20 |
| Phi-4-mini-instruct | bf16 | 914.656000 | 20 |
| Phi-4-mini-instruct | int8 | 1964.243800 | 20 |
| Phi-4-mini-instruct | int4 | 673.194850 | 20 |
| Qwen3-8B（对照） | int4 | 658.872850 | 20 |

主代理结论：本轮所测配置没有满足 600 ms 条件；不选定载体。不能把这项结论扩展成这些模型在所有实现/配置下都无法达标。int8 出现反复的 dtype 转换日志，本轮时间包含其开销。bf16 存在 WDDM 共享 GPU 内存现象；一个瞬时观察已保存，后续捕获时进程已退出的空结果也保留。全 GPU 离散读数不是 Nemu 精确峰值。

机械性重算由新增 Luna/max 子代理执行，主代理复核，原始核查日志位于 `../agent-delegation/2026-10-17/trial-v3.stdout.log`。较早子代理脚本猜测字段得到的错误完成数已拒绝采用。三语方向检查仍在继续，不能提前签发完整 P0-07 完成结论。

## 2026-10-17 三语观察已执行（追加）

原始数据 direction-v1/ 两款Qwen及 direction-v2/ Phi，每模型每语言20条（10个匹配seed的零/正调制）。Phi v1因遗漏第二个EOS而继续生成，保留作故障证据；v2修复后60条均以EOS结束。方向变化和质量限制见REPORT，不能将“token变化”当作预期语义方向通过。延迟门槛仍未满足，生产数值容差及精确显存归因未闭合，保持未选定。

## 2026-10-18 int4 优化对照（追加）

全部使用固定二十字夹具、TTS GPU驻留、RTX3080硬件舞台、1次预热+20次独立KV；每组20/20匹配。路径/并载/渲染均退出0。

| Qwen3-4B int4 配置 | 首句 p50 (ms) | 原始目录 |
| --- | ---: | --- |
| eager 参考 v1 | 658.946050 | opt-reference-qwen4-v1 |
| SDPA | 653.339250 | opt-sdpa-qwen4-v1 |
| SDPA + 请求内Ua缓存/末位置logits | 655.679100 | opt-sdpa-lean-qwen4-v1 |
| eager + 原生RMSNorm | 538.875850 | opt-native-rms-qwen4-v1 |
| 同时段 eager 参考 v2 | 654.718350 | opt-reference-qwen4-v2 |

原生RMS单项相对随后参考v2约降低17.7%，首次达到延迟线；未据此完成全部选型验收。该路径145个RMSNorm被替换；本次5-token夹具中，内核替换前后prefill/decode最大logits误差均0，零调制/注入/缓存对照误差也为0，缓存失效true。该零误差只覆盖已测夹具，不宣称任意输入逐位一致。SDPA轨迹实际落到math实现，不能把API名称当作融合成功。

## 2026-10-18 原生RMS完整入口数据（追加）

原始根目录 opt-native-rms-all-v1；全部矩阵与三语方向子进程exit0。默认eager、rank8、原位置/掩码、int4 NF4不变，只有载体RMS采用现有PyTorch原生实现。

| 模型 | 全部20个非预热样本p50 ms | 完整固定句/20 | 最大整卡采样 MiB | torch reserved峰值 bytes |
| --- | ---: | ---: | ---: | ---: |
| Qwen3-4B | 546.9558999975561 | 20 | 6637 | 5263851520 |
| Qwen3-4B-Instruct-2507 | 540.7333999974071 | 20 | 6635 | 5261754368 |
| Phi-4-mini-instruct | 593.8205999991624 | 20 | 6837 | 5463080960 |
| Qwen3-8B | 542.6337500030058 | 20 | 9867 | 8797552640 |

所有相同内核零调制/缓存比较误差0，缓存失效true；8B预填充注入误差0.001953125，其余注入误差0。Instruct内核替换对原模型logits误差prefill0.234375/decode0.125，其余本夹具为0。全卡样本与torch reserved不是同一口径，不能合并或当作Nemu精确峰值。

三语各20条、各10对，token变化配对数：Qwen4 5/2/3；Instruct 5/3/2；Phi 6/2/6（zh/en/ja）。180条均EOS且无EOS后token。主代理语义阅读、质量问题及验收范围见REPORT末节。
