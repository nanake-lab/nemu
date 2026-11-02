验证档位：重档；本卡含 GPU 正式测量，当前仅完成恢复与资源准备，GPU 未运行。

# P4-02 GPT-SoVITS V5 恢复 — 报告

## 当前状态

**部分完成；2026-11-02 后续恢复已完成 PyTorch 缓存直接复用，GPT-SoVITS 其余环境与测量待完成。** 2026-11-02 已按用户要求移除 PR #45 的 paused 标签；保持 draft / needs-user。已 fetch 并将 main `8fed313b7ca9c31423d0ad442789ddef7e6b063b` merge 到任务分支，合并提交 `960ce7b9`，不 rebase、不强推。仅 package.json 有冲突，保留双方追加脚本。原本地 `d5b83d0c` 已包含远端 `76a718ce`。

已读最新 AGENTS（含第 5、6 节）、ARCHITECTURE、CONTRACTS、Echo Core README、ADR 0012/0027/0034、P4-02 全卡与追加、旧报告/阻塞及 PR 暂停评论。DESIGN-VRAM-01 不在本次 main 树中。本轮不做载体同驻，不再沿用旧三语/250 ms 门槛；只报告用户要求的单模型测量。

**尚未实现 provider 替换**：按 AGENTS 黄金规则 13，先取得最小硬件可行性证据。环境未就绪，旧 Qwen 仍在现有源码的服务默认入口中，不能声称已切为 GPT-SoVITS。取消、健康检查、离线降级代码未改；全部旧实现及测量/失败证据保留。[历史报告](QWEN3-TTS-HISTORY.md)、[历史阻塞](QWEN3-TTS-BLOCKED-HISTORY.md)为本轮改写前的原文副本，旧结论不得套用到新模型。

## 准备结果与失败

