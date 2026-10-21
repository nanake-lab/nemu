# P4-02 TTS 旁路服务（三种语言）— 报告

## 结果

部分完成，真实验收未通过。2026-10-21 用户授权 GPU 安静窗口后，已解决 SoX 导入调用，生成纯合成三语参考，分别以新进程串行完成 0.6B 和 1.7B 的三语各 20 句、真实链路取消和至少 30 分钟压力测试。两者首包 p50 均超过 250 ms，1.7B 合计显存超过 8 GB；RSS 仍有上升趋势，不能签发稳定性通过。详见 [BLOCKED.md](BLOCKED.md)。

PR [#45](https://github.com/nanake-lab/nemu/pull/45) 保持草稿及 needs-user，不转可审查、不合并、不取下一张卡。已 fetch 并 merge 最新 origin/main，合并提交为 9c986a6；必读规则与 P4-02 卡未变化。下列静态/功能检查在该合并后重跑，GPU 数据来自合并前同一 P4-02 运行实现；main 没有改动这部分实现。

## 交付物

- `services/voice-sidecar/`：FastAPI health/warmup、WebSocket float32 PCM、取消、有界背压、串行推理、模型离线校验和 Base 逐步解码。缺 Python 依赖或模型时降级保持存活。
- `packages/voice/src/tts/`：本地 TtsProvider、OpenAI 兼容云端适配器。云端仅模拟验证，没有真实云端请求或密钥。
- `packages/contracts/src/tts.ts`：用户批准的最小补齐 CONTRACTS §5.4 既有类型与 zod 校验，沿用 Lang；Python schema 从共享契约生成。批准记录见 `tasks/requests/P4-02-tts-contract.md`。
- desktop 仅 voice-sidecar supervisor 配置条目及其测试；未改 P4-01 utilityProcess/通道。模型锁/schema 仅追加 tts 分区，其他分区随 main 保留。
- `qwen_import.py`、`audit_qwen_import.py`：锁定 qwen-tts 0.1.1，禁用未使用的 25 Hz SoX 能力；真实导入审计阻止并记录任何 SoX 启动尝试。
- `design_test_voice.py`、`benchmark.py`、`gpu_common.py`、`launch-benchmark.mjs`、`summarize_acceptance.py`：生成参考、精确变体测量及无 CUDA 的原始数据复算。README 保留复现步骤。
- `services/voice-sidecar/tests/fixtures/voice-design-v1/`：三语纯合成参考 WAV（LFS）及 voice.json/manifest，登记 VoiceDesign 固定修订、Apache-2.0、cloneAllowed、SHA-256；没有使用任何 P4-01 录音或真实人物参考，也不是 Nemu 定稿音色。
- 本报告目录保留逐句 JSONL、WDDM 原始采样、失败运行、控制台日志、参考元数据和检查证据。`gpu-evidence-sha256.json` 是原始文件清单；`git-byte-preservation.json` 核对 24 个原始文件 Git blob 与磁盘字节摘要一致。原始 CRLF 数据通过局部 attributes 保持原字节。

## 真实 GPU 验收

用户停止 runner 后确认 Runner.Listener、Runner.Worker、Blender 和其他测量均未运行，窗口中没有交叉运行编译或测试，见 `quiet-window-20261021.json`。RTX 3080；使用 P0-07 probe 环境，载体 Qwen3-4B-Instruct-2507 固定修订、NF4/bfloat16、eager、原生 RMS、layer 17/rank 8 m=0 hook、48 步 decode 的 KV 同时驻留。TTS snapshot 共 33 个文件和载体 10 个文件均已校验。

复用 probe 的实际 NVIDIA/D3D11 渲染夹具及 sample.vrm，并采集所属子进程；该夹具不等于正式 Nemu 角色的最终整机视觉验收。实际包版本/路径见 `gpu-runtime-environment.json`：推理栈与 probe 一致；其 FastAPI 0.143.2、uvicorn 0.54.0 与服务默认锁的 0.115.12、0.34.2 不同，不能把本次数据描述成默认锁环境的完整复验。

| 项目 | Base 0.6B | Base 1.7B | 判定 |
| --- | --- | --- | --- |
| 中文首包 p50，20 句 | 854.682 ms | 854.117 ms | 均失败，要求 ≤250 ms |
| 英文首包 p50，20 句 | 851.609 ms | 863.247 ms | 均失败 |
| 日文首包 p50，20 句 | 853.555 ms | 862.669 ms | 均失败 |
| 专用显存观测峰值 | 6,899,363,840 B | 8,925,876,224 B | 全部所属进程 |
| 共享显存观测峰值 | 288,731,136 B | 288,436,224 B | 全部所属进程 |
| ADR 0012 §2.2 两项峰值之和 | 7,188,094,976 B（7.188 GB） | 9,214,312,448 B（9.214 GB） | 0.6B 通过；1.7B 失败，要求 ≤8,000,000,000 B |
| torch allocator allocated 峰值 | 5,508,812,800 B | 7,543,661,568 B | 补充指标，不替代 WDDM 合计 |
| torch allocator reserved 峰值 | 6,341,787,648 B | 8,283,750,400 B | 同上 |
| 连续压力时长/合成次数 | 1800.125 s / 289 | 1804.297 s / 298 | 均完成，无运行异常 |
| 压力首末 RSS 差 | +12.813 MiB | +15.012 MiB | 5 分钟中位数仍上升，稳定性不接受 |
| 取消后最后分块，15 次 | 最大 0.7621 ms | 取消后未观测到新分块（记 0） | 均满足 ≤50 ms 的停止出块要求 |

1.7B 确实驻留并完成自身测量，精确变体模式禁用了自动回退；没有将 0.6B 数据当作 1.7B。取消值是客户端发送取消后收到末块的时间，0 不代表推理核/服务确认耗时为零。

压力中 torch 当前 allocated 保持不变；0.6B reserved 在前段扩张，1.7B reserved 保持不变。RSS 六个 5 分钟分箱的中位数仍递增，完整数值见 `acceptance-summary.json`。RSS 属于同进程的载体、TTS、HTTP 客户端/服务，未定位归因，不能直接断言某模型泄漏，也不能自行设允许增长百分比判通过。

WDDM 两次分别有 5560/5710 个观测，无采样错误，最大开始间隙 1.431/1.336 秒；渲染/采样子进程均退出 0。观测峰值受 CIM/驱动刷新与瞬态漏采限制，不是连续精确峰值。ADR 上界使用专用最高值加共享最高值，未以 torch allocator、显卡全机数字或 total committed 替代。

证据：`gpu-0.6B-v2/`、`gpu-1.7B-v1/` 及对应 console 日志。最初 `gpu-0.6B-v1/` 因 warmup 响应 JSON 解析失败而终止，完整失败保留，不算 TTS 性能通过；随后基准客户端禁用 localhost 环境代理、增加 HTTP 状态检查，未改变系统代理或验收阈值。复算：`python services/voice-sidecar/summarize_acceptance.py --reports reports/P4-02`。

## 替代方案比较

| 方案 | 实测与限制 | 当前可用结论 |
| --- | --- | --- |
| 本地 0.6B | 三语 p50 851.609–854.682 ms；合计 7.188 GB | 显存符合，延迟不符合；RSS 稳定性待定位 |
| 本地 1.7B | 三语 p50 854.117–863.247 ms；合计 9.214 GB | 延迟和显存均不符合 |
| 云端适配器 | 接口/格式/风格映射/取消模拟测试通过；未测真实云端 | 不提供虚构的网络延迟或显存节省数字；真实云端本次仍不测 |

## 合并 main 后的代码检查

| 命令/检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm install --frozen-lockfile` | 通过 | `post-main-install.log` |
| `pnpm lint`、`pnpm typecheck` | 通过 | `post-main-lint.log`、`post-main-typecheck.log` |
| `pnpm test` | 454 通过 | `post-main-test.log` |
| `pnpm tts:schema` | 通过 | `post-main-schema.log` |
| `uv sync --project services/voice-sidecar --locked` | 通过，默认环境无 inference extra | `post-main-uv-sync.log` |
| ruff、mypy --strict | 通过，20 个 Python 文件 | `post-main-ruff.log`、`post-main-mypy.log` |
| pytest | 26 通过，1 个 opt-in GPU smoke 明确跳过；完整 GPU 测量另见上表 | `post-main-pytest.log` |
| `uv build --project services/voice-sidecar` | sdist/wheel 通过 | `post-main-wheel.log` |
| `pnpm test:e2e`（含 workspace build） | 3 通过，0 重试；原样 7 进程/0 重启断言通过 | `post-main-e2e.log`、`post-main-e2e-captures/` |

原样 shell.spec.ts 保留 7 个受守护进程、重启 0 次的断言；服务缺依赖/模型的降级路径另有真实子进程测试。CPU doubles/TestClient 验证三语/style、坏输入、参考篡改、取消、背压和 OOM 回退，不能替代 GPU 性能验收。e2e 生成的 P0-01 文件已复制到本卡证据目录，再按 HEAD 原始字节恢复，未更新视觉基准。之前提交 d7bc45d 的 Ubuntu/Windows CI 成功见 `CI-ubuntu.json`、`CI-windows.json`；它们属于历史提交，不代表本轮新 HEAD 已通过 CI。

## 实现边界

Base 的 style 只索引参考，不接收文字指令。参考路径不得越出音色目录，必须校验来源、cloneAllowed 与 SHA-256。合成用请求范围 talker forward hook，每 4 个 codec frame 解码，保留 25 个左上下文；上游末尾整句解码仍执行。实际接缝听感没有签发通过。

1.7B 生产路径可因低可用显存/OOM 降至 0.6B，并在 health 报告；已发送部分音频时不重播句子。取消先停止发送，原推理线程退出前保留 lane。云端 API key 仅由调用者临时提供，不持久化或输出远端错误正文。

## 新增依赖与许可决定

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| FastAPI | 0.115.12 | MIT | 默认 HTTP/WS 服务 |
| uvicorn | 0.34.2 | BSD-3-Clause | ASGI |
| websockets | 15.0.1 | BSD-3-Clause | WS 客户端/服务 |
| structlog | 25.3.0 | MIT/Apache-2.0 | 结构化日志 |
| jsonschema | 4.23.0 | MIT | 校验生成的共享边界 schema |
| qwen-tts | 0.1.1 | Apache-2.0 | 可选推理链 |
| torch / torchaudio | 2.10.0+cu128 | BSD | 可选 CUDA 推理 |
| transformers | 4.57.3 | Apache-2.0 | 可选模型加载 |
| soxr | 1.1.0 | LGPL-2.1-or-later | 用户 2026-10-21 有条件批准，动态加载、不静态链接、不改源码；P7 打包前复核 LGPL 分发义务：许可证文本、源码出处、可替换 |
| soundfile 随包 libsndfile | soundfile 0.14.0 | LGPL-2.1 | 用户同日有条件批准，CFFI 加载独立 DLL，不静态链接、不改源码；P7 打包前复核 LGPL 分发义务：许可证文本、源码出处、可替换 |
| sox Python 包装 | 1.5.0 | BSD-3-Clause | wheel 不含 exe/dll/pyd，不打包或调用 SoX 程序；未使用的能力在服务自有导入边界被禁止 |

开发检查依赖 httpx 0.28.1（BSD-3-Clause）、types-jsonschema 4.26.0.20261006（Apache-2.0），其余精确锁见 uv.lock。bitsandbytes 0.50.2 来自既有 probe，本卡未修改载体分区。没有新增 GPL/AGPL 应用代码。

原 SoX 阻塞已解决：pysox 导入会调用 `sox -h`，Qwen 无条件导入未使用的 25 Hz 路径会触发它。`qwen_import.py` 在真实 pysox 导入前提供明确拒绝 Transformer/Combiner 的能力模块，仅支持锁定的 12 Hz 模型；版本不符或已有不受控 SoX 模块时拒绝启动推理。没有修改第三方源码或全局 subprocess。真实 Qwen 导入审计的 `sox_process_attempts=[]`，禁用能力调用也被拒绝，见 `sox-import-audit-v1.json`。随后 VoiceDesign 和两种 Base 的真实运行均走此边界。源码/许可证静态证据见 `license-followup-20261021.json`、`probe-audio-licenses.json`、`licenses/`。独立 pyd/DLL 边界不等于已完成上游二进制内部链接和分发复核，P7 仍须核查。

## 待用户确认与后续建议

- [ ] （阻塞）三语 250 ms 首包失败、1.7B 显存超线；保留当前数据，不放宽阈值。
- [ ] （阻塞）定位持续 RSS 上升，再安排新安静窗口验证稳定性；本次不以完整跑满 30 分钟代替稳定性通过。
- [x] LGPL 条件批准、SoX 调用阻塞解除、纯合成参考生成与登记。
- [ ] 真实分块音频接缝/听感尚未签发；声音定稿属于 P4-03。

后续可在本卡范围定位首包耗时与 RSS 来源，再决定优化与复测；真实云端比较须另获授权。P4-06 的投递记账和正式角色声音不在本卡范围内。

## 下一步

等待解决上述验收阻塞并复测。PR 保持草稿，不合并，不取下一张卡；全部标准满足后才转可审查，由 Claude 审查。

本轮 GPU 和本机重负载检查已结束，已通知用户可以恢复 Windows runner。新 HEAD 的 CI 状态单独记录，不以旧提交检查替代。

## 本轮远端检查归档

代码/报告提交 `4a4eb935b7e12e5bf27f982414c6e75baf83ce91` 的 [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/39583470293) 已成功，包含 TS/构建、契约、transport、四个 Python 项目、Rust、.NET 和 IDL/三语言一致性检查，见 `CI-ubuntu-gpu-followup.json`。同提交 [Windows 作业](https://github.com/nanake-lab/nemu/actions/runs/39583470934) 因 PR 为草稿按规则跳过，见 `CI-windows-draft-gpu-followup.json`；本轮本机 e2e 已通过，不声称当前提交的 Windows CI 已运行。此后仅追加该检查归档与报告澄清，未改变受测实现。
