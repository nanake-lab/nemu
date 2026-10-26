# P4 声音输入与输出（第 4–11 周）

出口条件：并入 N2。中、英、日三种语言的识别、合成、口型都可用；语音输出管线能把任意来源的输出块变成她的声音和口型，并如实回报投递结果。在 P6-02 接上 Echo Core 之前，用测试输出源驱动。

---

## P4-01 语音识别与语言识别

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P1-02
- **工期：** 5 天
- **分支：** `task/P4-01-asr`
- **目录：** `packages/voice/src/asr/`、`packages/voice/web/`（麦克风采集模块，供舞台导入）

### 交付物
- `packages/voice/web/mic.ts` + `mic-worklet.ts`：
  - `getUserMedia`（开启回声消除、降噪、自动增益）→ AudioWorklet 重采样到 16 kHz → 每 20 ms 一块 Int16 PCM → 写入麦克风通道；
  - 舞台只需要 `import { startMic } from '@nemu/voice/web'`。
- 语音工具进程（Electron utilityProcess）：
  - sherpa-onnx Node 绑定 + Silero VAD + SenseVoice（int8）；模型按 `config/models.lock.json` 下载并校验哈希；
  - 自动语言识别，输出 `zh`、`en`、`ja`（其他语言按最接近的处理并标记）；
  - 两种模式：按键说话（`stage.mic.set` 控制）和常开监听；可选唤醒词「Nemu」（默认关闭）；
  - 打断判定：舞台正在播放时，VAD 检测到持续 ≥ 200 ms 的语音 → 发出 `voice.bargeIn`；
  - 识别完成 → 发出 `voice.recognized`（CONTRACTS.md 4.1），由主进程转成观察事件；
  - 打点：`vadEnd`、`asrDone`。
- `AsrProvider` 实现，以及一个云端识别适配器的接口桩（不需要接通）。

### 验收
- 识别错误率：每种语言 50 条测试录音（你录 20 条日常说话 + 30 条公开数据集样本），中文、日文按字、英文按词计，各 ≤ 10%（完成标准第 11 项的一部分）。
- 语言识别：150 条录音中语言判定正确率 ≥ 98%；中英夹杂的句子按主要语言判定。
- VAD 结束判定的中位延迟 ≤ 300 ms（从录音里最后一个音节结束算起）。
- 打断：播放中插话，从开口到 `voice.bargeIn` ≤ 250 ms。
- 语音进程常驻时 CPU ≤ 2%（静音状态）。

> **2026-10-22 追加（ADR 0017）：** VAD 结束判定的目标改为中位 ≤ 350 ms；打断改为从开口到 `voice.bargeIn` 中位 ≤ 300 ms，持续 200 ms 的门限不变，并报告提前误触发的条数与比例（只报告）。

---

## P4-02 TTS 旁路服务（三种语言）

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P0-01
- **工期：** 5 天
- **分支：** `task/P4-02-tts-sidecar`
- **目录：** `services/voice-sidecar/`、`packages/voice/src/tts/`

### 交付物
- Python 服务（FastAPI + WebSocket）：
  - `POST /warmup`、`GET /health`（含显存占用）。
  - `WS /synthesize`：输入 `{ text, lang, style, voiceId, sampleRate: 24000 }`，流式返回 float32 PCM 分块；支持取消。
  - 模型：Qwen3-TTS **Base**（0.6B 与 1.7B 可切换），用参考音频克隆固定音色。**Base 模型不接受文字指令**，所以 `style` 的处理方式是：按 `voice.json` 找到这种语言、这种语气的参考音频，用它做克隆参考。参考音频由 P4-03 提供；在那之前用一段测试音色。
  - 显存保护：可用显存不足时自动降为 0.6B，并在 `/health` 中报告。
- 主进程守护逻辑接入 P1-02 的进程守护。
- `TtsProvider` 实现：本地旁路服务；另做一个云端适配器（OpenAI 兼容接口即可）用于回退。云端支持文字指令时，按 `voice.json` 的指令表把 `style` 翻译成指令。

### 验收
- 预热后，三种语言各 20 句短句的首包延迟 p50 ≤ 250 ms（在你的 3080 上，模式 A 的显存配置下测）。如果达不到，记录实测数据，并给出 0.6B、云端两种方案的对比。
- 30 分钟连续合成压力测试：显存和内存不持续增长。
- 取消：发出取消后 50 ms 内不再收到分块。

### 禁止
- 不得克隆任何真实人物的声音；测试也只能用 VoiceDesign 生成的音色，或开源数据集中明确允许的声音。

> **2026-10-26 追加（里程碑重排）：** 按 ADR 0027 改为 GPT-SoVITS V5（v5turbo，零样本），不再要求三种语言；旁路服务的取消与健康检查代码保留；显存与首个声音的延迟在 3080 上重测，P4-02 原定的 250 ms 首包另定目标。PR #45 按此恢复。

---

## P4-03 Nemu 声音设计（一个声音，三种语言）

