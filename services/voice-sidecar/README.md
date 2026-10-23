# TTS 旁路服务（P4-02）

FastAPI + WebSocket，运行 Qwen3-TTS Base；支持 `zh`、`en`、`ja`。当前交付已完成 CPU/模拟验证，真实 GPU 验收等待用户安静窗口，详见 `reports/P4-02/REPORT.md`。测试音色尚未生成，不能把现有测试结果解读为真实模型效果。

## 安装与启动

```powershell
uv sync --project services/voice-sidecar --locked
# 默认不安装 CUDA 推理栈；CI 使用这一配置。
# 实际推理环境可显式安装锁定的可选依赖：
uv sync --project services/voice-sidecar --locked --extra inference

# 显式下载：不会在启动或 CI 中自动下载模型。
python services/voice-sidecar/prepare_models.py --cache services/voice-sidecar/.cache/models --variant 0.6B

$env:NEMU_TTS_CACHE = (Resolve-Path services/voice-sidecar/.cache/models).Path
$env:NEMU_TTS_VOICE = 'C:\path\to\voice.json'
$env:NEMU_TTS_VARIANT = '0.6B' # 或 1.7B
uv run --project services/voice-sidecar python services/voice-sidecar/src/main.py
```

推理栈与 P0-07 对齐：torch/torchaudio `2.10.0+cu128`、transformers `4.57.3`、qwen-tts `0.1.1`。`config/models.lock.json` 的 `tts` 分区固定每个模型的修订、文件大小和 SHA-256。读取本地权重前逐文件校验；禁止自动回退到网络上的新修订。

默认只监听 `127.0.0.1:17863`。`NEMU_TTS_PORT` 仅用于隔离测试；桌面 supervisor 使用固定的默认端口。默认缓存/声音配置缺失时保持降级运行。连 FastAPI 等依赖也没有时，`src/main.py` 用标准库提供降级 `/health`，不崩溃重启。

## 接口

- `GET /health`：始终以 HTTP 200 报告进程存活。`ready` 区分能否合成，`status`/`reasons` 报告降级；含请求和实际模型变体、忙碌状态及 CUDA allocator 当前/峰值显存。allocator 不是 Nemu 合计显存。
- `POST /warmup`：离线加载并校验模型。生成参考提示和首轮推理预热由基准脚本的三语 warmup 句完成；该路由不隐式生成说话内容。
- `WS /synthesize`：每个连接一条请求，字段来自 `@nemu/contracts` 的 `ttsRequestSchema`。Python 读取该 schema 的生成版，`pnpm tts:schema` 检查漂移。最大文本 4096 字符。
- 二进制帧：24 kHz、单声道、little-endian float32。正常结束以 close 1000 表示。输入/音色拒绝用 1008，后端不可用或推理失败用 1011。客户端不能把异常结束当作成功投递。
- 取消帧：`{"cancel":true}`，也可关闭连接。服务停止发送，底层生成在下一个 talker hook 合作退出。取消期间推理 lane 保持占用，直到原线程退出；不会与下一条 CUDA 生成重叠。客户端收到 AbortSignal 后立即丢弃本地排队分块。
- 队列上限 8 块，推理串行；慢客户端触发背压。握手 10 秒、发送 2 秒有界；客户端总请求超时 120 秒。不记录完整文本、音频、密钥或远端错误正文。

1.7B 加载时可用显存不足 6 GB，或加载/合成遇到 CUDA OOM，会降至 0.6B，并在 health 报告。该 6 GB 是运行时保守准入值，不是验收阈值。合成已发出 PCM 后发生 OOM 时，当前句报错而不重播；下一句使用小模型。精确变体基准禁用回退，避免把 0.6B 数据算作 1.7B 数据。

qwen-tts 0.1.1 的公开 `generate_voice_clone` 会整句返回。本适配器为该锁定版本安装请求范围内的 talker forward hook，每 4 个 codec frame 解码一次、保留 25 个左上下文 frame；hook 在退出和取消时移除。私有输出布局有运行时检查和 CPU double 测试。真实音质与接缝尚未签发；2026-10-21 双变体实测延迟未达标，详见 reports/P4-02/REPORT.md；上游末尾的整句解码仍会执行，计入真实压力与耗时，不能据 CPU 测试断言性能达标。

## 音色文件

Base 不接收文字风格指令；`style` 精确索引对应语言的参考音频，不擅自替换成 neutral。示例形状如下，SHA-256 必须来自实际文件：

```json
{
  "id": "p4-02-synthetic-test",
  "references": {
    "zh": {
      "neutral": {
        "path": "zh.wav",
        "text": "参考音频中的准确文字",
        "sha256": "实际音频的64位摘要",
        "provenance": "VoiceDesign",
        "source": "固定修订的模型来源",
        "license": "Apache-2.0",
        "cloneAllowed": true
      }
    }
  }
}
```

