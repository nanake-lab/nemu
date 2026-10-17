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
