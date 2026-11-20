重档。本卡包含 RTX 3080 10 GB 上的真实语音合成、取消和 30 分钟压力测量。

# P4-02 交付报告（2026-11-21）

**部分完成。** GPT-SoVITS V5 已接到默认本地语音服务。中文及英文各 20 句的 WebSocket 首包中位数分别为 **1,053 ms、759 ms**，均超过原卡的 250 ms。取消达标。压力测试跑满，内存稳定性仍待验证。PR #45 保持草稿。待决定的工作见 [BLOCKED](BLOCKED.md)。

## 从桌面使用的服务入口实测

沿用桌面守护使用的 `services/voice-sidecar/src/main.py`、HTTP 预热/健康检查和 WebSocket 合成接口。`GsvEngine` 默认在独立子进程中加载已核验的本地 V5 权重，用 CUDA FP16 推理，G2PW 固定到 CPU。英文词典改在预热时加载，上游文本输出在子进程内屏蔽。TypeScript 本地 provider 使用同一套契约。准备推理环境及参考音色见[服务说明](../../services/voice-sidecar/README.md)。

| 检查 | 实测 |
| --- | --- |
| 首包，中文 20 句 | P50 1,053 ms，P90 1,249 ms，最大 4,238 ms |
| 首包，英文 20 句 | P50 759 ms，P90 1,421 ms，最大 1,795 ms |
| 取消，首包前及收到首块后各 3 次 | 首包前 3 次均未收到音频。另 3 次各有 1 个在途块，最晚 18.89 ms 到达，连接均以 1000 关闭 |
| 压力，40 条固定中文或中英混合短句循环 | 连续 1,800.018 s，完成 1,856 次，40/40 句在压力阶段各自字节一致。首包 P50 658 ms，P90 914 ms，最大 2,234 ms，聚合 RTF 0.545 |
| 健康与恢复 | 首次预热 47.81 s。主动终止模型子进程后，HTTP 仍在线，返回 `WORKER_EXITED`、`ready=false`、`vram=null`。再次预热并收到完整语音耗时 28.22 s |
| 离线降级 | 实际默认进程在资源不全时可重复预热并返回缺失原因。校验覆盖损坏文件、DNS 拒绝及缺基础依赖的标准库健康入口 |

首包计时从客户端开始建立 WebSocket 到收到第一块可播放的 24 kHz float32 PCM。每句单独建连，中文与英文各用登记的合成测试参考音色。未计声卡起播，没有载体或舞台同驻。V5 整句生成后分块发送。原 Qwen 0.6B 初轮三语首包约 0.85 s，测试条件有别；云端适配器通过接口测试，尚无真实端点的同口径延迟数据。[历史对照](QWEN3-TTS-HISTORY.md)

## 三项归因

