# P4-02 本地语音合成服务

默认本地 provider 为 GPT-SoVITS V5（v5turbo）。合成器接收中文（可夹英文）和英文。WebSocket 返回 24 kHz、单声道、小端 float32 PCM。模型先合成整句，再由旁路服务分块发送。验收数据及未完成项见 [P4-02 报告](../../reports/P4-02/REPORT.md)。

## 在 Windows 上启动

使用已准备好的 Python 3.12 推理环境、本地权重、NLTK 资源和 FFmpeg 动态库。上游代码固定在 `04a44eb977586645f33594b72af13a035a2056c6`，预训练权重固定在 `8f8857563c040d542de108a2c18b0425abd6c9c5`。版本和许可见 [依赖清单](../../reports/P4-02/DEPENDENCIES.md)。预热时按 [SHA-256 清单](src/nemu_voice_sidecar/gsv-v5-files.json) 校验模型与关键源文件。

以下示例在仓库根目录运行，使用本卡已有的缓存和合成测试音色。桌面守护进程从服务目录的 `.venv` 选取 Python，缺少时使用 `PATH` 中的 Python。桌面启动时应使用同一套推理依赖，并继承以下环境变量。

```powershell
$cache = 'D:\Dev\cache\p4-02'
$env:NEMU_GSV_ROOT = Join-Path $cache 'gpt-sovits-v5-official'
$env:NEMU_GSV_FFMPEG_BIN = (Get-Item "$cache\gsv-v5-resume-04\ffmpeg\*\bin").FullName
$env:NLTK_DATA = Join-Path $cache 'nltk_data'
$env:NEMU_TTS_VOICE = Join-Path $cache 'gsv-v5-test-voice\voice.json'
$python = Join-Path $cache 'gsv-v5-venv\Scripts\python.exe'
& $python services/voice-sidecar/src/main.py
```

## 服务接口

- `GET /health` 返回进程存活、`ready`、`modelLoaded`、`busy`、支持语言和降级原因。加载后 `vram` 是模型子进程在预热或最近一条成功请求结束时的 PyTorch allocator 快照，包括当前及峰值字节。
- `POST /warmup` 校验本地资源，加载 CUDA V5 模型和英文词典，并确认 G2PW 使用 CPU。推理进程独占上游工作目录、日志输出和网络限制。缺依赖或资源时 HTTP 进程保持存活，可以在补齐后重试。缺少基础 HTTP 依赖时由标准库入口提供降级健康检查。
- `WS /synthesize` 每连接接收一条 `@nemu/contracts` 定义的请求，文本最多 4096 字符。按 `voiceId/lang/style` 选择登记的参考音频，核对来源、许可及哈希。测试音色是 VoiceDesign 合成的虚构声音。中英参考音频和对应文字分别登记在 `voice.json`。
- 正常完成和 `{"cancel":true}` 使用关闭码 1000，输入拒绝用 1008，推理失败用 1011。收到取消后停止发送 PCM。已开始的整句计算退出前继续占用推理通道。客户端收到 AbortSignal 后丢弃排队的音频。
- 服务默认监听 `127.0.0.1:17863`。`NEMU_TTS_PORT` 可用于隔离测试。云端适配器由调用方显式配置。

## 验证

```powershell
pnpm tts:schema
pnpm test:tts
pnpm exec vitest run packages/voice/src/tts/providers.test.ts
```

真实 GPU smoke 需在项目约定的安静窗口中显式开启 `NEMU_RUN_TTS_GPU_TESTS=1` 和 `NEMU_P4_02_QUIET_WINDOW=user-confirmed`，并使用已准备好模型的 Python。正式测量方法和原始记录见任务报告。

`inference` 可选依赖组、`prepare_models.py`、`benchmark.py` 和需要传入 `--variant` 的 `pnpm tts:bench` 保留给旧 Qwen 对照实验。历史结果见 [Qwen 对照](../../reports/P4-02/QWEN3-TTS-HISTORY.md)。
