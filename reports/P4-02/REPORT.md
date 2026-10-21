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
