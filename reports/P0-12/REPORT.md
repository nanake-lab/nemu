已认领 P0-12，官方权重下载中途断开，已保留文件和证据并停止。
尚未转换或运行模型，七项测量全部未测，不能据此判断模型是否适用。
需要用户决定是否另行授权恢复下载；本次不再使用 GPU，可以恢复 runner。

## 当前状态

**重档：用户指定，本卡计划包含正式 GPU 测量；本次止于准备步骤。** 部分完成，保持 draft，见 [BLOCKED.md](BLOCKED.md)。不取下一卡，不合并，不等待或报告 CI。

任务分支 `task/P0-12-visual-carrier` 从 main `ee5c2b6068b9a2200ad5fd48ebbc8e337458c0d5` 建立（含 ADR 0034）；[PR #128](https://github.com/nanake-lab/nemu/pull/128)。只写本卡允许的两个目录；模型文件在获准的 D 盘缓存中。

步骤 2 下载第一份权重时 `curl` 退出 56：`schannel: server closed abruptly (missing close_notify)`。下载进程退出 1；6 个小文件已按上游摘要核验。第一份权重仅有 2,418,060,864 bytes 的 `.partial`，预期 5,329,398,688 bytes，未通过完整性认证；第二份权重未下载。没有重试、换下载工具或关闭 TLS 校验。原始 [download.log](preparation/download.log)、[失败记录](preparation/download-failure.json)、[已核验文件](preparation/download-files.json)均保留。

BF16、Q4_K_M、mmproj、合成输入生成、注入探针适配及七项 GPU 测量全部未执行。GPU 使用时间为 0；三小时测量上限未开始计时。未下载 Qwen3-VL-4B-Instruct。下载故障不证明 Qwen3.5 有模型或引擎兼容性问题，也不触发自动回退。

## 空间

D 盘初次盘点可用 **189,794,258,944 bytes（189.79 GB，176.76 GiB）**。本次预算约 40 GB：原权重约 9 GB、下载暂存副本最多约 9 GB、BF16/mmproj 约 9 GB、Q4 约 3 GB、证据和余量约 10 GB。实际采用直接下载，没有分段副本；40 GB 是准备预算而非模型验收阈值。官方两份原始权重实际合计 **9,319,828,096 bytes**。空间充足，未删除任何文件，无需提供清理候选。第二次空间快照见 [plan-binding.json](preparation/plan-binding.json)，停止时文件占用及哈希见 [manifest.json](manifest.json)。

## 七项结果及 P0-10 对照

旧值只来自 P0-10 的 [REPORT](../P0-10/REPORT.md)、[INJECTION-REPORT](../P0-10/INJECTION-REPORT.md)、[BENCHMARK-REPORT](../P0-10/BENCHMARK-REPORT.md)，本次没有重测旧模型，也不把旧值作为新模型阈值。

| 项目 | P0-10 Qwen3-4B / Q4_K_M 的已有记录 | P0-12 Qwen3.5-4B |
| --- | --- | --- |
| 1. 零注入逐位置 logits | 中/英/日 24/11/16，合计 51/51 位置完整字节一致，最大差 0 | 未测；无 logits 或 token 轨迹 |
| 2. 每请求更新 m | 显式清 KV 后 A 重复、fresh-A/B 各 51/51 一致；保留 KV 的诊断不一致 | 未测；循环状态清除与不清除对照均无实测 |
| 3. 两个注入位置可用性 | 36 层中零起算 17 层；attention 后与整层位置均有观测，原生广播覆盖全部 prefill 行 | 未测；只取得固定源码，不能声称回调或原生注入已可用 |
| 4. 中英日速度 / 首 token | 50 条无约束 P50：原生 prefill 432.02 ms、客户端 TTFT 3737.55 ms、decode 127.99 token/s；原 wire 200 条，三语 67/67/66 | 未测；4 预热、20 固定句、50 无约束、200 原 wire 输入尚未生成；不使用紧凑 schema |
| 5. 单张图片处理时间 | 旧载体为纯文本，无对应测量 | 未测；三种分辨率合成图片尚未生成 |
| 6. 载体 + 视觉编码器显存 | 旧载体单独、context 8192，专用和共享独立峰值之和 4,723,433,472 bytes；无 mmproj | 未测；无加载及采样，不能作 TTS/渲染同驻结论 |
| 7. 不思考输出稳定性 | 旧模型非思考配置；零注入自由 token 对照一致；20 固定句额外加顿号，精确匹配 0/20；没有与本卡完全对应的重复稳定性专项结论 | 未测；仅固定官方模板，没有模型输出 |

逐项交回用户判断；本报告不签发“通过/不通过”。下载失败与上述未测状态不支持条件切换；生产基线未改变。

## 已取得的准备证据

- 官方仓库 `Qwen/Qwen3.5-4B`，固定修订 `851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a`；[完整官方元数据](preparation/model-metadata.json)。许可 Apache-2.0，已下载 LICENSE；权重尚未完整取得，不能把元数据中的预期哈希写成实测哈希。
- [config.json](preparation/model/config.json) 为 32 层、hidden size 2560、词表 248320。用户公式 `round(17/36 × 32) = 15`，零起算第 15 层是配置中的 full_attention；只是固定换算，未进行择层或实测。
- [官方模板](preparation/model/chat_template.jinja) 的 `enable_thinking=false` 分支预填 `<think>\n\n</think>\n\n`。实际输入模板尚未渲染，未声称生成内容稳定或无思考。
- llama.cpp 固定 b11669 / `620d2225cc436ab0b001e3200e590e3b286ef9ab`：[189 源文件哈希](preparation/converter-source-check.json)、[字段比较更正](preparation/converter-source-comparison.json)、[55 引擎文件哈希](preparation/engine-check.json)。源码与引擎均匹配旧清单；未启动引擎验证兼容。
- 固定提交的 [qwen35.cpp](preparation/upstream/qwen35.cpp) 有 `attn_residual` 和 `l_out` 节点，整层处调用 `build_cvec`；[hybrid clear](preparation/upstream/llama-memory-hybrid.cpp) 调用 attention/recurrent 的 clear，[recurrent clear](preparation/upstream/llama-memory-recurrent.cpp) 在 data=true 时清缓冲。仅为静态依据，不能代替 fresh context 配对实测。原文 Git blob 与已缓存的固定提交树一致，见 [upstream-inventory.json](preparation/upstream-inventory.json)。上游两个文件超过 400 行，为保留固定源码与 MIT 许可而完整归档，不是新增产品实现。

## 交付与可复现入口

- [prepare_visual_model.ps1](../../spikes/carrier-engine/prepare_visual_model.ps1)：固定修订、单次下载、上游 SHA-256/Git blob 校验，失败即停并保留 partial。本次已执行一次并失败。
- [convert_visual_model.ps1](../../spikes/carrier-engine/convert_visual_model.ps1)：CPU 环境、源码/模型/引擎绑定检查，BF16 → Q4_K_M → F16 mmproj；排除推测解码 MTP 草稿层，保留主模型 32 层。**尚未执行，不能视为已验证转换入口。**

本次命令如下；下载目录已存在，入口会拒绝覆盖，后续恢复方式须用户另行决定，不能直接重跑。

```powershell
Set-Location D:/Dev/nemu-wt/P0-12
pwsh -NoProfile -File spikes/carrier-engine/prepare_visual_model.ps1 *> reports/P0-12/preparation/download.log
# 以下未执行；只有准备成功且无阻塞时才可运行
pwsh -NoProfile -File spikes/carrier-engine/convert_visual_model.ps1
```

没有可复现的 GPU 结果命令，因为尚未实现/执行本卡 GPU 入口；不把 P0-10 旧入口伪装成本卡测量。原始记录与文件摘要清单在本报告目录。

## 检查、失误与限制

两个 PowerShell 入口 AST 语法错误 0，见 [script-parse.json](preparation/script-parse.json)；只有下载入口在真实边界执行，转换入口仅作语法检查。哈希、JSON 完整解析、Markdown 相对文件链接和 diff 检查见 [closeout-check.json](closeout-check.json)。本次没有改动生产语言包，不运行其工作区测试；准备失败后没有继续执行转换或 GPU 测试。

归档时给 preparation 原始证据设置 `-text`，避免 Git 转换 CRLF、破坏实测哈希；逐文件工作文件与暂存 blob 已核对相同。此前默认文本规范化下 diff 检查退出 0，保留原始 CRLF 后最终 `git diff --cached --check` 退出 2，报原始证据行尾 whitespace（包含官方 LICENSE）。最终 diff 检查不记通过；不修改原始字节或关闭 whitespace 检查来消除提示。该归档检查结果与下载故障、未测状态分开记录。

源码哈希首次汇总误用 `Where-Object expected -ne actual`，将 actual 当字面值，错误报告 189 个差异。原始逐文件 expected/actual 均保留；用 `Where-Object { $_.expected -ne $_.actual }` 比较既有记录，两列实际差异为 0。首次错误汇总未覆盖，单独追加更正。一次只读 `rg` 的 Windows 通配路径写法报路径错误，改为目录加 `-g '*.cpp'` 后读到源码；这两项不是转换或 GPU 结果。

## 依赖与下一步

没有新增安装或产品运行时依赖。复用的 CPU 工具环境版本与许可见 [environment.json](preparation/environment.json)；固定 llama.cpp 源码为 MIT，二进制 CUDA/LLVM 组件仍沿 P0-10 的本地使用范围，不作整包分发批准。模型 Apache-2.0 许可不等于本次已下载完整模型。

等待用户就 [BLOCKED.md](BLOCKED.md) 决定后续；本次停止。已通知恢复 runner。推送后仅请求审查当前部分交付，不把整卡标为完成。