- **建议执行者：** Codex 生成候选，你挑选
- **难度：** 低
- **里程碑：** N2 她在了（排练版）
- **依赖：** P4-02、P2-01（`06-persona-seed.md`）
- **工期：** 4 天（你约 2 小时）
- **分支：** `task/P4-03-voice-design`
- **目录：** `tools/voice-design/`、`content/_candidates/voice/`

### 步骤
1. Codex 根据人设种子写出 5 组声音描述（年龄感、音色、语速、能量感各有不同），每组用 Qwen3-TTS VoiceDesign（1.7B）生成 4 个候选，共 20 个。
2. 每个候选念同样的测试台词：每种语言 3 句（平静、开心、小声说话），共 9 句。Codex 把它们做成一个本地 HTML 对比页面。
3. 你选出 2–3 个；Codex 按你的意见调整描述，再生成一轮。
4. 你定稿 1 个声音描述。
5. 语气参考音频：用定稿的描述，让 VoiceDesign 为每种语言、每种语气（12 种表演表情中常用的 6 种 + 4 种 voice 风格）各生成 3 段 8–12 秒的候选。
   - 用说话人嵌入检查每段与该语言「平静」参考的相似度，低于 0.80 的丢掉；
   - 剩下的由你每种挑 1 段；
   - 语气相近、实在挑不出差别的，可以合并成同一段。
6. 用 Base 模型分别以这些参考音频合成同一组测试句，确认语气能被带出来。
7. 写好云端指令表（每种语气一条文字指令），留给支持指令的云端 TTS 使用。

### 交付物
- `content/_candidates/voice/voice.json`：音色 id、定稿的声音描述、每种语言 × 每种语气对应的参考音频、云端指令表。
- `content/_candidates/voice/ref/<lang>/*.wav`。
- 你签字后，由你把它们移到 `content/nemu/voice/`，并在 `content/nemu/manifest.json` 中登记：来源为 VoiceDesign 生成，原创。

### 验收
- [ ] 你签字确认声音，并确认三种语言听起来是同一个人。
- [ ] 你确认各种语气听起来都对。
- 一致性：
  - 同一句话用「平静」参考合成 10 次，两两余弦相似度最小值 ≥ 0.85；
  - 三种语言的「平静」参考彼此之间、各语气与本语言「平静」之间，相似度都 ≥ 0.80（完成标准第 11 项的一部分）。

> **2026-10-26 追加（里程碑重排）：** 拆成两步（ADR 0027 第 5 条）：过渡期只准备一组同一声音、不同情绪的参考音频（来源由你选，许可要清楚，不入仓库）；她最终的音色等 Breeze 可用后再选。

---

## P4-04 口型同步（运行时）

- **建议执行者：** Antigravity
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P3-01
- **工期：** 3 天
- **分支：** `task/P4-04-lipsync-runtime`
- **目录：** `packages/avatar-runtime/src/lipsync/`

### 交付物
- 舞台音频链：音频通道上的 `stage.speech.begin` / `chunk` / `end` → AudioBuffer 队列 → 播放；同一条链路上接入 wLipSync 节点。每句真正开始播放时发出 `stage.speech.started`，并把这句的 `cues` 交给物理状态机（P3-04）触发。
- 按句子的 `lang` 切换 `profile.lipSyncProfiles` 中对应语言的档案。
- 口型映射：wLipSync 的 A、I、U、E、O 分数 → `profile.visemeMap` 指定的表情权重；做平滑（上升约 40 ms，下降约 80 ms）；音量低于阈值时闭嘴。
- 与 L3 面部层对接：说话时嘴部通道优先。
- `stage.speech.stop`：按 `fadeMs` 淡出并清空队列；每句结束发出 `stage.speech.done`，带 `interrupted` 和实际播出的字数 `playedChars`。
- 打点：`audioStart`。

### 验收
- 偏差测试（完成标准第 10 项）：每种语言合成 20 句话 → 录下音频包络和嘴部张开度曲线 → 互相关求偏移，平均 ≤ 50 ms、最大 ≤ 80 ms。
- 打断：发出 `stage.speech.stop` 后 50 ms 内音量降到 -40 dB 以下；`playedChars` 与实际播出位置一致（误差 ≤ 1 个字或 1 个词）。

---

## P4-05 口型校准（三种语言）

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P4-03
- **工期：** 2 天
- **分支：** `task/P4-05-lipsync-calibrate`
- **目录：** `tools/lipsync-calibrate/`

### 目标
wLipSync 的校准档案原本要在 Unity 里用 uLipSync 生成。为了不引入 Unity，用 Node 实现同样的 MFCC 校准，并为三种语言各生成一份档案。

### 交付物
- `pnpm lipsync:calibrate --lang <zh|en|ja>`：
  1. 用 Nemu 的定稿声音合成该语言里对应 a、i、u、e、o 的长元音和若干常用词语；
  2. 按 uLipSync 的算法（参数与 wLipSync 读取的档案格式保持一致）计算每个音素的 MFCC 平均值；
  3. 输出 `content/nemu/lipsync/<lang>.json`。
