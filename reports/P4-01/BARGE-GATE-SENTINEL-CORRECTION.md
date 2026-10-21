# P4-01 原生零值哨兵修正 — 2026-10-22

先前 gated 三组结果保留在 vad-diagnostic-gated.json，未作为验收或生产方案。实际构造日志 native-vad-zero-sentinel.log 证实：请求数值 0 后，C API 采用 minSpeech=0.25、minSilence=0.5 的默认值。因此该组三组没有执行预期的有效零样本延迟，不能用其结果否定持续窗门控本身。

固定版本 [C API GetVadModelConfig](https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/sherpa-onnx/c-api/c-api.cc) 通过 SHERPA_ONNX_OR 把数值零当默认值。现用非零 0.000001 秒传入；原生配置日志已验证保留该值，16 kHz 下乘积为 0.016、整型样本计数为 0。只修正 API 编码，门控仍须至少 3200 个确认样本（实际最早七窗 224 ms），短噪声／分离脉冲／reset 测试保持，并补样本换算回归测试。整仓 475 项通过。

同一固定三组后验阈值 0.5、0.35、0.2，正常结束 VAD 的 minSilence 仍 0.168；全部 60 条及冻结参考保持。新结果写 vad-diagnostic-gated-effective.json，不覆盖先前三组，不把离线诊断称实时验收。生产实现不改；不缩短契约 200 ms，不调整 300/250 ms 门槛。此修正及源码先提交后运行。
