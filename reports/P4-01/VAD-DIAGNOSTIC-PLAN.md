# P4-01 VAD 配置诊断预登记 — 2026-10-22

已知完整实时原配置结果 FAIL，VAD 中位 320.1 ms、59/60 打断超过 250 ms。以下是故障诊断，不是验收重试，不得以离线数值签发实时通过。

固定同一 Silero 权重、全部 60 个冻结 PCM 和独立时标，顺序不变，不剔除样本。使用与运行时相同 320 样本投递、512 窗；只以累计样本时钟记录原生 isDetected 变化，不运行 ASR、不等待实时、不包含 IPC，保留全部结果和负值。五组全局配置都运行：

| ID | 打断后验阈值 | 原生 minSpeech 秒 | 结束 minSilence 秒 |
| --- | --- | --- | --- |
| baseline | 0.5 | 0.2 | 0.2 |
| window-accounting | 0.5 | 0.168 | 0.168 |
| posterior-035 | 0.35 | 0.168 | 0.168 |
| posterior-020 | 0.2 | 0.168 | 0.168 |
| shorter-endpoint | 0.35 | 0.168 | 0.128 |

正常 VAD 后验阈值保持 0.5、minSpeech 0.032；打断 minSilence 保持 0.032；maxSpeech 20 秒。0.168 只是待核验的首窗计数补偿候选：原生从第一阳性窗末记 temp_start，再等指定时间；原参数需第一窗再加 ceil(3200/512) 个窗，共 256 ms；候选为七个窗约 224 ms。没有把契约 200 ms 改为 168 ms；生产代码保持原参数，候选不能在没有有效持续时长与短噪声拒绝验证的情况下投入运行。

报告每组的中位结束延迟、超过 250 的打断数量、负值数量与全部逐条值。任何候选若不能同时消除超限／提前／缺事件，不称为解决方案。即使离线看似通过，也必须保留配置来源并补连续语音／噪声验证后，重新做完整实际实时测量；不改 300/250 ms 门槛或独立参考。

实现依据为固定 v1.13.8 的官方 [silero-vad-model.cc](https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/sherpa-onnx/csrc/silero-vad-model.cc)。其概率／当前帧细节未由该版本 [Node addon](https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/scripts/node-addon-api/src/vad.cc) 导出；不能把完整 1340 ms 延迟全归因于一个窗，声学与时标质量还需分开诊断。
