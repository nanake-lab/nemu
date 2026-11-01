官方权重已下载并核验，固定 b11669 已完成 BF16、Q4_K_M 和视觉编码器转换。
前三项注入数据已取得；第四项因测量脚本缺少 server 参数而停止，后三项未测。
本次不判定模型通过、不切换生产模型；GPU 已退出，可以恢复 runner。

## 状态与边界

**重档；部分交付，阻塞于第四项。** [BLOCKED.md](BLOCKED.md) 记录原因与待用户决定事项。[Draft PR #128](https://github.com/nanake-lab/nemu/pull/128)，分支 `task/P0-12-visual-carrier`，基于 `origin/main` 的 `ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5`。续做时 fetch 后必读文件无变化。

用户“继续下载，网络波动时有发生”授权后，沿同一 curl 方法续传。原失败 partial 另存并保留，恢复过程实际网络重试次数为 0；14 个官方文件均完成大小和上游摘要核验。随后严格沿准备、合成输入、注入、七项测量顺序执行；第四项第一次请求 slot erase 返回 HTTP 501 后停止，未更改参数重试。

没有删除文件、下载 Qwen3-VL、提取概念方向、选择注入位置、设新阈值或改生产代码/配置。仅两个允许目录产生仓库改动，权重、构建和原始数据位于 D 盘缓存。本报告逐项交回用户判断，不签发通过结论，也不把脚本退出 0 等同于模型通过。

## 七项结果与 P0-10 对照

旧值来自 P0-10 [REPORT](../P0-10/REPORT.md)、[INJECTION-REPORT](../P0-10/INJECTION-REPORT.md)、[BENCHMARK-REPORT](../P0-10/BENCHMARK-REPORT.md)，未重测旧模型；不同词表/模板/生成长度不能作同条件性能比较。

| 项目 | P0-10 Qwen3-4B / Q4_K_M | 本次 Qwen3.5-4B / Q4_K_M |
| --- | --- | --- |
| 1. 零注入逐位置 logits | 中/英/日 24/11/16，共 51/51 位置字节一致，最大差 0 | 中/英/日 12/11/12，共 35 个位置；每组零注入、非零后清零、原生撤除配对均 35/35 字节一致，最大差 0；共 11 组、385 个位置配对 |
| 2. 每请求更新 m | 显式清 KV 后 A 重复、fresh-A/B 各 51/51 一致；保留 KV 诊断不一致 | attention/full/native 各自 A 重复及 fresh-A/B 各 35/35 一致；显式同时清 KV 和 recurrent。保留两类状态的诊断共 9 个首位置，0/9 字节一致，最大绝对差 3.190608–5.494915 |
| 3. 两个位置可用性 | 36 层零起算 17，attention 后与整层均有观测；原生覆盖全部 prefill 行 | 32 层零起算 15，`attn_residual-15` 与 `l_out-15` 实测可访问；2560 维 F32，末行与全行掩码均产生预设加法，原生 control vector 覆盖全行；不选择位置 |
| 4. 三语速度 / 首 token | 无约束 50 条 P50：内部 prefill 432.02 ms、客户端 TTFT 3737.55 ms、decode 127.99 token/s；原 wire 200 条三语 67/67/66 | **失败于请求前准备**：server 启动后第一次 slot erase 返回 HTTP 501。4 预热、270 正式请求均未发送；没有可报告的 HTTP TTFT、prefill、decode 统计 |
| 5. 单图处理时间 | 纯文本，无对应测量 | 未测；21 张合成 PNG 已生成，未送入模型 |
| 6. 载体 + mmproj 显存峰值 | context 8192 载体单独专用/共享独立峰值之和 4,723,433,472 bytes；无 mmproj | 未测；mmproj 已转换但未加载。失败 server 的载体单独采样已保留，不能冒充本项结果 |
| 7. 不思考输出稳定性 | 固定句精确匹配 0/20；没有与本卡完全对应的重复稳定性专项结论 | 未测专项重复请求。前三项使用官方非思考模板，已留 token 轨迹，但不将其替代本项输出稳定性或思考内容判断 |

## 注入证据和限制

完整逐位置数据见 [injection-summary.json](results/injection-summary.json)。每个位置保存全部 **248,320 个 float32 logits**，比较完整字节及最大绝对差，同时保留预测 token 和实际喂入 token。step 0 是最后一个 prefill 位置，其余为 decode。每条最多 24 个位置，遇 EOG 停；实际三语各 12/11/12。非零组沿同语种基线轨迹 teacher forcing，零注入另有自由生成轨迹，三种零注入的自由 token 与各自基线相同。

层换算严格为 `round(17/36 × 32) = 15`，不搜索层。该层是 full_attention；模型还有 linear_attention 层。只用预先固定的 2560 维 `e0=(1,0,...)`，A=+e0、B=-e0、零向量/撤除，不复用旧概念方向。

直接调用 `llama_memory_clear(llama_get_memory(ctx), true)` 后安装本次向量再 prefill。固定 [hybrid 源码](preparation/upstream/llama-memory-hybrid.cpp) 同时调用 attention/recurrent clear；[recurrent 源码](preparation/upstream/llama-memory-recurrent.cpp) 清理 cells、位置、src/tail、索引并清零缓冲。实测同一 context 的 A→B→0→A、fresh-A/B 配对补充静态依据。保留状态的诊断只比较首位置，不能称为全轨迹配对，summary 中 `samePositionSet=false` 如实保留。

| 语种 | prefill 行数 | attention 末行/全行实际改变行数 | full 末行/全行实际改变行数 | 原生改变行数 |
| --- | ---: | ---: | ---: | ---: |
| 中 | 35 | 1 / 35 | 1 / 35 | 35 |
| 英 | 28 | 1 / 28 | 1 / 28 | 28 |
| 日 | 30 | 1 / 30 | 1 / 30 | 30 |

预设加法与实际 float32 差值的最大误差为 4.76837158203125e-7，逐模式数值及 decode 回调记录见 summary / raw。原生注入在 `l_out` 回调之前发生，以无向量观测配对测量。`old` 是旧公共探针保留的字段名，本次明确指 `attn_residual-15`，不是旧模型节点。这里不比较位置优劣，不将固定基向量可写入推断为行为概念方向有效。

## 第四项停止原因

b11669 已加载 CUDA、完成全层卸载、建立 context 8192，并明确记录 prompt cache disabled。实际参数包含 `--cache-ram 0`、`--slots`；待发送 completion 的 payload 固定 `cache_prompt=false`。官方模板的实际非思考输入已经离线渲染，server 的默认 chat 示例 `thinking=1` 不代表本次预渲染输入；本次未发送 completion。

测量脚本漏加 `--slot-save-path`，而固定源码 `post_slots` 在该参数为空时对所有 slot action 返回 501（即使 action=erase）。**这是本代理编写的测量配置遗漏，不能归因于模型或 b11669 不支持 Qwen3.5。** [诊断证据](results/server-failure-diagnosis.json) 保存源码摘要与行号片段；原始 traceback 在 raw 的 `item-4.stderr.log`。HTTP 错误响应正文未被保存，具体原因由固定源码和实际 argv 对照得出，不伪称捕获到了正文。

异常后 harness 终止自己启动的 server，采样器退出 0。未修参数重跑、未改引擎、未转直接调用绕过第四项。第五至七项没有启动，也没有提前实现/声称已验证其测量入口。

## 输入、模板、空间与来源

文字输入见 [inputs/plan.json](inputs/plan.json)、[jobs.json.gz](inputs/jobs.json.gz)、[schemas.json](inputs/schemas.json)。沿用 P0-10 的 20 条固定句、前 50 条无约束、全部 200 条原 wire schema；200 条中/英/日为 67/67/66，50 条为 17/17/16，固定句 20 条为旧方案的中文原句。四条预热中原第四条 compact 改为原 wire 重复预热，以遵守本次禁止紧凑 schema；其余原消息内容保持不变。新模型 tokenizer/template 导致 token IDs 与旧模型不同。

[官方模板](preparation/model/chat_template.jinja) 的 `enable_thinking=false` 已用于实际渲染，模板 SHA-256 `a4aee8afcf2e0711942cf848899be66016f8d14a889ff9ede07bca099c28f715`。实际 prompt 在 jobs 和三个 `inputs/injection-*.txt`，末尾预填 `<think>\n\n</think>\n\n`，这些是模板前缀，不是实测生成的思考内容。

[图片清单](inputs/images/manifest.json)：256、512、1024 三种正方形分辨率，每种包含红/绿/蓝纯色、几何组合、中文/英文/日文文本，共 21 张 PNG。System.Drawing 脚本绘制，字体 `msyh.ttc` 的哈希已登记；无下载图片、用户照片或屏幕输入。图片未进行模型识别测试，不作 OCR 成功声明。

D 盘首次可用 **189,794,258,944 bytes（189.79 GB / 176.76 GiB）**，本次预算 40 GB（原权重约 9 GB、下载暂存最多约 9 GB、BF16/mmproj 约 9 GB、Q4 约 3 GB、证据余量约 10 GB）。续做检查可用 185,794,899,968 bytes，足够；没有执行删除。原失败 2,418,060,864 bytes partial 保留在 `D:/Dev/cache/P0-12/failed-download-01/shard-01.partial`，哈希见 [续传完成记录](preparation/resume-01/completion.json)。

官方仓库 `Qwen/Qwen3.5-4B` 固定修订 **851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a**；14 文件实测摘要和上游核验结果见 [model-inventory.json](preparation/model-inventory.json)，完整 [官方元数据](preparation/model-metadata.json)。两份权重合计 **9,319,828,096 bytes**。许可 Apache-2.0，原文 LICENSE 已保存。首次网络失败的日志、哈希和历史清单 [manifest.json](manifest.json) 不覆盖；它描述旧停止点，不是本次最终清单。

固定 llama.cpp **b11669 / 620d2225cc436ab0b001e3200e590e3b286ef9ab**，MIT。转换使用原 CPU venv（torch 2.11.0+cpu、transformers 4.57.6），量化使用同版 CPU 独立目录；未安装新依赖。源码/引擎校验沿 [converter-source-check.json](preparation/converter-source-check.json)、[比较更正](preparation/converter-source-comparison.json)、[engine-check.json](preparation/engine-check.json)。版本/许可见 [environment.json](preparation/environment.json)。CUDA 二进制等仍按 P0-10 本机使用范围，不作整包分发批准。

| 自转换产物 | bytes | SHA-256 |
| --- | ---: | --- |
| BF16（`--no-mtp`，仅主模型 32 层） | 8,424,393,728 | `1409508b90b58f536bc0dda4b1c3e1d54528f008f0ee53b08a4a6f976ab15388` |
| Q4_K_M | 2,708,804,608 | `045b7d309c8021d4d7bcebe1156ae94a1e3e128df271d05f5adfcea38ee20b27` |
| mmproj F16 | 672,423,520 | `73a88d2d689250d44e14bf7504476f287ca7a04ac2594592263251217e581069` |

[转换 argv、时间、退出码](preparation/conversion-commands.json) 与 [产物清单](preparation/gguf-inventory.json) 完整保留，三步退出均 0。成功转换和本次文本/注入加载不等于视觉运行兼容性已经验证。

## 安静窗口与原始数据

开始前确认 Runner.Listener / Runner.Worker 均未运行，原生探针每次请求和 decode 再检查，server 入口也检查。窗口 UTC **2026-11-01 16:11:04.945 → 16:13:41.202**（本地 11 月 2 日 00:11–00:13），约 **156.26 秒**，这是含阶段间准备的窗口墙钟时间，并非 GPU 忙碌计数器。硬截止 19:11:04.945 UTC；父进程按剩余时间 WaitForExit，超时终止自己启动的进程树；未触发上限。异常后没有继续 GPU 工作，已通知用户恢复 runner。

6807 份原始文件共 1,394,458,102 bytes；SHA-256 去重后 824 个 gzip 对象共 244,324,309 bytes，逐个解压往返核验。见 [raw-inventory.json](results/raw-inventory.json)、[raw-filemap.json](raw/raw-filemap.json)、[归档日志](preparation/archive.log)。完整 logits、激活、token 轨迹、进程日志、失败 server 的采样都能按 filemap 恢复；缓存原文件未删除。当前交付文件摘要见 [manifest-resume.json](manifest-resume.json)，排除清单自身与最终 closeout 记录。

## 可复现入口（记录本次运行，不是批准再次运行）

```powershell
Set-Location D:/Dev/nemu-wt/P0-12
pwsh -NoProfile -File spikes/carrier-engine/prepare_visual_model.ps1 -EvidenceDirectory reports/P0-12/preparation/resume-01
pwsh -NoProfile -File spikes/carrier-engine/convert_visual_model.ps1
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/prepare_visual_inputs.py
pwsh -NoProfile -File spikes/carrier-engine/prepare_visual_images.ps1
pwsh -NoProfile -File spikes/carrier-engine/build_probe.ps1 -VisualCarrier -OutputDirectory D:/Dev/cache/P0-12/build-visual-01
pwsh -NoProfile -File spikes/carrier-engine/run_visual_injection.ps1
pwsh -NoProfile -File spikes/carrier-engine/run_visual_http.ps1 # 本次在第一次 slot erase 失败
# 停止 GPU 后，仅整理已保存数据：
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/summarize_visual_injection.py
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/archive_evidence.py --source D:/Dev/cache/P0-12/gpu-01 --inventory reports/P0-12/results/raw-inventory.json --output reports/P0-12/raw
```

现有输出目录受到防覆盖保护，重放应使用新目录且需用户解除本次失败停止状态。原生各阶段精确 argv、开始/结束时间和 exit code 均在 raw 根目录。恢复原始文件可遍历 filemap 的 files，将 `gzip.decompress(raw/object)` 写入新目录的 path，并核对 bytes/sha256。文件分块为逐文件无损 gzip，不是仅保留摘要。

## 验证与已知限制

- 官方文件摘要核验、三步 CPU 转换退出 0；新探针 MSVC 14.44.35207 `/W4 /WX /O2 /fp:precise` 编译退出 0，[构建绑定](preparation/build-visual.json)、[日志](preparation/build-visual.log)。前三项执行退出 0，只说明采集完成。
- 第四项退出 1：HTTP 501；停止后只做离线统计、哈希、归档及交付检查，不运行后续测量或以重跑替换失败。
- 公共 Native/Trace/build/http/compare 的新参数保留 P0-10 默认行为；本次只实测新模型路径。未重跑旧模型或整个生产工作区，也没有声称完成 Python mypy 全套检查。整卡仍未完成，不能合并或当作模型切换验收。
- 初次汇总的 `Where-Object` 字段比较错误已有独立更正，历史原记录未覆盖；本次两次只读源码搜索使用不存在目录/Windows 通配形式，未取得内容，随后从已固定的实际源路径读取，没有更换引擎或测量方法。
- 原始 preparation 证据保持 CRLF；历史 diff whitespace 检查退出 2，原因包括原始日志/官方 LICENSE，不篡改原始字节消除提示。最终交付检查单独记录于 closeout-resume.json。
- 首次交付字节核对将 Git LFS 指针误当图片内容，并发现 JSON 暂存换行转换，记录保留在 preparation/closeout-resume-attempt01.json。最终改用 LFS oid/size 核对图片，并按 -text 属性重新暂存 JSON；不改实验值、不重跑测量。
- 完整上游源码和机器生成 JSON 清单可能超过 400 行，为保存原文和逐文件/逐位置证据；新增手写脚本均小于 400 行。

## 下一步

等待用户决定是否授权修正本次测量配置后继续；本次保持 draft / needs-user，推送后请审查。未测项如实保留，不自动回退、不等待或报告 CI、不取下一卡。
