按重档完成本次④–⑦的预定测量请求，前三项沿用已有数据，没有重跑。
文字首 token、图片处理、显存和重复输出都有记录；来源检查及计时局限也完整保留，不判定模型是否通过。
请逐项审查；GPU 已退出并通知恢复 runner，生产模型不切换，PR 仍保留草稿。

## 当前状态与改动

[PR #128](https://github.com/nanake-lab/nemu/pull/128)，分支 `task/P0-12-visual-carrier`。重档理由：正式 GPU 测量。继续时 fetch 核对 `origin/main` 仍为 `ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5`，相对原验证基线无新增变化；PR 头为 `196077fb16561097f0cf9b1ea28585c8d2ef1502`，最新评论没有额外审查结论。

按本次用户批准，为第四项补上 `--slot-save-path D:/Dev/cache/P0-12/gpu-02/item-4/server/slot-erase-only`。目录新建且最终为空，仅调用 erase，没有 save/restore。实际新旧 server argv **只多这一参数对**，包括原端口 57359 在内的其他值一致：[命令对照](continuation-02/command-comparison.json)。第五至七项为同一配置增加既定 mmproj，使用另一新建空 slot 目录；均不修改模型、引擎或固定图片。输出改用 `gpu-02`，原 `gpu-01` 不覆盖。

`http_probe.py` 补启动参数；`visual_http_benchmark.py` 接续第四项；`run_visual_http.ps1` 保留累计截止并每五秒记录 runner 检查；`visual_remaining.py` 顺序执行⑤–⑦；两个 summarize 脚本只读已保存数据。仅修改 `spikes/carrier-engine/` 与 `reports/P0-12/`，没有新安装依赖、生产代码/配置变更、方向提取或位置选择。

④的 274 请求、⑤的 21 图片请求、⑥的 3 图片采样请求、⑦的 9 重复请求共 **307 次 completion** 均返回完整结束事件，`cache_n=0`、无 context truncation；④和⑦全部 EOS，⑤有 **9/21 达到既定 64-token 输出上限**，没有删去这些样本或增加上限。独立 PNG 预处理时间未测；mypy 未运行，因此不声称全部工程验证完成，保持 draft。详见 [执行核对](continuation-02/execution-audit.json)与 [BLOCKED.md](BLOCKED.md)。

## 七项逐项结果与 P0-10 对照

旧值仅取 P0-10 的 [REPORT](../P0-10/REPORT.md)、[INJECTION-REPORT](../P0-10/INJECTION-REPORT.md)、[BENCHMARK-REPORT](../P0-10/BENCHMARK-REPORT.md)，本次没有重测旧模型。

| 项目 | P0-10 Qwen3-4B / Q4_K_M | P0-12 Qwen3.5-4B / Q4_K_M |
| --- | --- | --- |
| ① 零注入逐位置 logits | 中/英/日 24/11/16，共 51/51 字节一致，最大差 0 | 沿用前次中/英/日 12/11/12，共 35 个位置；11 组零注入、非零后清零/撤除配对各 35/35 字节一致，最大差 0 |
| ② 每请求更新 m | 清 KV 后 A 重复、fresh-A/B 各 51/51 一致；保留 KV 对照不同 | 沿用前次 attention/full/native 各自 A 重复与 fresh-A/B 各 35/35 一致；同时清 KV/recurrent。保留状态的 9 个首位置均不同，最大差 3.190608–5.494915 |
| ③ 两位置可用性 | 36 层零起算 17；attention 后及整层可观测 | 沿用前次 32 层零起算 15，`attn_residual-15` / `l_out-15` 可访问；2560 维 F32，末行/全行掩码和原生 control vector 实际加法均留证。不选择位置 |
| ④ 三语速度 / 首 token | 无约束 50 条 P50：prefill 432.02 ms，客户端 TTFT 3737.55 ms，decode 127.99 token/s | 无约束 50 条 P50：**270.77 ms / 283.88 ms / 116.62 token/s**；原 wire 200 条 P50：274.16 ms / 288.74 ms / 115.29 token/s |
| ⑤ 单图处理 | 纯文本，无对应项 | 256/512/1024 各 7 张；客户端 TTFT P50 **94.49 / 174.70 / 618.35 ms**。编码区间、image embedding prefill 与完整 prompt 边界见下表，PNG 预处理没有独立计时 |
| ⑥ 载体 + 视觉编码器显存 | 载体单独 context 8192，专用/共享独立峰值之和 **4,723,433,472 bytes** | 实际载体 + mmproj 同驻；全视觉会话独立峰值之和 **9,356,083,200 bytes**。第⑥项本身为 9,345,597,440 bytes；不是 TTS/渲染同驻结果 |
| ⑦ 不思考输出稳定性 | 没有对应三语重复专项；旧固定句精确匹配 0/20 | 三语各重复 3 次，各自只有 1 种输出，9 条生成文本均未见思考标记或解释性推理段落；第四项固定句精确匹配 **20/20**。只覆盖这些样本 |

①–③完整数据仍在 [injection-summary.json](results/injection-summary.json) 和 [原始映射](raw/raw-filemap.json)。按用户公式 `round(17/36 × 32)=15`，没有搜索层。仅固定 `e0=(1,0,...)`，A=+e0、B=-e0；直接调用 `llama_memory_clear(llama_get_memory(ctx), true)` 后安装本次向量再 prefill。固定 hybrid 源码同时清 attention/recurrent，fresh context 配对提供实测补充。保留状态诊断只比较首位置，不冒充全轨迹。每位置完整保存 248,320 个 float32 logits；计数包含末个 prefill 和后续 decode，非零组 teacher forcing，零组另有自由生成。激活加法最大 float32 误差 4.76837158203125e-7，不据此选择位置或判断概念方向有效。

## ④ 文字结果与输出检查

[逐请求与分语种统计](continuation-02/text-summary.json)。20 条固定中文句、50 条无约束、200 条原 wire 全保留；四条预热单列，不进统计。nearest-rank P50/P90，不剔除慢样本或格式/来源不符合的输出。来源检查沿用原 `local_irl.py::source_valid`，只查 sourceRef、节点 ID、sourceRefs 和 Unicode span 边界，不等同于人工语义正确率。

| 指标 P50 / P90 | P0-10 无约束 50 | 本次无约束 50 | P0-10 原 wire 200 | 本次原 wire 200 |
| --- | ---: | ---: | ---: | ---: |
| 原生 prompt，ms | 432.02 / 482.50 | 270.77 / 285.69 | 325.85 / 479.82 | 274.16 / 288.05 |
| 客户端 TTFT，ms | 3737.55 / 4373.81 | 283.88 / 302.15 | 328.41 / 4369.99 | 288.74 / 303.36 |
| 原生 decode，token/s | 127.99 / 133.39 | 116.62 / 118.85 | 127.16 / 132.65 | 115.29 / 118.15 |
| 整次完成，ms | 5635.36 / 7250.72 | 2389.50 / 2674.73 | 3315.21 / 7013.37 | 2429.00 / 2668.30 |
| 完整 Schema / 来源检查计数 | 49/50 / 47/50 | **46/50 / 22/50** | 200/200 / 192/200 | **200/200 / 100/200** |

因此不能只给出较短等待时间，省略来源检查结果。原 wire 的来源检查分语种为中 55/67、英 3/67、日 42/66；P0-10 分别为 67/67、67/67、58/66。原始输出未修写、未重试。

| 本次组别 / 语种 | n | prompt P50 ms | TTFT P50 / P90 ms | decode P50 token/s |
| --- | ---: | ---: | ---: | ---: |
| unconstrained / zh | 17 | 270.77 | 278.35 / 296.14 | 116.97 |
| unconstrained / en | 17 | 271.93 | 295.01 / 304.61 | 116.58 |
| unconstrained / ja | 16 | 270.59 | 282.84 / 301.70 | 116.62 |
| wire / zh | 67 | 273.15 | 286.69 / 302.15 | 114.76 |
| wire / en | 67 | 274.56 | 289.98 / 300.90 | 115.61 |
| wire / ja | 66 | 274.18 | 289.14 / 308.36 | 115.46 |

原生 decode TPS 使用引擎排除首 token 的内部口径；客户端 TTFT 为发送 completion 到首个 token/内容事件。固定句本次 TTFT P50/P90 54.32/71.37 ms，整次 151.45/237.60 ms；不与旧“首句事件”直接混为一个边界。模型、tokenizer/模板和本次 `--cache-ram 0`/显式 erase 与旧方案存在差异，这张对照表不是纯模型或纯引擎因果实验。

## ⑤ 图片计时与边界

沿用 [21 张合成 PNG 清单](inputs/images/manifest.json)，每种分辨率包括红/绿/蓝色块、几何组合、中/英/日文本。单张单请求，首张冷样本也保留，不另加图片预热。固定文字为 `Describe this synthetic image briefly. If there is text, transcribe it.`，嵌入同一官方非思考模板；实际 marker 从 server `/props` 读取，实际 prompt、完整 base64 请求和输出在 raw-02。没有图片下载、用户照片或屏幕输入。

| 分辨率 / n | 编码开始→图像 decode 开始 P50/P90 ms | image embedding decode 合计 P50/P90 ms | 原生完整 prompt P50/P90 ms | 客户端 TTFT P50/P90 ms |
| --- | ---: | ---: | ---: | ---: |
| 256² / 7 | 14.12 / 120.88 | 7.00 / 15.00 | 80.28 / 198.10 | 94.49 / 200.60 |
| 512² / 7 | 56.42 / 58.74 | 10.00 / 11.00 | 154.44 / 162.81 | 174.70 / 183.76 |
| 1024² / 7 | 339.45 / 361.21 | 111.00 / 114.00 | 590.69 / 608.54 | 618.35 / 646.95 |

[逐图计时边界](continuation-02/image-timing-boundaries-v2.json)同时保存日志行和原始偏移：编码区间从 `encode_mtmd_chunk` 调用前日志到 helper 的首批 decode 开始，包含主机/调度开销，**不是纯 GPU kernel 时间**；image embedding prefill 为 helper 日志中各批 llama_decode 的整数毫秒之和。256/512/1024 对应 64/256/1024 个图像 token；前两种各一批，1024 分两批 512。完整 prompt 时间包含图像编码、图像 embedding prefill 和文本 prompt 工作，不能再与这些子项相加。

实测配置日志为 CUDA CLIP、image_size=768、patch_size=16、image_min_pixels=8192、image_max_pixels=4194304，未修改这些默认值。**PNG 解码、resize、归一化没有独立计时，内部 resize 后尺寸也未单独导出**，不编造该部分数字。客户端计时不含本地文件读取/base64/模板准备。⑤中 9 条达到 64-token 输出上限，输出完整性及图片识别/OCR 准确率未做验收。

首次离线单批汇总对 1024 标为无独立值；随后根据同一原始日志追加了多批汇总 v2，没有重跑模型。v1 脚本、结果和日志仍保留。

## ⑥ 显存与⑦不思考重复

[显存摘要](continuation-02/memory-summary.json)：NVIDIA GeForce RTX 3080，载体 Q4_K_M + mmproj F16，context 8192、batch/ubatch 512、4 CPU threads、Flash Attention off，m=0。

| WDDM 采样范围 | 专用峰值 bytes | 共享峰值 bytes | 两独立峰值之和 bytes | 匹配快照 / 最大起始间隔 |
| --- | ---: | ---: | ---: | --- |
| 全视觉会话，含加载及⑤–⑦ | 9,240,739,840 | 115,343,360 | **9,356,083,200** | 198 / 0.485315 s |
| 第⑥项，3 个 1024 图请求 | 9,230,254,080 | 115,343,360 | **9,345,597,440** | 21 / 0.358581 s |

按 owned process tree 匹配，每个快照先合计，专用/共享分别取峰值后相加；全会话 199 快照中 1 次空匹配不当成零值，采样器无错误。这是采样独立最大值，不是连续峰值；没有加载 TTS/舞台渲染，不能签发整个 Nemu 的同驻结论。日志另记录视觉 compute buffer 4628.88 MiB，保留其实际配置，不为压低显存调整参数。

[九条重复输出](continuation-02/stability-summary.json)：使用原 `injection-zh/en/ja.txt`，每语种三次，greedy/seed 731，最多 64 tokens，每次先 erase；同语种生成文本逐字相同。人工读取九条均为要求复述的短句，未出现可见思考或解释段落；自动思考标记检查也为空。不外推到更多输入或内部推理过程。

官方 `enable_thinking=false` 模板预填 `<think>\n\n</think>\n\n`；这是输入前缀，不计作生成的思考内容。实际走 `/completion`，发送已渲染的 token/文本，未使用 server 默认 chat 示例的 thinking 设置。模板与实际输入在 [inputs](inputs/plan.json)、raw-02 的 actual-image-prompt / request 中，英文渲染还与冻结输入逐字核对。

## 来源、失败历史与可复现性

官方 `Qwen/Qwen3.5-4B` 修订 `851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a`，Apache-2.0；14 个官方文件哈希见 [model-inventory.json](preparation/model-inventory.json)。固定 llama.cpp b11669 / `620d2225cc436ab0b001e3200e590e3b286ef9ab`，MIT；本次 [55 个引擎文件](continuation-02/engine-check.json)全部与原清单相同，模型/mmproj及冻结输入 [30 项核对](continuation-02/preflight.json)无变化。无新增安装；既有 Python/torch CPU/transformers/Jinja 等版本与许可见 [environment.json](preparation/environment.json)。

| 产物 | bytes | SHA-256 |
| --- | ---: | --- |
| BF16，`--no-mtp` 主模型 32 层 | 8,424,393,728 | `1409508b90b58f536bc0dda4b1c3e1d54528f008f0ee53b08a4a6f976ab15388` |
| Q4_K_M | 2,708,804,608 | `045b7d309c8021d4d7bcebe1156ae94a1e3e128df271d05f5adfcea38ee20b27` |
| mmproj F16 | 672,423,520 | `73a88d2d689250d44e14bf7504476f287ca7a04ac2594592263251217e581069` |

D 盘首次可用 189,794,258,944 bytes，原预算 40 GB；本次继续前可用 162,644,070,400 bytes。未清理旧缓存。原始 curl 56 下载失败的 partial 和哈希、以及第四项首次 HTTP 501 的 traceback/argv/源码诊断全部保留：前者经用户授权续传解决，后者经本次授权仅补参数解决。旧失败不是模型不兼容证据，不下载或切换 Qwen3-VL。

本次窗口为 **2026-11-02 Perth 00:32:22.792–00:50:30.069**，1087.277 秒；连同先前 156.257 秒累计 **1243.534 秒（20 分 43.5 秒）**，包含阶段间准备，非 GPU 忙碌计数器。父进程按剩余时间强制终止自己启动的进程树，未触及三小时。开始前和请求前后检查 runner，另有 214 次定时检查全部为空；不是对所有瞬间后台状态的连续保证。两阶段入口均退出 0，sampler 均退出 0，server 的 Windows 返回 1 是 harness 主动关闭并已明确记录。结束后无残留模型进程，已通知恢复 runner。

原①–③及失败原始文件在 [raw](raw/raw-filemap.json)；本次 1269 文件、43,045,433 bytes 在 [raw-02](raw-02/raw-filemap.json)，去重后 1005 个 gzip 对象共 4,630,122 bytes，逐文件解压往返核验，缓存原文件未删除。按 filemap 将 object 解压到新目录下的 path，并核对 bytes/sha256 即可恢复。新的交付摘要见 [manifest-continuation-02.json](manifest-continuation-02.json)；历史 manifest/closeout 描述各自提交，不能当作当前源码摘要。

```powershell
Set-Location D:/Dev/nemu-wt/P0-12
# 本次执行记录；现有目录受防覆盖保护，不表示批准再次运行
pwsh -NoProfile -File spikes/carrier-engine/run_visual_http.ps1
pwsh -NoProfile -File spikes/carrier-engine/run_visual_http.ps1 -ScriptName visual_remaining.py -LogPrefix items-5-7
# GPU 退出后，仅离线整理
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/summarize_visual_remaining.py
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/summarize_visual_image_timing.py
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/archive_evidence.py --source D:/Dev/cache/P0-12/gpu-02 --inventory reports/P0-12/continuation-02/raw-inventory.json --output reports/P0-12/raw-02
```

## 检查及待审查限制

- [HTTP 流/缓存/截断边界测试](continuation-02/unittest.log)：`python -m unittest discover -s spikes/carrier-engine -p test_http_probe.py -v`，3/3，无失败；GPU 退出后才运行。
- Python AST、PowerShell AST、JSON 解析、Markdown 链接、修改范围、当前文件与暂存摘要见 [closeout](closeout-continuation-02.json)。没有通过删除断言、剔除样本或改阈值消除错误。
- 环境没有 mypy，`mypy --strict` **未运行**；没有自动安装新依赖。图像预处理独立计时、图片识别/OCR准确率、TTS/渲染同驻和更广输入稳定性均没有实测结论。
- 原始 CRLF、模板末尾空行等如实保留；全量 diff-check 的 whitespace 结果与源码/报告检查分别记录，不篡改原始字节消除提示。机器生成清单/逐位置数据和原文归档超过 400 行是为保持证据；新增手写入口均小于 400 行。

请审查本次测量与上述限制，逐项由用户判断是否满足切换条件。保持 draft / needs-user，不合并、不自动回退、不等待或报告 CI，不取下一卡。
