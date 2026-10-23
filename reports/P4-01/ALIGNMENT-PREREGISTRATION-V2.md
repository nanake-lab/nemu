# P4-01 独立时标预登记 v2 — 2026-10-21

本版本在任何 VAD/打断延迟测量前提交。v1、完整 60 条严格诊断、粗定位失败和用户确认记录都保留。不是根据延迟成绩换参考；目前尚无延迟成绩。原始工具、模型、CPU 参数、60 个固定 ID、数据顺序、250/300 ms 阈值、静音 CPU ≤2% 均不变。

## 修正与理由
v1 对每个文字单位强制 end > start 是执行方自行加入的校验，不是任务卡或 CONTRACTS 的要求。全量诊断发现 22 条含同点词元；模型只有 80 ms 网格，短单位可能映射到同一点。保留原 v1 函数与原测试，v2 另立函数和版本，不把旧失败改写成通过。

v2 对所有文字单位保留原始 start/end 点并检查有限、非负、end >= start、序列非降、不越界；整句首单位 start 至末单位 end 必须正时长。零时长词元不删除、不挪动，逐条记录到 metadata.quantizationWarnings；整句空、全点重合、负值、逆序、越界或缺任何样本仍阻塞。起点仍取第一个发音单位 start，末端仍取最后单位 end，不用能量阈值、Silero、SenseVoice 时间点或被测延迟修正它们。

v1 另有 1 条最后点比 4.36 秒文件末端多 40 ms；不裁到文件末端。v2 在全部 60 条规范 WAV 末尾统一追加 1 秒数字静音，保留所有原始样本，既不裁切、也不改增益或起点。规范输入在 alignment-input-v2/，快照为 alignment-inputs-v2.json；原 v1 输入和快照保留。独立对齐器和被测系统必须使用相同的 v2 WAV，不能只给对齐器加尾巴。原实时重放额外 1 秒尾部等待不变。

## 输入、执行和判据
文字使用用户于 2026-10-21 明确验证的逐句稿，完整保留重读；来源与验证哈希见 user-verification-20261021.json。文字草稿来自被测识别器，经过用户验证；用户集不能称独立盲测。时间点始终由独立 Qwen 工具产生。

工具 qwen-asr 0.0.6 / Qwen3-ForcedAligner-0.6B，Apache-2.0，固定模型修订 c7cbfc2048c462b0d63a45797104fc9db3ad62b7；manifest 与独立 uv.lock 不变。CPU float32/eager、2 线程、seed 0，官方 raw span 和全部质量标记留仓库外。参考 annotation=qwen3-forced-alignment-v2，method=P4-01-forced-alignment-v2。全部 60 条都必须有效才生成 timing-references.json；meta 包含方法提交、文件和输入哈希。

保留 80 ms 网格及未经校准误差的限制。报告全部临界案例；不得从小数位声称真实毫秒精度。VAD 用最后正常 speechEnd 减参考末端；提前截断和缺事件为失败。打断用第一 voice.bargeIn 减参考首点；负值和缺事件为失败。VAD 中位 ≤300 ms，打断每条 ≤250 ms。20 ms 实时 PCM 重放、迟到 >20 ms 判无效、异步资源监测和完整原始事件均沿用 v1。

对齐完成并退出模型后，确认没有 Blender、Electron/Playwright、GPU 测量或 Runner.Worker 才开始 CPU 延迟测量；空闲 Runner.Listener 可在线。本任务不运行 GPU 模型。独立工具精度限制不构成放宽阈值的理由。
