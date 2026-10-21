接通本地语音识别与语言识别：舞台侧 AudioWorklet 输出 16 kHz/20 ms PCM，Electron utilityProcess 运行 Silero VAD + SenseVoice int8，并通过主进程把 voice.vad、voice.bargeIn、voice.recognized 和时延打点发到 AppBus。未实现 Echo Core 网关接入。

公开 FLEURS 测试集固定三语各 30 条：中文 CER 7.48%、英文 WER 8.13%、日文 CER 8.76%，LID 90/90。整仓 Vitest 459/459、lint/typecheck 通过；本机真实语音进程静音 CPU 60.92 秒均值 0.107%、最大 0.126%。完整本地 Electron 回归 4 通过，专门测量项在普通回归中跳过但已单独运行通过；附加 OS 快照初始化问题已如实记录，Windows CI 用标准取证流程重跑。

**部分完成 / needs-user：** SenseVoice 权重是 FunASR 自定义模型许可，等待用户决定；用户朗读录音 0/60，完整 150 条尚不能验收；VAD/打断物理延迟缺独立音节起止标注，未宣称达标。三语朗读表及外置音频目录已准备。所有模型和公开音频下载只在本机执行且校验 SHA-256，CI 不下载。

完整结果、失败记录、依赖与许可清单见 `reports/P4-01/REPORT.md` 和 `BLOCKED.md`。请 Claude 审查实现；不合并、不取下一张卡。
