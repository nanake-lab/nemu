# 本地恢复与转换记录

用户在2026-11-16明确允许保留桌面背景进程继续测量。此前已核实#205最新head `aa50f06230a5291be0d0d1f4cbe72e35e3438dff`，本地v4补跑结束。开始前nvidia-smi显示RTX 3080、0%利用率、864 MiB背景占用；Runner.Listener与Runner.Worker均不存在。Windows的C+G分类不能单凭列表证明计算繁忙，按用户许可保留桌面背景，不结束其他程序。各模型串行运行，测量前和逐条调用前检查runner窗口。

完整检验及其长历史部分已在API调整前冻结；本次再次核对摘要相同，没有重写。准备阶段的“未执行”状态已由本次恢复执行取代，旧准备说明保留其历史含义。

## Qwen3-8B转换

使用仓库登记的llama.cpp release b11669，提交`620d2225cc436ab0b001e3200e590e3b286ef9ab`。CPU转换与量化均退出0；转换约144秒，量化约162秒。模型原文件保留，生成的BF16中间文件也保留，产物都在原模型修订目录的`R-PROMPT-FIT-01/`下。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| carrier-BF16.gguf | 16388044192 | 159365704ecb4ba9775b3a527843122eddd8d2699c516709fdcdb63099e67250 |
| carrier-Q4_K_M.gguf | 5027784096 | 881ae79cd9ce76ef6764e5cffaee6895c2b53f2f1d464c5ded11c28d5a0b1602 |

实际逐参数命令、开始结束时间、退出码和工具摘要保存在私有`local-resume/conversion.json`；两个stdout日志保留原字节。命令等价写法如下，变量只用于显示路径：

```powershell
$modelRevision = 'D:/Dev/_leftover/2026-10-20/workers/model-adapter/probe/.cache/models/Qwen--Qwen3-8B/b968826d9c46dd6066d109eabc6255188de91218'
$fitOutput = Join-Path $modelRevision 'R-PROMPT-FIT-01'
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe -B D:/Dev/cache/P0-10/source/llama.cpp-b11669/convert_hf_to_gguf.py $modelRevision --outfile (Join-Path $fitOutput 'carrier-BF16.gguf.partial') --outtype bf16
# 成功后在同目录将partial重命名为carrier-BF16.gguf，保留该文件。
& D:/Dev/cache/P0-10/llama.cpp/b11669-cpu-quantize/llama-quantize.exe (Join-Path $fitOutput 'carrier-BF16.gguf') (Join-Path $fitOutput 'carrier-Q4_K_M.gguf.partial') Q4_K_M 4
# 成功后在同目录将partial重命名为carrier-Q4_K_M.gguf。
```

转换器SHA-256为`e9a1da876330bbce9687541ab31736542a01b4ac43c6686126514a50f122fb7f`，量化器为`7ec08d1251763e1842d8bf5e195ea71e7c08ea4f2da3f5c3517d70ff152d1abf`。推理前核对release引擎55个文件、模型、模板、tokenizer及执行器摘要；没有扫描其他模型位置。

产物目录另存`manifest.json`，来源、Apache-2.0许可证和可分发标志沿`config/models.lock.json`中该修订的登记；用途仅为本次测量，未登记为生产载体。其副本保存在私有`local-resume/converted-model-manifest.json`。

## 执行与判定边界

官方聊天模板显式传入`enable_thinking`。沿载体登记使用greedy：temperature=0、seed=731、repeat_penalty=1、presence/frequency penalty=0，因此每输入每写法只测一次，不声称获得随机重复的2/3证据。上下文8192，输出预算为8192减输入token再减1，思考和正文共享预算；完整长历史保留，不截断、不扩预算。单槽、逐条erase、cache_prompt=false；Qwen3.5-4B按载体登记同时加载mmproj，本次输入只有文字。

每写法两条技术冒烟均有效后才冻结并调整。正常传输但预算耗尽、思考未闭合或正文为空均记无效，保留原始流，不重试；其他写法和模型继续。正式调整中的无效项排除于语义分母，比较沿METHODS只取双方有效的同一输入。没有有效基线时无法判断改善；若仍有可用候选，只能按简单优先作受限选择并明确说明，不能把缺失基线写成零错误。

首正文等待从请求发送起，取剥除思考后首段非空正文；完整等待到流结束，不含启动、清槽和分词时间。思考长度报告生成流中的token与字符；流中token数不等同于服务器总tokens_predicted，控制标记及结束token的口径单列。显存按WDDM每行owned PID集合匹配计数器，求和后取采样最大值，不是连续峰值，也不把其他桌面程序的显存算到模型上。

前两组各6次冒烟、228次调整的请求token和预算、原始流重建、冻结先后顺序已由机械子代理逐项核验，均无差异；主代理负责全文语义判定。机械核验记录位于私有`local-resume/mechanical-audit-first-two.json`。最终各配置计数、等待和显存见REPORT，不以本说明代替测量结果。

前两组归档日志均记录build 11669（620d2225c）和CUDA后端；Qwen3-4B-Instruct-2507为37/37层GPU offload，Qwen3.5-4B为33/33层。Qwen3.5日志有自动显存拟合因显式`-ngl 999`而中止的警告，之后服务成功初始化并完成调用。其CLIP另预留约4628.88 MiB CUDA compute buffer，因此这里的显存包含已登记mmproj路径的开销，不代表删去视觉组件后的最小文字推理显存。

其余调整阶段也完成机械核验：Qwen3.5-4B-thinking仅P3进入正式调整，76条中7条有效、69条limit；Qwen3-8B的两种思考配置各228条正式调用全部有效。逐项请求token、预算、原始流重建、正文边界和冻结先后均无差异。对应私有记录为mechanical-audit-thinking-formal.json、mechanical-audit-qwen8-preliminary.json和mechanical-audit-qwen8-thinking-formal.json，未用机械核验替代语义判定。

Qwen3-8B-thinking的官方模板以assistant header结束，实际生成流包含开标记151667与关闭标记151668；逐条核对两标记及正文边界，思考token计数排除生成的开标记和关闭标记。Qwen3.5-4B-thinking的关闭标记为248069，沿其模板和实际生成流计算，不把两个模型的控制标记口径混用。

检验阶段API 40条和本地100条均完成：123条有效、17条技术无效。原始流重建、冻结请求和预算逐项核验无差异，明细见私有mechanical-audit-holdout-first-six-arms.json与mechanical-audit-holdout-final-arm.json。结束后模型服务退出，nvidia-smi为0%利用率、736/10240 MiB；已通知用户可重新启动runner。
