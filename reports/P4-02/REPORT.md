# P4-02 TTS 旁路服务（三种语言）— 报告

## 结果

当前优化候选及开发短测已交付，正式重测尚未执行；候选结果与剩余显存边界见文末。PR 仍为草稿及 needs-user。

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

## 初轮真实 GPU 验收（优化前，保留原结论）

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

## 2026-10-21 用户批准的后续压力重测判据

此规则仅用于后续重测，不追溯重判已归档的原始结果：torch allocated 首末不变，且第 20–30 分钟 RSS 对时间的普通最小二乘线性斜率 ≤12 MiB/小时。任一条件不满足，延长同一轮连续压力到总计 2 小时，报告第 110–120 分钟斜率、allocator 和趋势，由用户判断是否趋平；延长不自动等于通过。`pressure.py` 提供复算，正式 benchmark 固定执行，不增加延迟或实时率门槛。正式安静窗口需为可能的延长预留时间。

## 本地首包/生成优化候选（2026-10-21 后续指令）

代码与开发验证完成，正式验收尚未重跑。全部改动仅在 services/voice-sidecar/ 与 reports/P4-02/；PR #45 继续草稿及 needs-user。原始 20 句/30 分钟结果和失败全部保留。未测真实云端。

### 方案与兼容性

未新增运行时依赖。评估 [faster-qwen3-tts](https://github.com/andimarafioti/faster-qwen3-tts/tree/b35fdc7d71eb3e4d19e63f4b4a330a1aef31d682) 0.5.4（MIT，commit b35fdc7d71eb3e4d19e63f4b4a330a1aef31d682）后发现其要求 Transformers >=5.15.1 与 qwen-tts-hf，不能直接用于现有 qwen-tts 0.1.1/Transformers 4.57.3 锁。源码仅在忽略缓存中研究，没有安装、复制或提交其应用代码；许可及 pyproject 摘要见 faster-qwen-evaluation.json。Context7 配额不足，按 [PyTorch 2.10 CUDA graph/stream 文档](https://docs.pytorch.org/docs/2.10/notes/cuda.html) 和实际安装源码核对。

自有 cuda_graph.py 保留上游采样、精度、词表头、条件输入和 512 帧生成上限，版本门禁固定 torch 2.10.0+cu128 / transformers 4.57.3 / qwen-tts 0.1.1。predictor 15 个单步 graph 使用固定 KV 存储；talker 以精确注意力长度/连续布局捕获单步。资源预算：256-token 静态 KV 缓冲、每请求前 16 个 talker decode 帧、最多 48 个位置图；超范围/预算走上游 eager，范围末端按原值复制 KV 后继续生成，不截断音频，不更换模型。所有图共用捕获 stream/内存池，跨图可见输出保持引用，重放串行；tokenizer 使用独立池/持久 stream。

固定填充 softmax 的第一版出现 codec 差异，已拒绝；仅 predictor 加速的隔离试验一致，talker 隔离试验不一致，随后修正实际 KV 长度与连续布局。早期逐图 stream/大缓冲版本还出现 0.6B 8.550 GB、1.7B 10.333 GB 和满卡停滞，均保留原始证据，没有冒充优化成功。最终缩小缓冲、共用捕获资源；独立解码复用同一 CUDA stream，避免每请求累积 stream workspace。

默认继续原 eager/4 帧路径。候选正式重测时显式启用 NEMU_TTS_CUDA_GRAPH=1、NEMU_TTS_ASYNC_DECODE=1、NEMU_TTS_FIRST_FRAMES=2；完整命令见服务 README。

### 分段计时与解码

timing.py/profiling.py 记录 prefill、上游 prefill 与冷 graph_prepare、每帧 talker backbone、每个 predictor 步骤、每次 tokenizer 解码的 host/CUDA 时间；事件只在请求结束收集，不逐步 synchronize。frame 完成时间及引擎投递首末块另有记录。app.py 的 wire/ 证据记录 ws.send_bytes 完成时刻；正式 benchmark 另记客户端首末包。

实时率 = 音频时长 / 首块至末块间隔，只作报告指标。不能把引擎投递时刻等同客户端收到时刻，也不能把含冷捕获的首次请求称为预热结果。上游结束时的整句解码仍执行，tokenizer_decode 的最后一项可看到该成本；未以删除其计算来隐藏耗时/显存。

解码改为有界队列、独立线程与持久 CUDA stream；producer event 保证 codec 就绪。首块 2 帧，其余 4 帧，左上下文仍 25 帧。取消后不再投递，生成与解码线程都退出才释放 lane。

### 最终开发短测（不是正式 p50/压力验收）

载体与同一真实 GPU 渲染夹具驻留；两个变体各新进程串行。每模式三条固定贪心输入，双方相同 64-token 限制，仅用于开发一致性；正式 benchmark 强制 generation_limit==512。基线为 optimization-0.6B-v3/、optimization-1.7B-v1/ 的原始 eager4/eager2，不改原始数据。最终结果为 optimization-0.6B-v9/、optimization-1.7B-v4/。

| 指标 | 0.6B | 1.7B |
| --- | --- | --- |
| 热图/异步首块，中/英/日 ms | 233.205 / 196.403 / 223.672 | 269.548 / 249.451 / 210.670 |
| 首块到末块实时率，中/英/日 | 1.280 / 1.222 / 1.490 | 1.497 / 1.110 / 1.158 |
| 同条件完整 codec 序列 | 三语全部逐元素相同 | 三语全部逐元素相同 |
| 同 2 帧分块的上游 vs 候选 PCM | 三语逐样本相同，最大绝对差 0 | 三语逐样本相同，最大绝对差 0 |
| 候选同步 vs 异步 PCM | 三语逐样本相同，最大绝对差 0 | 三语逐样本相同，最大绝对差 0 |
| 专用峰值 + 共享峰值 | 7,539,003,392 + 277,065,728 = 7,816,069,120 B（7.816 GB） | 9,382,764,544 + 270,987,264 = 9,653,751,808 B（9.654 GB，超线） |
| 三条异步样本的 torch allocated | 相同，5,121,306,624 B | 相同，7,151,142,912 B |

不能用三条开发样本冒充各语言 20 句 p50，也不能用这次 allocated 稳定签发 30 分钟压力通过。1.7B 仍有真实显存阻塞。两个最终短测的原始 WDDM/torch 数据、timing JSON、音频和 codec 哈希均归档。

另外用最终资源配置做了同一 512-token 上限的完整对照：optimization-full-0.6B-v2/，511 帧 codec 与上游逐元素相同，SHA-256 同为 db8760c04daf7bd0a81ee5ad0d6dec8054c40667bcf977bbcb98e5571a1cc032。40.88 秒音频的首块到末块 37.248 秒，实时率 1.098；该请求含冷捕获，不用于预热首包判定。其 Nemu 合计峰值为 8,089,268,224 B（8.089 GB），仍超过 8 GB，不能隐藏这一长输出边界。正式重测前需保留此风险/阻塞；未放宽阈值。

接缝：按真实 codec 片段边界计算跳变，见 optimization-summary.json。0.6B 日文 2 帧首块方案边界最大跳变 0.037994，原 4 帧为 0.011566；两者边界位置不同，不据此单独判定音质通过或失败。其余数值及原始 WAV 均保留。异步与同分块同步 PCM 完全相同，未引入额外波形差异；听感没有签发。optimization-audio-manifest.json 登记所有开发音频的来源、许可、cloneAllowed、参考 SHA 和产物 SHA；没有使用真实人物或 P4-01 音频。

summarize_optimization.py 无 CUDA 复算分段计时/接缝/PCM 差异；optimization-evidence-sha256.json 为逐文件原始摘要。原始 CRLF 字节保留，WAV 用 LFS。

### 检查与下一步

ruff、mypy --strict（30 文件）通过，pytest 35 通过/1 个 opt-in GPU smoke 明确跳过；见 optimization-*-final-v2.log。初始格式/类型失败日志也保留。已 merge main 的 ADR 0015 文档更新；合并后检查另行归档。PR 不转可审查、不合并、不取下一张卡。

正式窗口另约：载体/渲染照常驻留，两变体各语言 20 句、真实链路取消、ADR 0012 §2.2 显存和压力均按原阈值。通常约 90 分钟；如两轮压力均触发用户批准的 2 小时延长，需约 4–4.5 小时。压力判据仅用于新测量，不追溯原结果。

原始文件 Git/LFS 字节核对见 optimization-git-byte-audit.json，所有记录摘要均一致。

### 优化合并 main 后检查

合并提交 d585926（origin/main 806e09b，ADR 0015 仅文档）后，pnpm lint/typecheck、454 项 TS 测试、共享 schema、ruff、mypy --strict 30 文件、pytest 35 通过/1 个 opt-in GPU smoke 明确跳过、sdist/wheel、Electron e2e 3 项均通过。原样 7 受守护进程/重启 0 次断言不变；optimization-post-main-*.log 和 optimization-post-main-e2e-captures/ 为证据。测试写到 P0-01 的文件已复制后按 HEAD 原字节恢复，没有更新视觉基准。

GPU 与本机重负载检查均已结束，已通知可恢复 runner。没有运行新正式压力测试或真实云端请求；PR 不转可审查、不合并。下一轮窗口由用户另行通知。

### 分段定位的配对结果

optimization-stage-comparison.json 从相同贪心 codec 的逐步 CUDA event 区间复算。以英文 63 帧样本为例，上游 eager2 对比最终 graph2-async：0.6B predictor 的 15 步合计均值 96.270 →14.722 ms/帧，talker backbone 36.302 →34.571 ms/帧；1.7B predictor 97.910 →16.743 ms/帧，talker 37.730 →37.078 ms/帧。每条均有 945 个 predictor forward。prefill 约 39–46 ms，说明主要收益来自 predictor 调度区间，而不是减少权重大小。

这些 CUDA event 区间包含 host 逐步供给导致的 GPU 空闲间隙，不等于纯 kernel 活跃时间，也不构成逐 kernel 的 Nsight 归因。talker 只加速有界前段，后段保持 eager，故整句平均降幅较小。异步 decode 与生成竞争同卡资源，0.6B 此样本每片段解码中位区间 37.520 →82.643 ms，不能把“移出生成线程”说成解码本身变快；实际首末块实时率仍提高。最后的上游整句解码成本单列保留。

Trace 的引擎时钟从参考 prompt 准备完成后开始，开发表是热 prompt/热图的引擎投递边界；实际请求/WS 首包必须由另约的正式 benchmark 计量，不能直接以本表签发 ≤250 ms 验收。

### 优化远端检查归档

受测提交 1beae189796ba8439cf07af954c898179021fb27 的 [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/39620971993) 已成功，完整状态见 CI-ubuntu-optimization.json，包含 TS/Python/Rust/.NET/契约检查。同提交 [Windows CI](https://github.com/nanake-lab/nemu/actions/runs/39620971933) 因草稿规则跳过，未冒充通过；本机合并后的原样 e2e 已通过。此后仅追加分段分析、检查归档和报告，未改变受测实现。

优化执行后，Qwen/SoX/音频库的既有静态源码摘要仍与此前一致，见 optimization-third-party-source-audit.json；未修改第三方源码或打包/调用 SoX 程序。LGPL 的 P7 分发复核义务继续保留。

## 2026-10-23 优化后正式重测（未完成，保留失败）

使用同一 P0-07 RTX 3080 carrier、真实渲染负载、固定 VoiceDesign 合成参考，启用 CUDA graph、异步 decode 和 2 帧首块。每种变体使用新 Python 进程，未回退。三语各 20 句及每语 5 次真实 WebSocket 取消都完成；压力运行未达到批准的两小时结束点，因此不得视为压力验收通过。

| 变体 | 三语首包 p50（中/英/日，ms） | 取消后末块最大值 | WDDM 专用峰值 + 共享峰值（部分采样） | torch allocated 首→末 | RSS 20–30 分钟 OLS（MiB/h） | 压力时长/请求 | 结束状态 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0.6B | 209.224 / 215.497 / 209.500（本组样本均 ≤250） | 0/15 均未见新块，最大 0 ms | 8,604,368,896 + 288,280,576 = 8,892,649,472 B（超过 8,000,000,000；watcher 后续收到 CIM「调用已取消」） | 5,129,826,304 → 5,129,826,304 B | 13,103.03（超 12） | 4,295.528 s / 2,639 | Windows 进程以 `-1073741571`（`0xC00000FD`，STATUS_STACK_OVERFLOW）退出；未完成延长 |
| 1.7B | 225.787 / 225.165 / 227.418（本组样本均 ≤250） | 0/15 均未见新块，最大 0 ms | 9,780,871,168 + 276,910,080 = 10,057,781,248 B（超过 8,000,000,000；为部分峰值，watcher 发生 CIM 资源/内存错误） | 7,159,662,592 → 7,159,662,592 B | 12,704.46（超 12） | 4,004.946 s / 2,445 | sidecar 记录 `MemoryError`，WebSocket 1011；未完成延长 |

两轮的 20–30 分钟 RSS 斜率均触发 2026-10-21 用户批准的两小时延长；没有 110–120 分钟斜率，不能判定趋平。0.6B 压力段 RSS 末值低于首值，但这不抵消其 20–30 分钟 OLS 超线；1.7B 的部分 WDDM 峰值和进程 RSS 都继续升高。torch 当前 allocated 首末相同不代表稳定性通过。torch 峰值 allocated/reserved 分别为 0.6B 5,934,155,264 / 7,277,117,440 B，1.7B 7,626,955,264 / 8,455,718,464 B；它们与 ADR 0012 的 WDDM 指标分开报告。

已完成样本的分段计时如下。CUDA 时间是逐调用事件区间；talker、predictor、异步 decode、整句生成区间会重叠，不能相加为端到端延迟。详细逐句分位数、调用数、首末引擎发出时刻和原始跨度见 `optimized-formal-partial-summary-20261023.json` 及下方两个原始压缩包中的 `traces.jsonl`。实时率仅作报告指标。

| 变体 | 语言 | client 首包 p50 ms | 引擎首/末发出 p50 ms | prefill CUDA p50 ms | talker 单调用 host/CUDA p50 ms | predictor 单调用 host/CUDA p50 ms | decode 单调用 host/CUDA p50 ms | 实时率 p50（倍） |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0.6B | 中 | 209.224 | 205.7 / 1,258.7 | 42.1 | 0.350 / 5.52 | 0.089 / 0.873 | 81.1 / 82.7 | 1.704 |
| 0.6B | 英 | 215.497 | 212.0 / 1,596.6 | 47.5 | 0.444 / 5.70 | 0.088 / 0.871 | 80.2 / 82.9 | 1.544 |
| 0.6B | 日 | 209.500 | 206.0 / 1,901.3 | 41.2 | 0.515 / 6.03 | 0.088 / 0.866 | 79.5 / 82.5 | 1.491 |
| 1.7B | 中 | 225.787 | 222.0 / 1,272.6 | 46.0 | 0.356 / 8.71 | 0.089 / 0.876 | 82.1 / 86.4 | 1.642 |
| 1.7B | 英 | 225.165 | 221.6 / 1,631.6 | 46.8 | 0.435 / 9.01 | 0.088 / 0.880 | 80.4 / 85.4 | 1.497 |
| 1.7B | 日 | 227.418 | 223.9 / 1,817.0 | 44.0 | 0.458 / 9.18 | 0.088 / 0.880 | 79.9 / 85.0 | 1.468 |

0.6B 与 1.7B 的 p50 分别较旧 eager 基线约 0.85 s 明显降低，但本轮压力、显存并未通过。0.6B 不能以延迟通过替代显存/压力门槛；1.7B 的 WDDM 部分峰值也已明显超线。按用户要求未测真实云端延迟；云端 adapter 仍只有接口与模拟数据，不能提供实测对比。

失败原始数据保留在 `gpu-optimized-0.6B-formal-retry4-bounded-raw.zip` 和 `gpu-optimized-1.7B-formal-run1-raw.zip`；逐文件字节数/SHA-256 与压缩包 SHA 见 `gpu-optimized-formal-partial-archive-manifest-20261023.json`。VoiceDesign 来源/许可证/cloneAllowed 与哈希仍由既有 fixture/manifest 登记，没有使用 P4-01 录音或真实人物声音。

### 本轮测量器修正及本机检查

观察到正式压力把逐帧和 predictor 的 CUDA-event 计时用于全部数千压力请求，且 Windows CIM 进程树查询频繁失败。已将 benchmark 调整为只在 warmup 和三语延迟样本启用逐步事件；压力每句仍保存 client 首末块、音频时长、实时率、RSS 与 torch allocator。`bounded_memory_watch.ps1` 使用同一 WDDM CIM 计数器、每 5 秒刷新活动进程树，并对短暂 CIM 失败最多重试 3 次；RSS 改为 Windows `GetProcessMemoryInfo` 原生读取，避免每句新建 PowerShell 进程。此改动降低诊断开销但尚未实测证明消除了 RSS 增长，因而需要新窗口重测。

验收后 CPU/桌面检查：`pnpm lint`、`pnpm typecheck`、`pnpm test`（454 项）、`pnpm tts:schema`、`pnpm test:contracts`（315 项，100% statements）、`pnpm test:tts`（42 项通过、1 个 GPU smoke 按默认策略跳过）、严格 mypy（34 文件）、Ruff、`pnpm test:e2e`（3 项，包括 shell 守护检查）均通过。e2e 证据在 `optimized-e2e-20261023/`。GPU smoke 的跳过原因为未设置 `NEMU_RUN_TTS_GPU_TESTS=1`，CI 不下载模型/分配 CUDA；真实 GPU 正式测量另由上表记录。

PR #45 保持 draft 和 needs-user，不转可审查、不合并。待下一次独占安静窗口重测两个变体：三语各 20 句、取消、ADR 显存、30 分钟/必要时两小时压力；旧失败数据不覆盖。云端延迟仍不测。
## 2026-10-23 CI 修复与 D 盘迁移

`25e3d74` 的 Ubuntu Python step 因 Linux typeshed 不提供 Windows 专用 ctypes 属性及 CREATE_NO_WINDOW 失败。修复提交 `dc07044` 使用 sys.platform 平台守卫与受保护的启动标志；没有忽略 mypy 错误、修改验收阈值或删除测试。Ubuntu run 39893501347 已成功（ci-dc07044.json）。

迁移前修复已提交推送，之后用 git worktree add 创建 `D:\Dev\nemu-wt\P4-02`，原任务分支在此继续。sidecar .venv 用 D 盘 Python 3.12.14 重建；uv/npm/pnpm/Hugging Face/Torch/Inductor/Triton 和本卡 Cargo target 的用户级缓存路径设为 D:\Dev\cache 下。模型在新工作树 services/voice-sidecar/.cache/models；既有推理 probe 位于 D:\Dev\nemu-p007，未运行 GPU 推理。旧环境备份在 D:\Dev\cache\p4-02 下。

模型、本卡缓存及原报告共 5,064 文件、19,072,548,377 字节逐文件 SHA-256 一致，见 migration-byte-verification-20261023.json。原始报告字节额外保存在 D:\Dev\cache\p4-02\original-reports；D:\Dev\p4-02-gpu-evidence-20261023 原始测量目录未删除。迁移不改变此前失败的测量结论。

迁移后四项目依次完成 uv sync --locked、ruff、严格 mypy、额外 Linux 平台 mypy、pytest、uv build；前三项目各 1 项通过，sidecar 42 通过/1 个 opt-in GPU smoke 跳过。migration-python-checks.log 保存完整输出。D 盘 pnpm frozen-lockfile 安装、lint/typecheck 和 454 项测试通过（migration-*.log）。本次没有运行 GPU 测量或请求其他执行者审查。

旧工作树已从 Git 注销，但残余依赖链接使 Git 删除目录失败；本卡模型和数据已迁移。共享旧 uv cache 的占用文件禁止整体移动/重命名，逐文件迁移结果另见迁移状态补记。新的工作和缓存只使用 D 盘。PR #45 保持 draft/needs-user，GPU 压力和显存阻塞继续保留；正式重测须先另约安静窗口，最多预留两变体各两小时。

迁移状态补记：旧工作树已注销且数据搬迁完毕，残余 2 个占用文件共 6,100,992 B；旧 uv cache 剩余 75 文件共 77,926,912 B（主要为已加载动态库/锁），完整缓存副本和旧缓存原件均在 D 盘保留。未终止其他进程以强制解锁，旧 C 目录因此仍存在，不声称物理文件已完全清零。逐文件移动日志为 migration-uv-move.log / migration-old-worktree-move.log。C 盘可用空间由约 16.4 GiB 升至 49.6 GiB。所有新环境和缓存使用 D 盘；sidecar sys.executable/base_prefix/import 路径均已核对为 D，原生 RSS API 返回有效值。迁移后的 Windows/Electron e2e 本次未重跑（前一轮已通过），本次完成用户要求的四项目 Python CI 检查和 TS 静态/单元验证。

## 2026-10-23 迁移后 GPU 开发短测

本轮用户授权 10–15 分钟短窗口；测前 Runner.Listener/Runner.Worker/Blender 与其他语音测量均未运行，GPU 计数器未见明显活动负载。已 merge origin/main 的 P5-02 更新（不改本卡实现）。两个变体各新进程串行，P0-07 载体/渲染驻留，使用同一纯合成 VoiceDesign 参考、graph + async + 2 帧首块、512 帧生成上限。开发 driver 单独把连续合成观察限制为 240 秒，并明确 acceptance=false；原 benchmark 的 1800/7200 秒与压力判据没有修改。完整 driver 在原始归档中。

| 指标 | 0.6B | 1.7B |
| --- | --- | --- |
| 中/英/日首包 p50（各 20 句，ms） | 201.454 / 196.527 / 197.618 | 205.334 / 208.111 / 208.742 |
| 15 次取消后末块最大 ms | 0.880 | 0.966 |
| 连续合成秒 / 请求 | 241.016 / 166 | 240.562 / 164 |
| torch allocated 首末 B | 均为 5,129,826,304 | 均为 7,159,662,592 |
| RSS 首→末 B | 3,383,541,760 → 4,380,250,112 | 3,578,900,480 → 4,512,686,080 |
| RSS 增量 MiB | 950.535 | 890.527 |
| WDDM 专用峰值+共享峰值 B | 7,710,973,952 + 288,141,312 = 7,999,115,264 | 9,531,293,696 + 289,026,048 = 9,820,319,744 |
| 首末块实时率 p50 中/英/日（仅报告） | 1.808 / 1.673 / 1.551 | 1.775 / 1.637 / 1.565 |

两轮 WDDM 采样有效且 watcher 正常退出；完整进程均 exit=0。详细 trace 均只有 63 行（3 预热+60 延迟），压力阶段未继续增加，说明计时减负生效。然而 RSS 仍显著增长，不能将旧压力问题归因于详细 CUDA-event 计时或声称已解决。整段 4 分钟的开发 OLS 斜率约 14,407/13,125 MiB/h，仅作诊断，不与 20–30 分钟正式窗口混同。1.7B 显存仍超线；0.6B 此短段仅比 8 GB 低约 0.9 MB，不能覆盖已保留的长输出/长压力超线结果。没有 20–30 分钟或两小时数据，不签发稳定性通过。

原始归档 dev-resource-short-20261023-raw.zip（约 1.93 MB）、逐文件 SHA-256 清单 dev-resource-short-manifest-20261023.json 与汇总 dev-resource-short-summary-20261023.json；解压前原始目录保存在 D:\Dev\cache\p4-02\dev-resource-short-20261023。未使用真实人物或 P4-01 录音。GPU/渲染/采样进程已结束并通知用户可恢复 runner。

下一步继续定位 CPU/原生内存的按请求保留，重点检查请求生命周期、回调引用和第三方生成状态；目前没有足够证据裁定根因。先做诊断再约 GPU 短窗口，不直接开始正式两小时长测。PR #45 保持 draft/needs-user，不请求审查、不合并。