三个语言分别登记。音频必须在声音文件的目录内，匹配 SHA-256，并标明 VoiceDesign 或明确允许克隆的数据集来源及许可。请求不能携带任意参考路径。P4-01 用户录音禁止用于这里。`design_test_voice.py` 仅生成虚构成人测试音色，不接受人类参考音频；产物存于本服务 `.cache/`，附 `manifest.json`，不作为 Nemu 定稿声音。

云端适配器 `CloudTtsProvider` 接收 `voiceId -> providerVoice + instructions[lang][style]` 指令表，由调用者从 voice.json 提供；开启 instructions 时缺少映射即报错。只做模拟验证，没有调用任何真实云端。API key 由调用者解密后临时提供，适配器不存储。支持 24 kHz int16 PCM 跨网络帧解码及取消。

## 验证与 GPU 待测步骤

```powershell
pnpm lint
pnpm typecheck
pnpm test
pnpm tts:schema
pnpm test:tts
uv run --project services/voice-sidecar ruff check services/voice-sidecar
uv run --project services/voice-sidecar mypy --strict services/voice-sidecar
```

GPU 测试默认明确跳过；不下载模型。人工显式启用 smoke 测试也必须满足安静窗口，且不能替代下列完整验收。

仅在用户确认 runner、Blender、其他测量已停止后设置安静窗口标记；先按 AGENTS 第 9 节安排时间。复用 P0-07 环境，只追加本服务纯 Python 依赖搜索路径，不修改其环境：

```powershell
$env:NEMU_P4_02_QUIET_WINDOW = 'user-confirmed'
$env:NEMU_TTS_PROBE = 'D:\Dev\nemu-p007\workers\model-adapter\probe'
$probePython = Join-Path $env:NEMU_TTS_PROBE '.venv\Scripts\python.exe'
& $probePython services/voice-sidecar/design_test_voice.py --cache services/voice-sidecar/.cache/models --output services/voice-sidecar/.cache/test-voice-v1
pnpm tts:bench --variant 0.6B --cache services/voice-sidecar/.cache/models --voice services/voice-sidecar/.cache/test-voice-v1/voice.json --output reports/P4-02/gpu-0.6B-v1
pnpm tts:bench --variant 1.7B --cache services/voice-sidecar/.cache/models --voice services/voice-sidecar/.cache/test-voice-v1/voice.json --output reports/P4-02/gpu-1.7B-v1
```

两个变体在新进程中顺序运行；各有三语 warmup、每语言 20 句、每语言 5 次取消、固定至少 1800 秒压力测试。全程保留载体、P0-07 的真实 GPU 渲染夹具，记录 WDDM 专用和共享显存、torch allocator、进程 RSS 与每次请求原始数据。已有证据目录不得覆盖；失败也保留。显存判定按 ADR 0012：两项观测峰值之和 ≤ 8 GB（9 GB 内保留 1 GB）；渲染夹具不是完整 Nemu 正式资产的最终整机验收。测完通知用户可恢复 runner。

RSS 通过 Windows `GetProcessMemoryInfo` 读取当前 Python 进程，不为每句生成启动 PowerShell 子进程。P4-02 自有 `bounded_memory_watch.ps1` 使用同一组 WDDM CIM 计数器和 JSONL 字段，逐次重建活动进程树，避免两小时重测累计已退出的瞬时 PID；P0-07 probe 文件保持只读。该采样仍是 WDDM provider 的离散观测，不代表连续峰值。

## 2026-10-21 实测状态与 SoX 边界

双变体串行测量已完成：首包均约 852–863 ms，未满足 250 ms；0.6B 合计显存 7.188 GB，1.7B 9.214 GB（超 8 GB）；取消停止出块通过。30 分钟均跑完，但 RSS 分箱中位数仍上升，不接受稳定性通过。原始数据与复算工具见 reports/P4-02/REPORT.md；已有结果目录不得覆盖。纯合成参考已归档 tests/fixtures/voice-design-v1，不是正式角色音色。

锁定 qwen-tts 0.1.1 的未使用 25 Hz 路径会间接导入 pysox 并尝试 sox -h。qwen_import.py 在服务自有边界禁用该能力，真实 pysox 不导入，Transformer/Combiner 明确抛错；仅允许锁定的 12 Hz 模型。禁止绕开 load_qwen 直接导入 Qwen；不修改第三方源码或全局 subprocess。audit_qwen_import.py 在真实导入时审计并阻断任何 SoX 启动，当前尝试数为 0。soxr/随包 libsndfile 仅按用户许可动态加载，P7 复核分发义务。

## 首包优化候选（正式重测前仍显式启用）

原 eager/4 帧方案与数据保持不变。候选仅在服务自有实例上包裹 forward，不修改第三方源码；锁定 qwen-tts 0.1.1、transformers 4.57.3、torch 2.10.0+cu128，没有安装 faster-qwen3-tts。后者当前 0.5.4 改用了 Transformers 5.x/qwen-tts-hf，与现有锁不兼容，见报告源码版本记录。

