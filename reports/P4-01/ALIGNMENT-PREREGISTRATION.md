# P4-01 独立参考时标与延迟测量预登记 v1

登记日期：2026-10-21（Australia/Perth）。当前用户录音 0/60；本文件及机器可读 alignment-preregistration.json 必须先提交，之后才生成参考时标和测延迟。任何改法必须先另记修订并提交，保留旧结果；不得看到延迟结果后换对齐工具、挑录音或改阈值。

## 工具与独立性

- 工具：官方 qwen-asr 0.0.6，Apache-2.0。官方 wheel 的 SHA-256 为 b9c55a38413298f3a990a4475467399daec6e8f4172363053fc42e2166c2dfd3。
- 模型：Qwen/Qwen3-ForcedAligner-0.6B，Apache-2.0；固定修订 c7cbfc2048c462b0d63a45797104fc9db3ad62b7。权重及 tokenizer/config 的逐文件 SHA-256 见 packages/voice/src/asr/alignment/manifest.json。
- 环境：独立 Python 3.12，qwen-asr 0.0.6、Torch 2.10.0+cpu、transformers 4.57.6（官方包要求），其余见该目录 uv.lock。CPU、float32、eager attention、2 线程、seed=0。不加载 CUDA，不使用 vLLM 或 FlashAttention，不向云端上传录音。
- 工具和模型仅用于本机评测，不作为应用运行时依赖或应用分发资源。模型放 `%LOCALAPPDATA%/Nemu/models/evaluation/qwen3-forced-aligner/<revision>/`，独立 Python 环境在同一评测缓存根下。
- 只调用 Qwen3ForcedAligner.align(audio, text, language)，text 取用户人工文字稿，lang 取预定的 zh/en/ja 标签。不得调用任何 ASR 生成参考稿，也不得引入 Silero 的分段、输出或 SenseVoice 的识别结果；不使用能量阈值。

## 样本与冻结

- 全部用户录音：zh/en/ja 各 001.wav–020.wav，共 60 条。唯一 ID、路径及原朗读提示哈希已经写入机器可读预登记；固定顺序 zh → en → ja，再按编号递增。公开 FLEURS 90 条只参与 150 条 CER/WER/LID，不参与这一次用户录音延迟主分析。
- 用户通知录完后先运行 pnpm asr:user。保留原始 WAV 和用户实际读法的文字稿，不按 ASR 输出改稿。
- 对齐前登记全部原 WAV 哈希、人工稿 UTF-8 文本哈希和规范 PCM WAV 哈希至仓库外 alignment-inputs.json；一旦冻结，改变任何输入必须保留旧包，说明原因并在再次测量前提交修订，不能静默覆盖。
- ffmpeg 只作 16 kHz 单声道 PCM16 转换，保留完整前后静音，不裁剪、不归一化响度、不作 VAD。对齐器与被测系统重放同一份规范 WAV。规范 WAV 和原稿、token 级对齐原始结果均放仓库外。

## 起止点规则

1. 保留对齐器全部 token/span 原输出。只把包含 Unicode 字母或数字（包括汉字、假名）的项视作发音单位；纯标点不构成起止点。
2. 语音起点 = 文本顺序中第一个发音单位的 start_time × 1000；末端 = 最后一个发音单位的 end_time × 1000。英文末词结束、中文末字结束、日文末词结束作为最后音节结束的独立模型估计。不得移到 VAD 输出处或人工向达标方向修正。
3. 空结果、NaN/Inf、非正时长、越界或非单调项、缺样本、稿件/音频哈希不匹配均保存失败并阻塞整套结果，不排除该条后计算“通过”。
4. 官方模型配置 timestamp_segment_time=80 ms。80 ms 是时间网格，不是边界真实误差的保证；即使输出三位小数也不报告毫秒级真实精度。名义延迟照固定点估计计算，距 250/300 ms 阈值不超过 80 ms 的案例全部单列，说明对参考误差敏感；不据此改变阈值、删除样本或更换工具。
5. 仓库外 timing-references.json 每条包含：ID、lang、规范路径/哈希、原音频哈希、人工稿哈希、speechStartMs、speechEndMs、annotation=qwen3-forced-alignment-v1。对应 meta 文件含方法提交、方法/模型清单/uv.lock 哈希、实际工具环境及 60 条覆盖情况。不存在音频时不能制造空的“有效”时标文件。

## 延迟重放与判定

- 对全部 60 条各重放一次，固定 16 kHz、320 个样本一块、20 ms 实时节奏，从文件第一个样本开始计时，包含文件前后静音。开头前创建会话、完成模型加载与进程检查；末尾补 1 秒零 PCM 让端点正常结束。
- 播放状态使用现有 stage.speech.started/done 契约的脚本化播放夹具，不引入 TTS/GPU。记录原始 VAD 与打断事件；这是规范 PCM 输入到本地语音事件的重放评测，不把它写成物理麦克风/驱动端到端时延。
- VAD 延迟 = 与本条最后语音段对应的 speechEnd 时刻 − 最后发音单位参考结束时刻。所有原事件都保留；没有 speechEnd 为失败，过早结束记录为负值并标为提前截断失败，不能只挑一个晚于参考的事件掩盖早截断。
- 打断延迟 = 第一条 voice.bargeIn 时刻 − 第一发音单位参考起点。缺事件、早于参考起点的事件均为失败；持续 ≥200 ms 的打断逻辑保持既有独立单测。打断延迟上限仍是 250 ms。
- 主判据：所有 60 条都具备有效参考/结果；VAD 中位延迟 ≤300 ms，打断每条 ≤250 ms。另按语言报告 VAD 中位、最大值、全部打断值和缺失/提前截断数。不只报告平均或最佳样本。
- 与 AGENTS.md 第 9 节一致：测前确认无 blender.exe、Electron/Playwright 测试、GPU 测量或 Runner.Worker；空闲 Runner.Listener 可以在线。GPU 工作的跨进程识别不足时先与用户协调，不能宣称仅查 Electron/Blender 就证明无 GPU 任务。
- 测中进程检查必须异步进行，不能在每秒的 PCM 发送循环里同步启动 PowerShell 而阻塞时间轴。任一 PCM 块相对预定发送时间迟到 >20 ms，或出现新重负载，则保存原失败并判该次测量无效，等窗口恢复后完整重跑，不用追赶式突发输入伪装实时。
- 对齐进程和模型必须退出后才启动被测系统测量。先冻结参考，再看被测输出；不能把对齐器并行放在延迟测量窗口内。

## 来源与状态

- 官方工具与实现：https://github.com/QwenLM/Qwen3-ASR
- 官方模型卡及配置：https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/tree/c7cbfc2048c462b0d63a45797104fc9db3ad62b7
- 固定工具分发：https://pypi.org/project/qwen-asr/0.0.6/

本轮只预登记与校验输入规则，不加载模型、不对齐不存在的用户录音、不运行延迟验收。PR 保持草稿和 needs-user。全部验收有结果后再 fetch/merge main、重跑检查，才转可审查。

执行顺序：用户通知录完 → pnpm asr:user → pnpm asr:align:models（本机安静窗口内安装独立环境并校验模型）→ pnpm asr:align → pnpm eval:asr → 对齐器退出并重新确认安静窗口 → pnpm asr:latency。任何真实运行失败保留原始日志，不在看到被测延迟后修改本方法。