- **首次英文延迟。** 固定旧版输入的首次混合句在请求中导入 `text.english`，实测 4.233 s，其中 `g2p_en` 导入 2.856 s、`wordsegment.load()` 0.471 s，均计入前者。该句首次、复读的带探针整句时间为 6.402 s 和 1.076 s。11 月 6 日原请求仅有阶段时戳；当时 5.484 s 首声中的 3.973 s 落在前处理/组批，本次复现把主要的一次性开销定位到了英文词典和转换器。正式入口已将这段开销移入预热。
- **Private Bytes 中的 CUDA 贡献。** 对正式模型进程做了三次 Windows 分配区域快照，当时私有提交总量为 9.714–9.733 GiB。26 个可辨认的 CUDA/PyTorch CUDA DLL 映像的提交费用每次都是 **560.883 MiB**，均无查询失败。另用同一套 CUDA 构建、相同权重、FP32 和英文前端，各在全新进程加载一次。加载完成后 CUDA 路径为 **9.738 GiB**，CPU 路径为 **8.664 GiB**，净差 **1.074 GiB**。前一项是映像可直接归属的提交费用，后一项是切换设备的整进程净效应，两者有重叠。匿名分配没有调用栈，不能把其余提交逐字节分给驱动或模型，也不能倒推旧峰值 9.679 GiB 的精确组成。[Windows 区域提交量定义](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/ns-memoryapi-win32_memory_region_information)
- **首次 PCM 差异。** 冷中文及首次混合的首轮、复读四个哈希均复现了 11 月 6 日的原值。同句的语义 token 相同。逐层记录在上游声学编码器 `wns1` 的 TorchScript 门函数 `fused_add_tanh_sigmoid_multiply` 发现首个差异，该函数的输入逐字节相同，输出不同。单独关闭 cuDNN 后仍有差异。仅在诊断进程把这个函数换成源码中的相加、tanh、sigmoid 和相乘，两个 case 的首轮和复读 PCM 都逐字节相同。固定声学特征及 CUDA RNG 时，梅尔谱也可逐字节重放。两句时长保持 2.92 / 2.04 s，原始浮点 PCM 首轮与复读的差值信噪比为 17.45 / 29.68 dB。正式服务继续使用固定上游实现。人工听感未签发，配对 WAV 见[中文首轮](https://github.com/nanake-lab/nemu-evidence/blob/085e72122b3e9d7d54a52b8779e8e073026ed9e6/P4-02/continue-20261120/zh-first.wav)、[中文复读](https://github.com/nanake-lab/nemu-evidence/blob/085e72122b3e9d7d54a52b8779e8e073026ed9e6/P4-02/continue-20261120/zh-repeat.wav)、[混合首轮](https://github.com/nanake-lab/nemu-evidence/blob/085e72122b3e9d7d54a52b8779e8e073026ed9e6/P4-02/continue-20261120/mixed-first.wav)、[混合复读](https://github.com/nanake-lab/nemu-evidence/blob/085e72122b3e9d7d54a52b8779e8e073026ed9e6/P4-02/continue-20261120/mixed-repeat.wav)。

## 压力与显存

618 个压力阶段 WDDM 样本全部取到模型进程。专用显存在 2,180.5–2,222.5 MiB 范围内，首尾均为 2,202.5 MiB，共享显存始终为 72 MiB。覆盖预热、合成与恢复的采样中，专用显存最高为 2,514.5 MiB。

每次完成请求均记录服务进程树内存。Private Bytes 首尾增加 **46.70 MiB**，全窗线性斜率 **+24.85 MiB/h**，尾段 10 分钟 **+21.28 MiB/h**。尾段 HTTP 进程和模型子进程的斜率分别为 +17.67 和 +3.61 MiB/h。RSS 尾段发生工作集收缩，不能用它抵消私有提交量的增长。因此本次未签发“内存不持续增长”。逐进程、全窗及尾窗统计在[机器可读汇总](https://github.com/nanake-lab/nemu-evidence/blob/085e72122b3e9d7d54a52b8779e8e073026ed9e6/P4-02/continue-20261120/summary.json)。

## 验证与证据

- GPU 窗口为 Perth **2026-11-20 23:38:43 至 11-21 00:30:01**。独占标记由本卡取得并释放，开窗前已确认 Windows 队列与 runner、其他推理负载状态。一次接线试跑在 391 条压力请求后主动结束，以固定实现提交重新执行完整 30 分钟；两次原始记录都已归档。
- 原分支以 merge 提交 `955d82e0` 同步 main `da40fd06`。正式压力基于 `5a8dae20`。后续 `dbf3dc63` 修正 Linux 类型检查对 Windows DLL API 与动态 ORT 方法赋值的识别，Windows 分支仍做相同调用。恢复测试在新启动的子进程中覆盖了该改动。收尾时核对 main 至 `1a23a738`，增量是思考 demo、报告及研究设计，未改动语音契约、依赖、构建或测量输入。
- Windows 本机运行旁路 `ruff check .`、`mypy --strict src tests` 全部通过。旁路 pytest 53 项通过，1 项显式 GPU smoke 在离线测试中跳过；上述 GPU 结果来自单独的真实服务测量。旧汇总器 5 项边界测试通过。
- `pnpm tts:schema`、本地/云端 provider 与桌面守护定向测试（8 项）、`pnpm lint`、`pnpm typecheck`、`pnpm test`（631 项）通过。[产品代码提交的 Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/44768889041) 通过。草稿 PR 的 Windows 自托管作业跳过，Windows 汇总门禁未通过。
- 无新增第三方包或模型下载。本机复用此前核准的环境及 VoiceDesign 测试音色。P7 分发前需复核词频数据、派生词典、fastText 与动态链接组件。[依赖和许可](DEPENDENCIES.md)；[P7 待决表](gsv-v5-resume-09/P7-PENDING.md)。
- 本轮原件及 SHA-256 见[证据索引](EVIDENCE-INDEX.md#2026-11-21-正式-provider-与归因)。11 月 6 日的独立模型结果及历史失败仍见[旧汇总](gsv-v5-offline-20261107/RESULTS.md)和[上次报告](https://github.com/nanake-lab/nemu/blob/2c31dfbedeb44858c94e5d005a216b9be29fdb79/reports/P4-02/REPORT.md)。