- 工作区：`D:\Dev\nemu-wt\P4-02`；全部新代码下载、环境、权重、音频工作副本在 `D:\Dev\cache\p4-02\`。
- 开始 D 盘可用 159,491,710,976 B（148.54 GiB）；已提前报告新增 25–35 GiB、峰值预留 50 GiB 的估算。未删除任何缓存或用户文件。
- 官方源码固定 `04a44eb977586645f33594b72af13a035a2056c6`，逐文件 SHA-256 在 [源码清单](gsv-v5-20261102/code-manifest.json)。未修改上游源码。
- 官方 GPT-SoVITS 权重固定 `8f8857563c040d542de108a2c18b0425abd6c9c5`；计划 1,819,760,311 B。[六个完成文件](gsv-v5-20261102/downloaded.json)已核对大小与哈希；v5turbo 权重只有部分文件，最终下载进程退出状态未取回，不能算下载完成，vocoder/s1v3 尚未完成。
- 官方 README 链接的 G2PW 固定 `0c47645e02a7bc3688d7b263b0042c81e3cd82cd`；ZIP SHA-256 已匹配元数据，已解压至缓存。见 [计划](gsv-v5-20261102/g2pw-plan.json)及[准备状态](gsv-v5-20261102/preparation-state.json)。
- 中英文测试参考复用旧 VoiceDesign 纯合成输出，核对原哈希后复制到缓存；本次没有新增音频入库，不使用真人/声优/动漫角色声音，不选择她的正式声音。
- 初次依赖解析遭遇 PyPI msgpack TLS EOF；初次 PyTorch 下载也遭遇 TLS 握手失败，其 uv 内置重试全部如实保留。用户允许一次重试后设置 `UV_HTTP_RETRIES=0`；重试退出 2。见 [首次解析](gsv-v5-20261102/resolve.log)、[首次下载](gsv-v5-20261102/install-torch.log)、[批准重试](gsv-v5-20261102/install-torch-retry1.log)。
- 用户随后批准先查本地资源：找到与历史 P0-07 记录相同的 torch/torchaudio 2.10.0+cu128 缓存。共 11,841 个文件的 RECORD 摘要/长度全部一致，缓存 HTTP 元数据含与本次官方索引相同的 wheel SHA-256；原 wheel 压缩包不在缓存中，未声称重新对原 wheel 做完整哈希。见 [本地核验](gsv-v5-20261102/local-torch-verification.json)。uv 离线解析仍拒绝安装，见 [离线结果](gsv-v5-20261102/install-torch-local.log)，没有手工改缓存索引或重建 wheel。
- 随后按用户批准的顺序，以 curl `--continue-at - --retry 0` 单独下载官方 2.7.1+cu128 wheel；TLS 握手失败，退出 35，未接收 wheel 数据。见 [官方索引及期望哈希](gsv-v5-20261102/torch-official-download.json)、[直连失败](gsv-v5-20261102/torch-official-resume.log)。停止，没有换镜像、改代理/VPN/系统设置或关闭 TLS 校验。

## 依赖与许可

| 种类 | 名称 / 固定版本 | 许可证 / 状态 |
| --- | --- | --- |
| 代码 | RVC-Boss/GPT-SoVITS，`04a44eb977586645f33594b72af13a035a2056c6` | MIT；仅下载到缓存 |
| 预训练权重 | lj1995/GPT-SoVITS，`8f8857563c040d542de108a2c18b0425abd6c9c5` | 官方模型卡 MIT；部分下载，未安装验收 |
| 预训练权重 | XXXXRT/GPT-SoVITS-Pretrained 的 G2PWModel.zip，`0c47645e02a7bc3688d7b263b0042c81e3cd82cd` | 模型仓库卡 MIT；下载、哈希核对及解压完成 |
| 拟用运行时 | torch / torchaudio 2.7.1+cu128 | BSD；下载失败，未安装 |
| 本地复用候选 | torch / torchaudio 2.10.0+cu128 | BSD；缓存核验通过，离线安装失败 |

尚未增加产品运行时依赖。其他候选依赖见 [requirements.in](gsv-v5-20261102/requirements.in)，**未成功解析为完整固定版本清单，不算已登记可交付依赖**；完成安装后须另行登记全部实际版本与许可证。原 Qwen 依赖及 LGPL 条件决定保留在历史报告。

来源：[官方代码](https://github.com/RVC-Boss/GPT-SoVITS/tree/04a44eb977586645f33594b72af13a035a2056c6)、[官方权重](https://huggingface.co/lj1995/GPT-SoVITS/tree/8f8857563c040d542de108a2c18b0425abd6c9c5)、[G2PW 来源](https://huggingface.co/XXXXRT/GPT-SoVITS-Pretrained/tree/0c47645e02a7bc3688d7b263b0042c81e3cd82cd)。

## 验证与未完成项

| 检查 | 结果 |
| --- | --- |
| merge 冲突处理、package.json 解析、新增脚本差异格式 | 通过；双方脚本保留 |
| 本轮报告差异、相对链接、JSON 解析 | 通过；原始证据保留 CRLF，差异检查用进程级 core.whitespace=trailing-space,space-before-tab,cr-at-eol；普通检查会将 CR 报为尾随空白。两份历史副本 Git blob 与改写前一致 |
| 缓存 torch/torchaudio 文件核验 | 通过；边界见上文 |
| 安装环境 / 最小真实合成 | 失败 / not_run |
| provider 切换、取消回归、健康与离线回归、所涉语言检查 | not_run；未开始实现 |
| 模型加载后显存、专用/共享独立峰值、torch allocator | not_run |
| 首个声音 P50/P90、合成速度 | not_run |
| 连续 30 分钟内存稳定性 | not_run |

GPU 从未启动。runner 检查仍见 Listener；安静窗口尚未进入。没有载体同驻或 GPU 替代测量。未查询、等待或报告本轮 CI。

## 下一步

见 [BLOCKED.md](BLOCKED.md)。请用户检查网络链路后明确恢复；镜像需要另行批准且必须匹配官方 SHA-256。用户后续授权每阶段推送一次：当前先推送恢复、缓存复用证据，保持草稿；完整完成前不写“请审查”、不合并、不取下一张卡。

## 2026-11-02 后续恢复：PyTorch 本地复用阶段

已 merge main 0a66e3fa，规则新增设计/研究发散要求，CONTRACTS 新增 touch 后继说明，不改变本卡 TTS 读法。DESIGN-VRAM-01 仍未合入。

上次 uv 离线解析失败不能证明解包缓存不可用。本轮直接逐文件验证缓存与复制后的 RECORD 内容，将 torch/torchaudio 2.10.0+cu128 库文件及九个固定版本依赖复制到本卡独立环境，未改共享缓存，未构造或冒充原始 wheel。Python 库入口可用，未创建 torchrun 等命令行包装。CPU 导入和求和检查通过（exit 0），CUDA 尚未初始化；NumPy 尚未安装的告警保留，模型运行未验证。

证据：[复制程序与核验](gsv-v5-resume-02/torch-reuse.json)、[九个依赖的来源](gsv-v5-resume-02/torch-dependencies-reuse.json)、[CPU 导入原始输出](gsv-v5-resume-02/torch-import.log)。本轮 uv 对依赖的离线解析仍失败，保留 [日志](gsv-v5-resume-02/torch-dependencies.log)，随后按用户批准的直接复用路径处理，不下载替代 PyTorch。

PyTorch 下载不再是当前阻塞；其余 GPT-SoVITS 依赖、完整权重、音频读取依赖、最小 GPU 合成和正式测量仍未完成。torchcodec 0.10 与 torch 2.10 的对应按官方兼容表，尚未安装。

## 2026-11-02 后续恢复第二阶段：依赖安装成功，音频读取阻塞

本节为最新状态，取代上文历史的“其余依赖未安装”和“先修复 PyTorch 网络”待办。仍为部分交付。

其余依赖安装 exit 0，uv pip check 对 115 个包通过；所有实际包名、版本和许可证元数据已登记在 [installed-dependencies.json](gsv-v5-resume-02/installed-dependencies.json)。该清单是代码依赖，预训练权重来源/版本/许可仍单列在上文。TorchCodec 0.10.0 为 BSD-3-Clause；音频依赖真实加载尚不通过。

首次以 torchaudio.load 读取缓存中的合成 zh.wav，exit 1，无法加载 libtorchcodec 或其 DLL 依赖。CUDA 未初始化。现有 ffmpeg 目录只有三个可执行文件，缺少共享 DLL 是待核实方向。未更换读取工具或重试；按用户要求停止。见 [完整原始日志](gsv-v5-resume-02/audio-read-check.log)。全部 GPU 指标、30 分钟稳定性、provider 替换及回归仍 not_run，权重下载仍部分完成。

许可登记发现 Distance 0.1.3 的元数据为 GPL，frozendict 2.5.1 为 LGPL-3.0-only，soxr 1.1.0 为 LGPL-2.1-or-later；当前仅缓存实验环境，产品未新增依赖。不得直接将含 GPL 的该环境接入应用；新增许可边界及恢复选项见 [BLOCKED.md](BLOCKED.md)。报告不将元数据收集等同于许可审核通过。

PyTorch 复用阶段已推送 684bee46；本阶段证据随后单独推送。仅报告与证据有变更，保持草稿和 needs-user。下一步等待用户授权处理 DLL 阻塞及依赖许可范围；不合并、不取新卡。

## 2026-11-02 选项 1 恢复：发布查询限流，停止

本节取代前面的等待授权状态：用户已批准选项 1 及 frozendict 条件使用。已 fetch/merge 最新 main，新增 DESIGN-VRAM-01 报告已阅读；同驻仍不在本卡范围。

固定 GPT-SoVITS 04a44eb977586645f33594b72af13a035a2056c6 的 TTS.py:119/871 确认参考读取使用 torchaudio.load。g2p_en 2.1.0 METADATA 声明 Distance，但 g2p.py/expand.py 未导入；尚未移除或实测无 Distance 推理。models.py 顶层导入 f5_tts DiT，einx 的 lru_cache.py 导入 frozendict；中间完整链尚待记录。TTS.py:1732–1733 的 speed_change 调用已被注释，与用户描述有差异；本卡仍固定 speed_factor=1.0，不依赖系统 ffmpeg.exe。

[dll-check.json](gsv-v5-resume-03/dll-check.json) 记录 PE 普通导入表及逐项加载：core4–8 的非 FFmpeg 直接依赖均可加载，包括 c10/torch_cpu、MSVCP140、VCRUNTIME140/140_1 和 CRT；未解析项全部为 FFmpeg DLL。此检查不声称覆盖延迟导入或未来 FFmpeg 包内部依赖。只添加进程内 torch/lib DLL 目录，未改全局设置、未初始化 CUDA。

BtbN latest 元数据获取成功，但 latest 是可变标签，不能作为固定发布完成交付。继续查询日期发布时 GitHub API 返回 rate limit exceeded，命令 exit 1，按用户要求停止，未尝试其他工具/接口，也未走选项 2。[失败证据](gsv-v5-resume-03/release-query-failure.txt)。没有下载 FFmpeg，SHA-256、LGPL 版本和无 enable-gpl 构建证据均待取得，不能提前登记为通过。D 盘 137.63 GiB 可用，FFmpeg 预计预留 1 GiB。

frozendict 2.5.1（LGPL-3.0-only）按本次用户批准登记：不修改、独立安装动态导入、保留 LICENSE.txt、允许替换、P7 打包前复审。批准不等于推理验证通过。Distance 排除、音频重测、权重补齐、CPU 冒烟和 GPU 全部未执行；环境不宣称就绪。

下一步等待用户允许在限流解除后继续查询固定发布（或指定具体发布 URL）；仍采用选项 1。保持草稿，推送此次证据，不申请 GPU 窗口，不停 runner。

## 2026-11-02 非 API 固定发布恢复：FFmpeg 和音频读取通过

最新状态取代此前 API 限流阻塞。git 标签已复核（最新已出现 11-02，仍固定用户指定 11-01）。下载 URL 前缀：https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2026-11-01-12-49/ ，文件 checksums.sha256 与 ffmpeg-n8.1.3-29-gad3b795b72-win64-lgpl-shared-8.1.zip。ZIP SHA-256 786b43f77ea260af71fbb1bcdaabd43df83492122fb5575027f659d490d30b37 与发布清单一致；长期保存在 D:\Dev\cache\p4-02\gsv-v5-resume-04\，不依赖日期标签永久可下载。

解压六个 DLL avutil-60、avcodec-62、avformat-62、avfilter-11、swscale-9、swresample-6 均在 bin。直接执行缓存内 ffmpeg -version，仅核验配置；无 enable-gpl/nonfree，有 enable-version3，随包 LICENSE.txt 为 LGPL v3。版本 n8.1.3-29-gad3b795b72-20261101；原许可证保留。未使用系统 ffmpeg.exe、未改 PATH/注册表等全局设置。

[audio-check.log](gsv-v5-resume-04/audio-check.log)：仅 os.add_dll_directory 添加缓存 bin 后，TorchAudio 读取 zh/en 合成 WAV 均成功（24000 Hz；190080/203520 samples），CUDA 未初始化。构建配置、许可证、校验清单及脚本均在同目录。FFmpeg 音频加载阻塞已解除；Distance 排除、最小推理、权重与 CPU 冒烟仍待完成，GPU 未执行。PR 保持草稿，本阶段先推送。
