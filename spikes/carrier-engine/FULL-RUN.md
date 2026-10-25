# P0-10 完整测量入口（未运行）

GPU 仍需新的用户确认：停 runner 后查无 Listener/Worker。上一轮最小实验的确认已用完。本文件的命令不是用户授权或测量结果。

## 已完成的 CPU 准备

- `D:/Dev/cache/P0-10/injection-build-v3/build.json` 记录编译器、可执行文件和四个源码的哈希；MSVC `/W4 /WX /O2 /fp:precise` 退出 0。
- `D:/Dev/cache/P0-10/prepared/full-v3/plan.json` 绑定原夹具、schema、旧校验器、模板代码、原生探针及本卡执行器源码。每量化 474 请求，其中 4 条排除预热、20 条固定句、50 条无约束、200 条 wire、200 条 compact。`jobs.json.gz` 保存原始 prompt、预分词 ID、输入及上限。
- 全部 12 个 Python 文件 Ruff / strict mypy，6 项 unittest、3 个 PowerShell AST 检查通过。CPU 仿真只验证记录器/编码器，不证明 CUDA 回调或 schema 解码可行。
- 缓存中的旧 v1/v2 准备和编译记录保留；当前只用 v3。输入或源码变动时哈希检查拒绝运行，需重新准备。

```powershell
# 以下仅在新的完整安静窗口获确认后执行；输出目录必须不存在。
& D:/Dev/cache/P0-10/convert-venv/Scripts/python.exe spikes/carrier-engine/full_window.py `
  --build D:/Dev/cache/P0-10/injection-build-v3 `
  --prepared D:/Dev/cache/P0-10/prepared/full-v3 `
  --output D:/Dev/cache/P0-10/full-runs/run-01 `
  --quiet-window-confirmed
```

CPU 复现准备可用 `build_probe.ps1 -OutputDirectory <新缓存目录>` 与 `benchmark_native.py --prepare-only --prepared <新目录>`。不得覆盖旧记录；构建不会加载引擎。

## 固定的执行顺序与观测

1. 原生 Q4_K_M 配对探针，最多约 20 分钟，先验证回调节点确实存在、形状为预期 F32，异常即保留并停止，不继续完整 IRL。
2. Q4_K_M server，然后 Q8_0 server；不同时驻留。每个 server 仅 loopback、单 slot，8192 context、512 batch/ubatch、4 CPU threads、Flash Attention off、全 GPU offload、greedy、seed 731。正式请求禁用 prompt cache 和 context shift。使用准备好的 token ID，记录 native tokenizer 诊断。
3. 20 次 P0-07 固定句。原 P0-08 输入前 50 条加入无约束诊断，三臂交错；其余 wire/compact 交替，禁止事后挑样本。IRL 最多 2048 新 token，完整校验与原来源绑定检查分开统计。截断/缓存复用、HTTP/组件错误保留后停止；合规传输中的无效 JSON/schema 作为失败样本计数。
4. 各进程 WDDM 专用与共享显存时序，独立峰值相加。仅载体：GPT-SoVITS V5 未发现部署，未启渲染。没有合计同驻通过结论。

SSE 每事件记录从请求发出起的客户端时间；native timings 单列。`timings.cache_n` 是复用输入数，`prompt_n` 是实际处理输入数；`tokens_cached` 是最终 KV 长度，不拿它判断输入复用。EOS 前没有可观测文本/token 时，客户端 TTFT 为 null，不补估计值。

## 注入配对的边界

- 零起算 layer 17：`ffn_inp-17` 是旧位置；`l_out-17` 是整层位置。对两节点使用相同回调边界，比较同 e0 向量、同最后位置掩码，再比较整层位置的 all/last 掩码、原生广播与回调 all。未装回调的 plain 独立列出，避免把回调带来的计算分段差异误归到注入。
- 中/英/日三条固定输入，参考 greedy 轨迹至多 24 token。各比较臂沿同一参考 token 轨迹逐位置保存完整 logits、argmax 与实际喂入 token；这不是非零臂的自由生成。全零和撤除另做自由 greedy，与 plain token 比较。
- 每步保存两位置最后行注入前后激活；prefill 另存所有行，检查掩码。主实验每请求显式清 KV。库原生向量在同一上下文 A→B→零→A→清除，A/B 分别与 fresh context 配对；设置返回码和实际结果都记录。另有命名为 INVALID 的保留 KV 诊断，不能作为合法生成或性能证据。
- 三组 calm/excited 样本逐字取自 P0-07 `direction_probe.py`，分别提取两个位置的最后 prefill 激活、求 excited−calm 均值并单位化。幅值 1 是结构测量点，不是批准剂量。只评估技术提取可行性，不宣称四天性/全部概念的行为单调性、能力无损或迁移已通过。

完整窗口预计约 1–2 小时，取决于实际生成长度；不是已测完成时间。任何未运行项目保持「未实测」。所有 GPU 任务串行，期间不编译、不跑其他测试。结束后通知用户恢复 runner。
