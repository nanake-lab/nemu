接通本地语音识别与语言识别：舞台 AudioWorklet 输出 16 kHz/20 ms PCM，Electron utilityProcess 运行 Silero VAD + SenseVoice int8，主进程转发到 AppBus。本卡不接入 Echo Core 网关。

**草稿 / 部分完成 / needs-user 保留。** 用户要求整张卡完成后再审。2026-10-21 已决定 SenseVoice 现阶段继续用于开发，用户自行下载模型，不随应用分发，P7 再评估许可；manifest 保持 redistributable=false 和 FunASR 出处、模型名。

用户三语各 20 条文字稿与本机录音目录已建立，当前 0/60。独立 Qwen3 ForcedAligner 方法、固定版本/哈希、全部 60 条样本和起止点规则已预登记；只用于本机评测，不属于应用运行时。参考时标的 80 ms 网格及误差限制明确记录。未对齐不存在的录音，未宣称延迟达标。

此前公开 FLEURS 三语各 30 条：中文 CER 7.48%、英文 WER 8.13%、日文 CER 8.76%，LID 90/90。先前检查和静音 CPU 结果详见报告，不能替代本次最终提交的验收。

待用户录音后，登记输入、独立对齐、跑完整 150 条 CER/WER/LID，在安静窗口测 VAD 与打断延迟。全部验收有结果后，同步 main 并重跑检查，才转可审查交 Claude。不合并、不取下一张卡。

详见 reports/P4-01/REPORT.md、BLOCKED.md、ALIGNMENT-PREREGISTRATION.md 和 READING.md。
