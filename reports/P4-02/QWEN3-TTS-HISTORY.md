# Qwen3-TTS 历史证据（不代表当前目标模型）

旧实现、测量与失败原件完整保留在[私有历史原文](https://github.com/nanake-lab/nemu-evidence/blob/b526099269e6a31c2fe4f0c89e2c55946a893b23/P4-02/QWEN3-TTS-HISTORY.md)，逐文件哈希见 [EVIDENCE-INDEX](EVIDENCE-INDEX.md)。2026-10-26 后本卡目标是 GPT-SoVITS V5；正式 provider 尚待替换。

原 Qwen 初轮三语首包 P50 约 0.85 秒，未达旧 250 ms 目标；1.7B 原合计显存 9.214 GB 超过旧测量预算；原 30 分钟 RSS 持续增长未获接受。后续优化中的长输出显存超线、正式压力中断/栈溢出及 RSS 归因和重测全部保留在原件，未因迁移改判为通过。旧阈值和测量配置不用于新 GSV 独立测量。