```powershell
$env:NEMU_TTS_CUDA_GRAPH = '1'
$env:NEMU_TTS_ASYNC_DECODE = '1'
$env:NEMU_TTS_FIRST_FRAMES = '2'
# 可选分段计时：不记录文本；每请求追加一行 JSONL，GPU event 不逐步同步。
$env:NEMU_TTS_TRACE_DIR = (Join-Path $PWD 'reports/P4-02/new-acceptance-traces')
# 随后按上节原命令、独占窗口、载体/渲染驻留正式测两个变体。
```

捕获共用一条 stream 和一个 graph 内存池，避免逐图创建 cuBLAS workspace；所有重放串行，跨图可见的输出张量保持引用，临时池不与 tokenizer stream 共享。predictor 的 15 个单步 graph 保留上游 generate 的采样/词表头/条件。talker 使用 256-token 固定 KV 存储、与上游相同长度和连续布局的注意力视图（它限制优化覆盖，不限制可接受输入）：每条请求前 16 帧可加速，总共最多缓存 48 个位置图；超预算或长上下文使用上游 eager，生成到图范围末尾时无损复制 KV 后继续上游 eager。上限只限制优化资源，不截断音频、不改模型变体或 512 帧生成上限。未采用固定填充 softmax 的候选，因为它在真实三语 codec 比较中失败。

tokenizer.decode 移到有界队列的独立线程/持久 CUDA stream，按 producer event 等待 codec 就绪；首块 2 帧，其余 4 帧，仍只保留 25 帧左上下文。取消期间停止投递，推理 lane 等生成与解码均退出后释放。上游末尾整句解码仍保留，单独记录耗时。

`traces.jsonl` 记录 prefill、talker 每帧、predictor 每步、每次解码的 CUDA/host 时间、帧完成、引擎投递首末块和音频秒数；`wire/wire.jsonl` 记录实际 `ws.send_bytes` 完成时刻，benchmark 另记客户端首末包。每条请求各占一行，避免压力测试为每次请求创建独立文件。实时率为音频时长除以首块到末块间隔，仅为报告指标，不新增门槛。冷图准备单独计时，不能把冷请求结果称为热身后延迟。

正式 benchmark 仅对 warmup 与每语种 20 个 latency 样本启用逐步 CUDA-event 计时；取消和压力阶段保留客户端首末包、音频时长、实时率、RSS 和 allocator 样本，不创建每帧/每 predictor CUDA event，避免诊断仪器本身成为长测的资源增长源。

后续压力重测：torch allocated 首末相同，20–30 分钟 RSS OLS 斜率 ≤12 MiB/小时。失败自动延长该轮到总计 2 小时，报告 110–120 分钟斜率，交用户判断；不追溯旧结果。正式窗口请为两变体可能各 2 小时预留时间。短探针 optimization_probe.py 的相同 64-token 贪心限制仅用于开发 codec 对照，不能替代正式各语言 20 句或压力测试；正式 benchmark 强制 generation_limit==512。

### 线程资源与捕获范围诊断（2026-10-23）

异步解码现在由 engine 级单线程 executor 执行所有请求；每条请求仍有独立的有界队列、取消标记与 producer events，但不重建原生线程。executor 与 CUDA stream 都跨请求复用；OOM 重建模型前先关闭 executor，等待解码退出。默认 graph 捕获范围仍为 16，可显式设置 `$env:NEMU_TTS_GRAPH_FRAMES='4'`（支持 4/8/16）评估较小范围；只减少加速覆盖，超范围仍按原上限 eager 生成，最大 48 个 talker 位置图和共享池不变。

`resource_probe.py` 仅用于组件对照：设置 `NEMU_TTS_RESOURCE_CASE` 为 `graph` / `async` / `first2` / `eager` / `combined` 后使用 benchmark 的 `--variant --probe --cache --voice --output` 参数。每次用新进程；统一 64 帧贪心输入、240 秒观察，记录 tracemalloc、RSS、host allocator、pool/stream、捕获数、包装深度和 hook 数，明确 `acceptance=false`。tracemalloc 会降低吞吐，斜率同时结合每请求增量解释；不得用它签发正式 20–30 分钟压力通过。

`graph_memory_probe.py` 比较 `NEMU_TTS_PROBE_GRAPH_FRAMES=16/8/4`、`NEMU_TTS_PROBE_POOL=shared/separate`，固定三语 64 帧输入并保存 codec/PCM 哈希。独立池仅用于反事实对照，不是服务默认配置。正式 benchmark 仍使用 512 帧上限和原 1800/7200 秒压力规则。本轮原始数据、首次渲染启动失败和 Blender 门禁中止均保留在 reports/P4-02 的资源诊断归档；详见 REPORT.md。
