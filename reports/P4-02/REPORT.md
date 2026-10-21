# P4-02 TTS 旁路服务（三种语言）— 报告

## 结果

部分完成：代码与 CPU/模拟验证已交付，真实 GPU 验收由用户明确暂缓，见 [BLOCKED.md](BLOCKED.md)。不声称 250 ms、显存、30 分钟压力或真实 GPU 取消验收通过。没有合并，也没有取下一张卡。

从 `origin/main` 创建独立工作树和 `task/P4-02-tts-sidecar`，草稿认领 PR [#45](https://github.com/nanake-lab/nemu/pull/45)。开工与收尾均执行 fetch；以远端规则、卡片和锁定载体为准。用户另外批准补齐 CONTRACTS §5.4 既有接口，记录在 `tasks/requests/P4-02-tts-contract.md`。

## 交付物

- `services/voice-sidecar/src/main.py`：supervisor 入口；缺 FastAPI 等依赖时使用标准库的降级健康服务，保持存活。
- `services/voice-sidecar/src/nemu_voice_sidecar/`：FastAPI health/warmup、WebSocket PCM/取消、有界背压、串行推理 lane、Base 逐步解码、离线模型校验、参考来源和哈希校验。
- `packages/voice/src/tts/`：本地 TtsProvider 和 OpenAI 兼容云端适配器；云端只有模拟测试，无真实请求或密钥。
- `packages/contracts/src/tts.ts`：实现既有 TtsProvider/请求类型及 zod 校验；沿用 Lang。Python schema 由它生成，并有漂移检查。
- `apps/desktop/src/supervisor/process-config.ts`：只修改 voice-sidecar 一条健康检查，另加对应测试；未改 P4-01 的 utilityProcess、通道或主进程文件。
- `config/models.lock.json`/schema：只追加顶层 `tts`。登记 Base 0.6B、Base 1.7B、测试用 VoiceDesign 的来源、固定修订、11 个运行文件各自的 SHA-256、大小、Apache-2.0 与可分发标记。其他分区保持原样。
- `prepare_models.py`、`design_test_voice.py`、`benchmark.py`、`gpu_common.py` 和 README：显式下载、纯合成参考与待执行的验收工具。模型与未来测试声音放在本工作树的被忽略缓存中，不提交权重或音频。
- 本目录：成功/失败原始测试日志、模型元数据、载体文件校验、e2e 输出和截图。e2e 生成的 P0-01 路径已先复制进本目录，再按 HEAD 原始字节恢复，没有更新视觉基准。

缓存准备状态：Base 0.6B 的 11 个文件已从既有 probe 缓存复制并重新校验；Base 1.7B 的 11 个文件已下载并校验；VoiceDesign 下载仍在进行，只有完整文件通过 SHA-256 后才可供后续使用。选定载体缓存的 10 个文件也已逐文件与固定修订的远端摘要核对，见 `carrier-file-lock.json`，此过程没有导入模型或启动 GPU。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 | `lint-final.log` |
| `pnpm typecheck` | 通过 | `typecheck-final.log` |
| `pnpm test` | 454 项通过 | `vitest-final.log` |
| `pnpm tts:schema` | 通过；共享契约与 Python schema 一致 | `schema-drift.log` |
| `uv sync --project services/voice-sidecar --locked` | 默认环境通过；不安装 inference extra | `uv-sync-default.log` |
| `uv run --project services/voice-sidecar ruff check services/voice-sidecar` | 通过 | `ruff-final.log` |
| `uv run --project services/voice-sidecar mypy --strict services/voice-sidecar` | 全部 16 个 Python 源/测试/工具文件通过 | `mypy-final.log` |
| `uv run --project services/voice-sidecar pytest services/voice-sidecar/tests -ra` | 24 通过、1 明确跳过 | `pytest-final.log` |
| `uv build --project services/voice-sidecar` | sdist/wheel 通过 | `python-build-final.log` |
| `pnpm test:e2e`（含 workspace build） | 3 项通过，0 重试 | `e2e-final.log`、`e2e-final-captures/e2e-results.json` |
| 原样 shell.spec.ts 的 7 进程/重启 0 次 | 通过；voice-sidecar 使用真实入口、降级运行，其余未实现进程可仍为原有占位进程 | 同上；未改断言 |
| 无 site-packages 的 `python -S src/main.py` | 真实子进程 HTTP health 200、保持存活 | `test_stdlib_fallback_without_site_packages` |
| 三语、语气映射、坏输入、参考篡改、取消、OOM | CPU doubles/TestClient 通过；不是硬件性能结论 | `test_service.py`、`test_qwen_adapter.py` |
| GPU smoke | 明确跳过：未请求 GPU、CI 不下载模型或分配 CUDA | `pytest-final.log` |
| 真实三语首包/显存/30 分钟压力/取消 | **未执行：用户暂缓安静窗口** | `BLOCKED.md` |
| GitHub CI/Windows | 交付时以 PR 检查与本目录追加 CI 状态为准 | PR #45 |

测试中保留了初始失败日志：新导出 schema 未登记共享测试夹具、客户端错误被正常 close 覆盖、类型/格式检查错误。已修正并重跑；没有降低阈值、删断言或修改 e2e 测试。

## 实现与证据边界

Base 的 `style` 只选 `voice.json` 的语言/语气参考，不送文字指令。参考必须登记来源、许可、cloneAllowed 和 SHA-256，且不能越出声音目录；请求不能直接指定任意音频路径。没有读取 P4-01 用户音频，没有克隆任何人声，测试 VoiceDesign 也尚未执行；Python 的字节占位 fixture 只用于传输测试，未交给任何真实模型。

qwen-tts 0.1.1 的 `non_streaming_mode=False` 本身仍整句返回。适配器使用请求范围内的 talker forward hook，每 4 个 codec frame 解码、保留 25 个左上下文；上游的末尾整句解码仍会执行。CPU double 验证首个解码发生在生成结束前、取消移除 hook、输出长度及有界左上下文。真实音质、接缝、GPU 延迟和压力稳定性都待测，不能仅凭有二进制分块判为性能通过。

1.7B 的低显存准入与 OOM 都可降至 0.6B；已经发出部分音频时返回失败，不重播句子。health 的 HTTP 200 代表存活，`ready` 代表后端可用；显存降级原因不隐藏。取消后先停止网络发送，合作终止生成，原推理线程退出前 lane 不释放。

云端通过调用者提供的 `instructions[lang][style]` 表映射语气，支持 24 kHz int16 PCM 转 float32、跨帧半采样和取消。不保存密钥、不记录远端错误正文；没有把网络模拟延迟当作真实云端结果。

## GPU 预注册计划与待测比较

两个变体依次在独立进程中测量；使用 P0-07 probe 环境、选定的 Instruct-2507 固定修订、NF4/bfloat16、eager、原生 RMS 和 layer 17/rank 8 的 m=0 hook。固定合成三语输入及 48 步 decode 的 KV 同时保留。载体缓存按远端固定修订逐文件校验，TTS 本地模型按本卡锁校验。

每个变体先三语预热，再每种语言 20 句首包（含 WS 连接和请求），每种语言 5 次取消，以及固定至少 1800 秒连续压力。全程复用 P0-07 的 GPU 渲染夹具并采集全部子进程。该夹具不替代正式 Nemu 资产或最终整机视觉验收。

按 ADR 0012 §2.2，上界为 Nemu 所有进程专用显存的最高观测值与共享显存的最高观测值之和，判定 ≤ 8,000,000,000 字节，同时报告 torch allocated/reserved 峰值。原始采样缺失不得算作 0 显存通过；保留 WDDM 刷新/采样间隙限制。RSS 与显存趋势从逐句原始记录判断，不自行发明允许增长阈值。

| 方案 | 实际首包 p50 | Nemu 合计显存 | 当前可确认的范围 |
| --- | --- | --- | --- |
| Base 0.6B | 未测 | 未测 | 模型已锁；本地实现/CPU 模拟通过 |
| Base 1.7B | 未测 | 未测 | 模型已锁；低显存/加载及生成 OOM 回退有 CPU 测试 |
| 云端回退 | 未测 | 未测 | 兼容接口与模拟测试；真实延迟依赖网络和供应商，未调用 |

任何方案未达到 250 ms 或超显存时保留真实失败并比较，不放宽阈值。测试语句和声音设计描述已固化在脚本中。

## 新增依赖

默认 runtime 的精确版本：FastAPI 0.115.12（MIT）、uvicorn 0.34.2（BSD-3-Clause）、websockets 15.0.1（BSD-3-Clause）、structlog 25.3.0（MIT/Apache-2.0）、jsonschema 4.23.0（MIT）。开发检查新增 httpx 0.28.1 和 types-jsonschema 4.26.0.20261006；uv.lock 固定完整依赖与哈希，Python 3.12。

可选 `inference` 与既有 probe 对齐：qwen-tts 0.1.1（Apache-2.0）、torch/torchaudio 2.10.0+cu128（BSD）、transformers 4.57.3（Apache-2.0）。CI 默认不启用该 extra，不下载模型。载体的 bitsandbytes 来自既有 probe 环境，本卡未改变载体分区或其版本。

用户 2026-10-21 已批准下列可选 Qwen 推理依赖，批准条件是动态加载、不静态链接、不修改第三方源码：

| 依赖 | 版本 | 许可 | 用户决定与条件 | 后续核验 |
| --- | --- | --- | --- | --- |
| soxr | 1.1.0 | LGPL-2.1-or-later | 已有条件批准；Python 通过独立 soxr_ext.pyd 加载，本卡未静态链接或修改源码 | P7 打包前复核 LGPL 分发义务：许可证文本、源码出处、可替换 |
| soundfile 随包 libsndfile | soundfile 0.14.0 | LGPL-2.1 | 已有条件批准；soundfile 通过 CFFI dlopen 加载独立 libsndfile DLL，本卡未静态链接或修改源码 | P7 打包前复核 LGPL 分发义务：许可证文本、源码出处、可替换 |
| sox（Python 包装） | 1.5.0 | BSD-3-Clause | 已确认是命令行 SoX 的包装，wheel RECORD 不含 exe/dll/pyd；本服务没有直接 sox 调用或打包配置 | **尚不满足“不调用 sox 程序”：上游导入副作用会尝试 sox -h，见下文** |

独立 .pyd 的加载只确认 Nemu/Python 一侧的动态模块边界；没有据此声称上游 wheel 内部所有组件均采用动态链接。P7 还需核对所交付二进制的链接方式与可替换性。许可证据见 `probe-audio-licenses.json`、`licenses/` 和 `license-followup-20261021.json`。

sox 核查发现：锁定的 `sox/__init__.py:8` 在导入时执行 `os.popen('sox -h')`。Qwen 的 core 包无条件导入 25 Hz tokenizer，后者导入 speech_vq，speech_vq 顶层导入 sox；因此不能以本卡只使用 12 Hz 模型为由宣称不会尝试启动外部 SoX。本轮只做静态读取，没有导入 qwen/sox，也没有执行 SoX。没有打包命令行 SoX，但“不调用”的条件尚未满足，保持阻塞，不写成确认通过。

## 待用户确认

- [ ] （阻塞）以后安排 GPU 安静窗口，完成上列硬件验收；2026-10-21 用户选择本轮先交付代码与待测报告。
- [x] 用户 2026-10-21 有条件批准 soxr / 随包 libsndfile：动态加载、不静态链接、不改第三方源码；P7 复核分发义务。
- [ ] （阻塞）解决并核验 Qwen 导入链尝试 `sox -h` 的副作用；当前不能确认“不调用 sox 程序”。
- [ ] （不阻塞代码审查）真实分块音频接缝与听感待测；Nemu 声音定稿属于 P4-03。

## 后续建议（不在本任务范围内）

- P4-03 提供正式 voice.json 与跨语言/语气参考；当前没有定稿任何声音。
- P4-06 接入输出投递记账与舞台播放；本卡客户端异常结束会报错，不替终端宣称音频已投递。

## 下一步

转为可审查后等待 Claude 审查实现；硬件验收与非宽松许可仍待用户。**不合并，不取下一张卡。**

## 核对来源

- 本仓库 `tasks/P4-voice.md` P4-02、`docs/CONTRACTS.md` §5.4、`docs/adr/0012-continuity-carrier-mobile.md` §2.2、`config/models.lock.json` 的选定载体。
- [Qwen3-TTS 官方代码与说明](https://github.com/QwenLM/Qwen3-TTS)，以及 probe 环境 qwen-tts 0.1.1 安装源码。
- [FastAPI WebSocket 文档](https://fastapi.tiangolo.com/advanced/websockets/)。Context7 配额不足后使用官方文档和已安装源码核对。
- [OpenAI PCM 格式说明](https://developers.openai.com/api/docs/guides/text-to-speech)，仅用于兼容接口格式；未进行真实调用。

## 最终交付补记（2026-10-21）

- PR #45 已转为可审查，带 `needs-user`；等待 Claude 审查实现。没有合并，没有取下一张卡。
- 代码提交 `d7bc45d590fcaa499893c83dff0dd7f77ed433f3` 的 [Ubuntu CI](https://github.com/nanake-lab/nemu/actions/runs/39561117049) 和 [Windows CI](https://github.com/nanake-lab/nemu/actions/runs/39562278216) 均为 SUCCESS，完整步骤状态见 `CI-ubuntu.json`、`CI-windows.json`。Windows 包含 transport、isolation、e2e、visual 及证据溯源检查。
- VoiceDesign 下载已经完成，三个 TTS snapshot 共 33 个文件再次离线哈希复核通过，见 `cache-verification.json`。仍未加载真实模型进行 GPU 推理，也未生成声音。前文下载状态是首次推送时的记录。
- 此补记和缓存/CI 归档只追加报告证据，运行代码与上述受测提交一致；报告提交按仓库文档/报告 CI 排除规则处理。


## 继续阶段更新：许可决定与草稿（2026-10-21）

- 按用户指示，PR #45 已转回草稿；全部验收完成前不转为可审查。
- 上表替代此前“LGPL 许可待批准”的当前状态；批准已有明确条件，SoX 的禁止调用要求尚未满足。
- 本轮只记录决定、静态核查与更新 PR 状态；没有停 runner、没有运行测试、没有生成声音或启动 GPU。
- 已 fetch，origin/main 更新到 9042c59；必读规则与 P4-02 卡对比无变化。按用户第 3、4 条限定，本轮不合并 main 或改实现，安静窗口及验收完成后再按要求 merge 和重跑检查。
- 等待用户通知 GPU 安静窗口开始；真实云端延迟仍不测。开始推理前还须先解决“不调用 SoX”的已知问题，不能直接执行当前导入链。