- 算法一致性测试：同一段音频分别交给 wLipSync 的 WASM 处理和本脚本处理，比较两边算出的 MFCC 向量，误差在 5% 以内。如果 wLipSync 不对外提供 MFCC 数值，改为端到端验证：使用新档案时，五个元音长音的识别准确率 ≥ 90%。

### 验收
- 算法一致性测试通过。
- 用新档案跑 P4-04 的偏差测试，三种语言的结果都不差于默认档案。

> **2026-10-26 追加（里程碑重排）：** 口型校准只做中文和英文（ADR 0027 第 4 条）。

---

## P4-06 语音输出管线

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P4-01、P4-02、P4-04、P0-02、P4-07（2026-10-26 加：先有声音契约再接两层）
- **工期：** 5 天
- **分支：** `task/P4-06-speech-output`
- **目录：** `packages/voice/src/output/`

### 目标
把输出块（CONTRACTS.md 4.3 的 `output.chunk`）变成她的声音、口型和表演，并如实回报实际说出了什么。在 P6-02 接上核心之前，输入来自一个测试输出源。

### 交付物
- **输出块处理：**
  - 用 `createPerformanceParser()` 按三语规则切句，得到带 `lang` 的 `PerformanceSentence`；
  - 按 `ttsStyle` 和 `lang` 调用 TTS；
  - 经音频通道发送 `stage.speech.begin`（带这句的 `cues`）、音频分块、`stage.speech.end`。
- **投递记账：**
  - 汇总 `stage.speech.done`，按输出块计算 `delivered_chunks`、`played_chars` 与状态（DELIVERED、PARTIAL、CANCELLED、UNKNOWN_DELIVERY）；
  - 交给终端客户端作为 `speech_delivery` 执行结果（P6-02 之前写入本地日志）；
  - 已经播出的内容不撤回。
- **打断：**
  - 收到 `voice.bargeIn` 时取消未合成的句子，发送 `stage.speech.stop`；
  - 通知终端发送 `output.cancel`，并补发投递结果。
- **测试输出源：** `pnpm dev:speak <脚本>` 读取脚本化的输出块，可以模拟流式到达、延迟、取消，三种语言都有样例。
- **延迟打点：** 汇总为 `metrics.latency`，在调试面板显示。

### 验收
- 30 条脚本化输出（三种语言各 10 条）：表演提示与所在句子的偏差 ≤ 1 句（完成标准第 12 项，按舞台日志自动核对）。
- 打断后 300 ms 内停止；投递结果中的已播出范围与实际一致（完成标准第 9 项）。
- 网络或 TTS 故障注入：无法确认是否播出的块标为 `UNKNOWN_DELIVERY`，不会被标为已投递。

> **2026-10-26 追加（里程碑重排）：** 按 V2-106 接两层：A 的语气指令由适配层落实为选一段参考音频，标签在过渡期丢弃并记录；B 的语速倍率与增益作用在合成出的声音上；B 不写进 A 的指令。原验收中的三种语言改为中、英。

---

## P4-07 声音读出与风格契约（V2-106）

> **高难度卡（契约变更）：** 修改 `packages/contracts/` 与 `core/contracts/`，依据 ADR 0027 第 3 条与差异登记 V2-106。一方实现，另一方交叉审查；PR 标 `needs-user`，由用户合并。用户 2026-10-26 批准开这张卡（里程碑重排）。

- **建议执行者：** Codex
- **难度：** 高
- **里程碑：** E3 她有声音
- **依赖：** P4-02、P5-12
- **工期：** 3 天
- **分支：** `task/P4-07-voice-readout`
- **目录：** `packages/contracts/`、`core/contracts/`、`core/crates/core-kernel/src/modulation/voice/`
- **必读：** ADR 0027；差异登记 V2-15、V2-106；`docs/CONTRACTS.md` 第 3、7 节；P5-12 卡中的读出函数

### 交付物
- **B 身体层：** 固定、带版本的声音读出函数，把快状态 x 换算成语速倍率 s（无量纲，中性 1）与整句增益（dB，中性 0）；m = 0 时取中性；取值范围属于函数版本；音高本版本不用。
- **A 表达层的契约：** `TtsProvider.style` 的后继：一条自然语言的语气指令与内嵌事件标签；协议 p 只列可用标签与语法；未知或写错的标签丢弃并记录，不替她修改。
- **叠加语义：** B 作用在 A 合成出的声音上（V2-106），A 的指令里不写入 B。
- 旧消息与旧字段按 ADR 0007 只增不改。

### 验收
- 固定文字、改变 x：s 与增益按读出函数单调变化；m = 0 时为中性。
- 黄金向量与三种语言生成一致；兼容检查 0 违例。
- [ ] 你批准这次契约变更（阻塞）。
