# P4-01 独立参考时标稳定性预登记 v1

登记日期：2026-10-22。先提交本方法和脚本，再执行 120 次对齐。此轮只检验独立工具的上下文敏感性，不生成或替换正式 timing-references.json，不运行被测 VAD / ASR，不读取 latency.json 或被测事件时间。

- 工具：已有 qwen-asr 0.0.6（Apache-2.0），Qwen/Qwen3-ForcedAligner-0.6B（Apache-2.0），修订和权重 SHA 沿用 alignment/manifest.json。仅本机评测，D 盘已有权重，不新增运行时依赖、不下载新权重。
- 样本：全部 60 条用户已确认的实际文字稿，含重读；顺序、输入清单 SHA 和六个锚点 SHA 见 reference-stability-plan.json。每种语言固定用 public-<lang>-001 和 public-<lang>-002，来自已登记 CC-BY-4.0 FLEURS 公开集。没有按结果挑选锚点。
- 输入：冻结的 alignment-input-v3 音频逐字节保留，包含固定首尾各 1 秒数字静音。每个锚点独立转换为 16 kHz 单声道 PCM16，不裁剪、不调增益。锚点后接固定 12,800 样本（0.8 秒）数字静音，再接目标音频。
- 对齐：锚点和目标分别用各自登记语言的官方 tokenizer，FixedSegments 拼接，保留全部词元；full_text = anchor + “。\n” + target。CPU float32/eager、2 线程、interop 1、seed 0、确定性算法、离线模型，与 V4 一致。每个上下文运行一次。
- 取点：仅目标段首个词汇单位的起点、最后词汇单位的终点；减去精确前缀样本数 / 16,000。沿用 boundaries_v2 的有限值、非负、单调、界内及整段正长度检查。零时长单位全部记录，不丢弃、不夹紧。
- 稳定性规则：两个上下文均数值有效，并且起点差与终点差都不超过 80 ms（一个模型网格；只容许 1e-8 ms 浮点误差）。80 ms 是复核门槛，不是工具准确度保证。全部 60 条均满足才能称本轮上下文稳定；任何无效或超限均保留并判不稳定，不自动采用平均值、最早/最晚值或较有利的结果。
- 输出：所有上下文音频、文字和逐单位原始结果只写 D 盘仓库外。仓库报告仅含样本编号、数值、哈希、方法提交和诊断。即使全部稳定，也不证明物理真实起止点准确；另行决定是否有足够依据更换参考协议，并在下一次被测测量前提交决定。
- 资源：CPU 本机独占重负载，启动前与运行中检查 Blender、Electron / Playwright、GPU 测量、Runner.Worker；空闲 Runner.Listener 可保留。检测冲突则终止，该次不形成完整稳定性结论。原 V4 和所有延迟失败证据保持不变。

运行：pnpm exec tsx packages/voice/src/asr/alignment.ts stability
