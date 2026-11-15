# 本地组准备（未执行）

用户最新指示：暂不运行本地组，先准备与API组。本文件是已知路径和命令草稿，不代表转换或测量结果。

| 配置 | 模型路径 | 思考 |
| --- | --- | --- |
| Qwen/Qwen3.5-4B Q4_K_M | D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf | enable_thinking=true / false |
| Qwen/Qwen3-4B-Instruct-2507 Q4_K_M | D:/Dev/cache/P0-10/gguf/Qwen--Qwen3-4B-Instruct-2507/cdbee75f17c01a7cc42f958dc650907174af0554/carrier-Q4_K_M.gguf | enable_thinking=false |
| Qwen/Qwen3-8B Q4_K_M（待转换） | D:/Dev/_leftover/2026-10-20/workers/model-adapter/probe/.cache/models/Qwen--Qwen3-8B/b968826d9c46dd6066d109eabc6255188de91218 | enable_thinking=true / false |

只读写这些已知模型目录，不全盘扫描。Qwen3-8B转换产物置于同一模型修订目录的R-PROMPT-FIT-01子目录，保留原始BF16与任何失败partial文件。

登记工具：llama.cpp b11669，提交620d2225cc436ab0b001e3200e590e3b286ef9ab；源代码D:/Dev/cache/P0-10/source/llama.cpp-b11669，转换Python D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe，量化器D:/Dev/cache/P0-10/llama.cpp/b11669-cpu-quantize/llama-quantize.exe，推理release目录D:/Dev/cache/P0-10/llama.cpp/b11669。正式执行前核对登记哈希与release构建信息。

转换命令形式：`convert_hf_to_gguf.py <上述8B目录> --outfile <同目录产物>/carrier-BF16.gguf.partial --outtype bf16`；成功后保留BF16，再执行`llama-quantize.exe <BF16> <同目录产物>/carrier-Q4_K_M.gguf.partial Q4_K_M 4`。实际参数、开始/结束、退出码、工具版本及模型SHA-256随后记录；失败不换工具、不下载模型，保留原因并跳过该模型。

沿载体登记：greedy，temperature=0，seed=731，repeat_penalty=1，presence/frequency penalty=0；单槽、每条清空、cache_prompt=false、无控制向量。确定性组每项1次，不能宣称满足随机组2/3重复优势。官方模板显式传enable_thinking，按实际模板/引擎结果记录思考与正文边界；思考长度另报字符及可得token，不混入首正文等待。

长历史保留全部原话，不能为装下而暗删目标。装包前实际分词并检查上下文与输出预算；放不下记技术不可执行，报告上下文配置与限制。一次只加载一个模型；测量记录显存峰值、首token/首正文与完整等待。用户安排窗口后，确认#205本地补跑已结束、Runner.Listener/Runner.Worker均不存在且GPU空闲；窗口结束通知恢复runner。
