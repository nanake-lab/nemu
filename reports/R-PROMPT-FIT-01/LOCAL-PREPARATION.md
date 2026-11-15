# 本地组准备（未执行）

历史准备快照：以下“未执行”“待转换”和GPU待确认均描述恢复执行前的状态。用户随后允许保留桌面背景程序开展测量；转换、本地串行测量和独立检验均已结束。当前执行证据见[LOCAL-EXECUTION.md](LOCAL-EXECUTION.md)，最终结果以[REPORT.md](REPORT.md)为准，以下原准备记录保留。

用户已通知#205本地补跑结束并恢复本地任务。本文件仍是准备结果，不代表转换或模型测量；GPU前置条件尚待确认，见[BLOCKED.md](BLOCKED.md)。

恢复检查：main未变，#205最新head为aa50f06230a5291be0d0d1f4cbe72e35e3438dff。检验长历史已经在API调整前冻结，本次核对哈希一致，不重写。nvidia-smi为RTX 3080、0%利用率、863/10240 MiB占用，runner均停止；23项均显示C+G、逐项显存N/A，尚无法确认无其他计算进程。

已使用既有转换环境（Jinja2 3.1.6、tokenizers 0.22.2）完成五配置各234份材料的官方模板渲染与CPU分词，不运行推理。最大输入分别为5920、5922、5880、5880、5884 token；均装入8192上下文，最少输出预算2269，完整历史未裁切。strict mypy与Ruff无错误。新私有证据[local-resume](https://github.com/nanake-lab/nemu-evidence/tree/fa781bdf9db7739edb7392cc5bac28faf5658ec7/R-PROMPT-FIT-01/local-resume)保存逐配置输入、摘要和检查记录；不是冒烟成功或执行冻结。

私有证据已生成 `local-material-plan.json`：234份消息包（每写法76条调整、2条冒烟）供5个配置使用，未来计划1140次调整与30次冒烟。该文件明确标为材料准备，尚非可执行的冻结本地计划；没有加载模型、启动服务、分词或转换。后续仍需核对模板、分词、引擎及模型哈希，完成本地冒烟后再冻结执行。

| 配置 | 模型路径 | 思考 |
| --- | --- | --- |
| Qwen/Qwen3.5-4B Q4_K_M | D:/Dev/cache/P0-12/gguf/carrier-Q4_K_M.gguf | enable_thinking=true / false |
| Qwen/Qwen3-4B-Instruct-2507 Q4_K_M | D:/Dev/cache/P0-10/gguf/Qwen--Qwen3-4B-Instruct-2507/cdbee75f17c01a7cc42f958dc650907174af0554/carrier-Q4_K_M.gguf | enable_thinking=false |
| Qwen/Qwen3-8B Q4_K_M（待转换） | D:/Dev/_leftover/2026-10-20/workers/model-adapter/probe/.cache/models/Qwen--Qwen3-8B/b968826d9c46dd6066d109eabc6255188de91218 | enable_thinking=true / false |

只读写这些已知模型目录，不全盘扫描。Qwen3-8B转换产物置于同一模型修订目录的R-PROMPT-FIT-01子目录，保留原始BF16与任何失败partial文件。

登记工具：llama.cpp b11669，提交620d2225cc436ab0b001e3200e590e3b286ef9ab；源代码D:/Dev/cache/P0-10/source/llama.cpp-b11669，转换Python D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe，量化器D:/Dev/cache/P0-10/llama.cpp/b11669-cpu-quantize/llama-quantize.exe，推理release目录D:/Dev/cache/P0-10/llama.cpp/b11669。正式执行前核对登记哈希与release构建信息。

Qwen3.5-4B沿已登记载体同时核对模型同目录的mmproj-F16.gguf；本任务输入仍为纯文字。上游local-v4执行器采用8192上下文，输出预算为8192减实际输入token再减1，包含思考和正文；后续不得暗中截短历史来换取输出空间。

转换命令形式：`convert_hf_to_gguf.py <上述8B目录> --outfile <同目录产物>/carrier-BF16.gguf.partial --outtype bf16`；成功后保留BF16，再执行`llama-quantize.exe <BF16> <同目录产物>/carrier-Q4_K_M.gguf.partial Q4_K_M 4`。实际参数、开始/结束、退出码、工具版本及模型SHA-256随后记录；失败不换工具、不下载模型，保留原因并跳过该模型。

沿载体登记：greedy，temperature=0，seed=731，repeat_penalty=1，presence/frequency penalty=0；单槽、每条清空、cache_prompt=false、无控制向量。确定性组每项1次，不能宣称满足随机组2/3重复优势。官方模板显式传enable_thinking，按实际模板/引擎结果记录思考与正文边界；思考长度另报字符及可得token，不混入首正文等待。

长历史保留全部原话，不能为装下而暗删目标。装包前实际分词并检查上下文与输出预算；放不下记技术不可执行，报告上下文配置与限制。一次只加载一个模型；测量记录显存峰值、首token/首正文与完整等待。用户安排窗口后，确认#205本地补跑已结束、Runner.Listener/Runner.Worker均不存在且GPU空闲；窗口结束通知恢复runner。
