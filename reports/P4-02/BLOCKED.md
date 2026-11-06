# 当前状态（2026-11-04）：GPU 监视器 PID 归属错误，首轮已停止

桌面背景已获批准。监视器把实际 TTS 子进程误判为外部负载，模型加载期间停止；无首声样本，30 分钟未开始。待修复监视器并获准重跑。详细原因与证据见末尾。

# 当前状态（2026-11-04）：GPU 窗口预检未满足，未开始测量

用户已授权窗口；runner 未运行，但 nvidia-smi 显示其他 C+G 进程及约 1178 MiB GPU 占用。按用户要求停止，待清理占用或明确允许桌面背景基线。详见末尾 2026-11-04 记录。CPU 禁网通过结果仍有效。

# 当前状态（2026-11-03）：CPU 禁网真实合成通过，等待 GPU 窗口

中文和中英混合真实推理均通过，CUDA 未初始化、G2PW CPU-only、无联网请求。先前 A/B/C 与下载阻塞已解除；P7 待决项仍保留。当前等待用户确认约 45–60 分钟独立 TTS GPU 窗口；尚未实施 provider 替换或 GPU 验收。详细结果见文件末尾和 REPORT。以下历史原文保留，不代表当前阻塞。

# 当前状态（2026-11-03）

等待集中数据审计 A/B/C 的许可与来源决定。fastText 已批准；请看 [统一审计表](gsv-v5-resume-08/RUNTIME-DATA-AUDIT.md)。bin 尚为部分文件，真实禁网合成未运行。下文保留历史状态；最新细节见文件末尾。

# 当前阻塞：参考音频读取失败（2026-11-02，后续恢复第二阶段）

以下最新状态取代后文历史状态。PyTorch TLS 下载阻塞已由本地缓存复用绕过，网络设置未改变。

- torch / torchaudio 2.10.0+cu128 缓存哈希核对及 CPU 运算通过。
- 其余推理依赖安装退出 0；uv pip check 检查 115 个包通过。
- 首次 CPU 读取合成 zh.wav 退出 1：`RuntimeError: Could not load libtorchcodec`。原始错误列出 FFmpeg 4–8 对应 DLL 或依赖无法加载。TorchCodec 0.10.0 的 core DLL 实际存在；D:\Tools\ffmpeg\bin 只有 ffmpeg/ffplay/ffprobe 三个 exe，没有共享 DLL。缺少共享库是待核实方向，尚未定位全部缺失依赖。没有换音频库、降级或重试。
- CUDA 未初始化；权重未补完，provider 未切换，全部 GPU 测量 not_run。Runner.Listener 仍在运行。
- 安装后许可清单另发现 Distance 0.1.3 标为 GPL，frozendict 2.5.1 为 LGPL-3.0-only；仅在缓存实验环境中，尚未加入产品。AGENTS 第 3.8 条禁止应用引入 GPL，因此不能把这份环境直接作为产品依赖交付。既有 LGPL 批准不能自动覆盖新增依赖。

证据：[安装日志](gsv-v5-resume-02/install-gsv-dependencies.log)、[依赖检查](gsv-v5-resume-02/dependency-check.log)、[音频读取原始错误](gsv-v5-resume-02/audio-read-check.log)、[检查程序](gsv-v5-resume-02/audio-read-check.py)、[实际版本与许可证元数据](gsv-v5-resume-02/installed-dependencies.json)。元数据登记不等于完整分发许可审核通过。

## 恢复选项（尚未执行）

1. 用户授权后，检查 DLL 依赖链并在缓存内准备许可合规的 FFmpeg 共享库，固定版本/哈希，仅进程内配置加载路径；同时核对中文最小推理是否依赖 Distance，提出符合 GPL 禁令的依赖裁剪方案，再重新验证。
2. 用户授权后，恢复官方 2.7.1 wheel 断点续传及哈希核验，按上游参考版本重新准备独立环境；先解决现有 TLS 问题，不假定降级一定解决音频和许可问题。

按用户“遇到失败即停”要求，现停在此处，等待恢复方向。D 盘最近可用 137.66 GiB；没有删除文件。此次推送阶段证据，PR 保持草稿；不请求完整验收审查。当前无需为本卡停止 runner，正式 GPU 窗口仍待环境就绪后安排。

---

以下保留前阶段原文，供历史追溯，不代表当前待办。
# 最新状态：PyTorch 缓存复用已解决（2026-11-02 后续恢复）

已按用户批准直接复用本机 2.10.0+cu128，源/目标文件哈希核对、CPU 导入与运算通过；CUDA 未初始化。当前继续准备其余 GPT-SoVITS 依赖与权重，不能声称完整环境或 GPU 验收通过。用户新指示允许每阶段推送一次，保持草稿。下文是已保留的原下载阻塞及失败证据，不再要求先修好 PyTorch 网络才能继续；不代表 TLS 问题已修复。详见 [报告](REPORT.md#2026-11-02-后续恢复pytorch-本地复用阶段)。

# P4-02 当前阻塞 — 2026-11-02

**GPT-SoVITS V5 环境准备失败，已停止。** 本机验收、provider 切换和所有 GPU 测量均未完成，PR #45 保持草稿及 needs-user；paused 已按用户指令移除。

## 已试过及证据

1. 官方 PyTorch 2.7.1+cu128 安装：TLS handshake EOF；[初次日志](gsv-v5-20261102/install-torch.log)含 uv 内置重试。用户批准一次重试，关闭 uv 自动重试后仍失败，退出 2：[重试日志](gsv-v5-20261102/install-torch-retry1.log)。依赖解析另有 PyPI msgpack unexpected EOF：[解析日志](gsv-v5-20261102/resolve.log)。
2. 用户授权检查本地资源：缓存 torch/torchaudio 2.10.0+cu128 共 11,841 个文件的 RECORD 哈希和长度一致，元数据包含官方索引公布的 wheel 哈希；[核验记录](gsv-v5-20261102/local-torch-verification.json)。原 wheel 压缩文件未找到，旧 P0-07 环境路径已不存在；uv 离线安装仍拒绝解析：[日志](gsv-v5-20261102/install-torch-local.log)。不把缓存核验通过说成运行环境已就绪。
3. 按用户授权，官方 wheel 文件用 curl 断点续传方式单独下载：`--location --continue-at - --retry 0 --connect-timeout 30`。退出 35，`schannel: failed to receive handshake, SSL/TLS connection failed`，没有收到安装包数据：[日志](gsv-v5-20261102/torch-official-resume.log)。官方索引读取成功，wheel 下载握手失败；不据此断言 VPN、代理或杀毒软件中的哪一个是根因。

固定的官方目标和 SHA-256 在 [torch-official-download.json](gsv-v5-20261102/torch-official-download.json)。下载目标为 `D:\Dev\cache\p4-02\gsv-v5-wheels\torch-2.7.1+cu128-cp312-cp312-win_amd64.whl`；恢复时继续使用断点续传，不删除已有内容重下。只有完整文件匹配官方 SHA-256 后才允许本地安装。部分模型文件及所有失败日志均保留。

## 等用户决定

- 请用户检查 VPN、代理或杀毒软件 HTTPS 扫描；代理未改变用户/系统设置或关闭证书验证。网络恢复后，授权继续同一官方断点续传路径及剩余依赖准备，代价是下载时间。
- 或由用户提供已下载的官方 wheel，仍先核对上述官方 SHA-256；可避免当前机器的下载连接问题。
- 如要改用镜像，必须用户另行批准；仍需按官方 SHA-256 完整核对，不能仅相信镜像提供的哈希。

GPU 尚未运行，当前无需为本卡保持 runner 停止。环境恢复后再安排 45–60 分钟窗口（含连续 30 分钟合成），重新确认 Listener/Worker 都停止。先做最小真实可行性测量，再实施 provider 替换；本次只测量，不补新阈值。

原 Qwen 的测量失败与许可决定在 [历史阻塞](QWEN3-TTS-BLOCKED-HISTORY.md)及[历史报告](QWEN3-TTS-HISTORY.md)原文保留，不追溯改判。当前没有声称 GPT-SoVITS 可用，旧默认 provider 代码尚未替换。按用户的验收后推送边界，本轮不推送、不请求审查。

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

## 最新阻塞：CPU TTS 导入与权重连接（2026-11-02）

FFmpeg 阶段已推送 d63c90c5，保持有效。Distance 0.1.3 已卸载；以 --no-deps 检查 g2p_en 2.1.0 及显式 nltk/inflect/numpy 成功，未修改第三方元数据或源码。无 Distance 的 module.models CPU 导入通过，frozendict 确实加载，CUDA 未初始化。完整链见 [frozendict-chain.txt](gsv-v5-resume-04/frozendict-chain.txt)，许可证原文已保留。删除 Distance 后不能再声称所有包元数据依赖满足；它仍由 g2p_en 元数据声明。中文/中英混合真实推理未完成，不能以导入通过代替。

随后 CPU TTS 导入失败，exit 1：AR/models/t2s_lightning_module.py -> AR/modules/lr_schedulers.py:6 -> matplotlib，ModuleNotFoundError。此前最小依赖清单漏列该包，这是环境准备遗漏，非模型硬件失败。见 [tts-import.log](gsv-v5-resume-04/tts-import.log)。按用户失败即停要求，没有安装后自行重试。

固定权重续传同时未完成：curl 返回 exit 56，schannel: server closed abruptly (missing close_notify)。停止时检查本轮 curl 进程，未找到仍活跃的匹配进程，不能说是主动终止导致网络退出；原始日志保留。s2Gv5turbo.pth 现为 337652192 bytes 部分文件，未通过完整哈希；没有删除或重下。vocoder/s1 尚未下载。

当前停工，未转选项 2、未换镜像、未更改网络或系统设置、未申请或进入 GPU 窗口。下一步需用户授权补入固定版本 matplotlib 及许可登记、重跑 CPU 冒烟，并继续同一固定权重的断点续传；新失败继续保留。所有真实合成与 GPU 指标仍 not_run，PR 草稿。

## 最新状态：Pillow 许可需确认（2026-11-02）

用户已预授权缺失宽松许可包自动补齐和每文件最多 20 次权重续传，已取代此前逐次网络恢复授权。fetch/merge 后 main 无新增变化。

matplotlib 3.10.8 已核对为 Matplotlib License；其必需传递依赖候选固定为 contourpy 1.3.3（BSD-3-Clause）、cycler 0.12.1（BSD）、fonttools 4.61.1（MIT）、kiwisolver 1.4.9（BSD）、pyparsing 3.2.5（MIT）、python-dateutil 2.9.0.post0（BSD/Apache-2.0）、six 1.17.0（MIT），已有 numpy/packaging 可满足。尚未安装任何上述新增包。

Pillow 12.0.0 的 PyPI 元数据明确使用 MIT-CMU；补查 11.2.1 同为 MIT-CMU，不是 HPND。MIT-CMU 是独立 SPDX 标识，本轮不能自动把它归入用户列出的 MIT；按“其他许可停下”边界，等待用户确认是否允许该许可。没有安装后回退、没有修改第三方源码。见 [候选元数据](gsv-v5-resume-05/matplotlib-dependency-metadata.json)及[补查](gsv-v5-resume-05/pillow-approved-metadata.json)（文件名中的 approved 不代表用户已经批准）。如批准，可采用原候选 Pillow 12.0.0；或用户指定其他许可/版本路线，重新核验。

权重续传已按授权写入脚本：同 revision，curl --continue-at -；网络错误间隔 30 秒，最多 20 次尝试（保守地将首次也计入上限）；成功后同时核对大小与 SHA-256。此次第 1 次尚在传输时因上述许可边界停止本卡，由代理终止已核对归属的本轮 PowerShell/curl，不是新的网络失败。s2Gv5turbo.pth 保留 420141536 bytes，未完整校验；其余文件未开始。脚本、原日志和终止进程记录在本节证据目录，未换镜像、未改全局设置。

Distance 仍为有意排除：g2p_en 声明了它但已检查代码未导入，因 AGENTS 3.8 的 GPL 禁令排除，未修改第三方元数据。已有 frozendict 条件批准与 FFmpeg LGPL 批准继续有效。

CPU 真实中文/中英混合推理未运行，GPU 未运行。当前不是 GPU 窗口等待；先等 Pillow MIT-CMU 许可决定。保持 PR 草稿，推送本阶段证据。

## 2026-11-02：依赖补齐与 TTS CPU 导入通过

用户已明确批准 Pillow 12.0.0（MIT-CMU），并将 MIT/BSD 家族变体及 Zlib 纳入后续宽松许可预授权。许可清单按实际 SPDX MIT-CMU 登记，不把不同 SPDX 标识静默改写为 HPND。随包版权与许可文本保留在独立缓存环境。

已安装固定版本 matplotlib 3.10.8（Matplotlib License）、contourpy 1.3.3（BSD-3-Clause）、cycler 0.12.1（BSD）、fonttools 4.61.1（MIT）、kiwisolver 1.4.9（BSD）、Pillow 12.0.0（MIT-CMU）、pyparsing 3.2.5（MIT）、python-dateutil 2.9.0.post0（BSD/Apache-2.0）、six 1.17.0（MIT）。采用 --no-deps 显式固定上述安装项，未整体安装上游 requirements。完整环境元数据在 [installed-dependencies.json](gsv-v5-resume-06/installed-dependencies.json)。Distance 仍有意排除，原因及元数据不满足边界保持不变。

[安装日志](gsv-v5-resume-06/install-matplotlib.log) exit 0；[TTS 模块导入](gsv-v5-resume-06/tts-import-01.log) exit 0，CUDA 未初始化；上游 SyntaxWarning 原样保留。这只是导入，不是模型加载或真实合成。权重续传仍在进行，后续 CPU 中文/中英混合推理待完整哈希核验。PR 保持草稿，本阶段推送。

## 最新阻塞：CPU G2PW provider 隔离准备的 TLS EOF（2026-11-02）

依赖安装与 TTS 导入阶段已推送 90075936。随后 CPU 冒烟静态核对发现：固定上游 text/g2pw/onnx_api.py:343–353 检查 get_available_providers；存在 CUDAExecutionProvider 就创建 CUDA 优先 session，不受 TTS 的 device=cpu 参数控制。当前安装 onnxruntime-gpu 1.19.2，因此只检查 torch.cuda.is_initialized 不足以证明整个推理未碰 CUDA。未启动这条真实推理路径。

拟为 CPU 冒烟另备同版本 CPU onnxruntime（独立缓存路径加载，保留原 GPU 环境、不改第三方源码）。在核对 https://pypi.org/pypi/onnxruntime/1.19.2/json 元数据时，Invoke-RestMethod 报 Received an unexpected EOF or 0 bytes from the transport stream。PowerShell 非终止错误导致 shell exit 0，但 HTTP 请求失败；后续空元数据不算许可核验。见 [原错误转录](gsv-v5-resume-06/onnxruntime-metadata-failure.txt)。没有重试/更换访问工具/安装 CPU 包；这不是用户授权自动续传的权重请求，故按其他失败停下。

权重第 1 次续传仍在进行时由代理终止本轮已核对归属的下载进程，非网络错误；部分 s2Gv5turbo.pth 保留 554826208 bytes，未完整校验。完整脚本/日志/终止归属记录已保存。此次将新尝试上限保守设为 19，加上前轮因许可停止的 1 次不超过 20；未用自动重试隐藏原始失败。

下一步等待用户授权重取 ONNX Runtime CPU 包元数据并准备 CPU-only provider（核对固定版本与许可），随后继续已有权重断点及 CPU 真实中文/混合推理。尚不满足申请 GPU 窗口的前置。仍为草稿，GPU 与真实合成全部 not_run；未改用户或系统网络设置。

## 2026-11-02：G2PW 进程内 CPU 隔离通过

用户已批准进程内替换 ort.get_available_providers，只返回 CPUExecutionProvider；不另装 CPU 包、不改第三方源码。固定上游 GPT_SoVITS 下 InferenceSession 搜索仅发现 text/g2pw/onnx_api.py 两个互斥分支和不在推理路径中的 onnx_export.py。

实测 onnxruntime 1.19.2 原 provider 列表包含 TensorRT/CUDA/CPU，preload_dlls 属性不存在；上游 onnx_api.py:23 调用由 except Exception 跳过。替换后初始化 G2PW，断言 session_g2pw.get_providers() == ['CPUExecutionProvider'] 通过，同时 torch.cuda.is_initialized() == False。见 [原始日志](gsv-v5-resume-07/g2pw-isolation.log)与[脚本](gsv-v5-resume-07/g2pw-isolation.py)。本阶段尚未加载 TTS 权重或合成。

后续 provider/显存设计输入：G2PW 自动按可见 CUDA provider 选 GPU，不受 TTS device 参数控制，正式 provider 必须显式指定 G2PW 的 provider；升级 ORT 时重新核对 preload_dlls，不能沿用 1.19.2 的跳过结论。当前替换仅在 CPU 冒烟进程内生效，未实施正式 provider。

此前 CPU 隔离阻塞已解除。剩余固定权重续传进行中，真实 CPU 推理待完整大小/哈希通过。用户已扩大固定来源瞬时网络重试授权，每请求最多 5 次、间隔 30 秒并逐次留日志；权重仍最多 20 次。PR 继续草稿。

## 2026-11-02：固定权重补齐通过

固定 HuggingFace revision 8f8857563c040d542de108a2c18b0425abd6c9c5 的剩余三文件均完成：s2Gv5turbo.pth 766406144 B、vocoder.pth 57761163 B、s1v3.ckpt 155284856 B；完整大小和 SHA-256 均与原 download-plan.json 一致，见 [verification.jsonl](gsv-v5-resume-07/verification.jsonl)。加上先前已核验的六个文件，计划内权重已齐。

本轮主权重与 s1v3 各 1 次，vocoder 第 1 次 exit 56/missing close_notify 后等待 30 秒，第 2 次从断点成功，失败原日志和退出码保留。未换镜像或全局设置。本阶段推送后再做 CPU 真实合成，仍非 GPU 验收。

## 最新阻塞：语言识别模型 CC-BY-SA-3.0（2026-11-02）

权重阶段已推送 5b8f4790。CPU 真实冒烟已启动：G2PW session 仅 CPUExecutionProvider，torch.cuda.is_initialized=False；TTS 全模型加载成功，耗时 4.251378700006171 秒。v5turbo/vocoder 加载 keys 全匹配。日志与脚本见 [cpu-smoke.log](gsv-v5-resume-07/cpu-smoke.log)、[cpu-smoke.py](gsv-v5-resume-07/cpu-smoke.py)、[事件](gsv-v5-resume-07/cpu-smoke.jsonl)。事件文件含先前隔离检查的开头两条，后续新 isolation/model_loaded 属真实冒烟；不是两个成功合成。

首条中文进入参考文本处理后，text/LangSegmenter/langsegmenter.py 配置的 fast_langdetect 自动请求 https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.bin，日志出现两次 Falling back to low-memory model 后仍下载同一 bin。底层异常被库捕获，原日志没有完整原因，不能将其擅自归为获准的 TLS 重试。代理发现后终止本轮拥有的 CPU 冒烟 Python 进程，退出 -1；未换模型/工具重跑，部分下载保留。HF_HUB_OFFLINE 对 fast_langdetect 自己的下载器不起作用；本轮代码依赖的自动下载与回退行为已如实记录。

[fastText 官方模型页](https://fasttext.cc/docs/en/language-identification.html)于本日核对：lid.176.bin 与压缩版模型均为 Creative Commons Attribution-Share-Alike 3.0，即 CC-BY-SA-3.0。这是预训练语言识别权重许可，不能以 fastText 或 Python 包代码许可代替，也不属于本轮 MIT/BSD/Zlib 等预授权。新增模型未固定完整哈希、未获许可批准，因此停止。正式 provider 也需控制这条自动下载/回退路径。

中文尚未产生输出，中英混合未执行；真实合成仍未通过。没有进入 GPU 步骤，不申请 GPU 窗口。下一步由用户决定：批准独立使用该 CC-BY-SA-3.0 语言识别模型及其固定下载/署名/分发条件，随后先取完整文件与 SHA-256，禁止运行中隐式下载再重测；或要求另行研究不使用该模型的路线（不得静默改第三方源码/分段行为）。PR 保持草稿，报告所有阶段证据。
## 2026-11-03 集中数据审计：等待三组许可/来源决定

已按用户要求集中核对运行时下载与随包数据，完整统一表见 [RUNTIME-DATA-AUDIT.md](gsv-v5-resume-08/RUNTIME-DATA-AUDIT.md)。表列文件/来源/用途/数据许可/中文与混合路径，未将 Python 包代码许可代替模型或语料许可。静态搜索原始结果和 173 个随包数据文件的 SHA-256 已入证据目录；未宣称穷尽所有可能动态执行，真实禁网冒烟仍是最终漏项检查。

fastText lid.176.bin 与包内 lid.176.ftz 的 CC-BY-SA-3.0 已获用户批准：不修改、独立文件、保留署名和许可声明、P7 打包前复审。补正原 fast-langdetect 0.3.2 清单漏登记模型许可的问题；ftz 为 938013 B，SHA-256 8f3472cfe8738a7b6099e8e999c3cbfae0dcd15696aac7d7738a8039db603e83。署名及许可证链接在审计表。bin 使用用户批准的固定 URL 和上游 cache_dir 续传；集中发现其他未决项后主动停止，现保留 17219584 B 部分文件，未计算/声称完整固定哈希，不可供推理加载。停止退出 -1，非网络失败，恢复应从断点继续。

两个 MIT NLTK tagger 包已按 nltk_data 固定提交 550b6625bcef1f2abff2ff770a5a0d272c9c6b2a 准备在 D:\Dev\cache\p4-02\nltk_data\taggers，保留 ZIP 和解压目录；实际当前 NLTK pos_tag 需要 _eng JSON，g2p_en 导入还检查旧 zip，两者均备齐。XML 与 ZIP SHA-256 在 nltk-hashes.jsonl。

一次性交回 A/B/C：A NLTK CMUdict 0.6 是自定义宽松许可，不在明确 SPDX 预授权内；B wordsegment 的随包词频数据独立许可范围未明确（来源页仅明确代码 MIT）；C GPT-SoVITS 派生热词/姓名/拼音/多音字词表尚缺逐文件来源与许可追溯。详细文件、已有声明与边界均集中在表中，未逐项试错，不把来源不明等同于已确认禁止使用。

CPU 禁网脚本已准备：在第三方导入前替换 socket connect/connect_ex/create_connection/getaddrinfo/sendto，拒绝并记录目标地址；保留 HF offline、进程内 G2PW CPU provider 及 CUDA 未初始化断言，完成时若出现任何网络请求日志也视为失败。三项守卫自检均在发起真实连接前阻止成功，日志已保留；未运行真实禁网合成。该 Python socket 守卫不是系统防火墙，对未识别的原生库直接联网不能作绝对保证。

现按“其他许可一次性列出后再停”停止，等待 A/B/C 的集中决定；未进入 GPU 阶段，不停 runner、不申请窗口。PR 保持草稿；原权重核验、G2PW 隔离和 CPU 模型加载结果仍有效，中文/混合真实合成未通过。


## 2026-11-03 A/B/C 用户决定生效

用户已批准 CMUdict 0.6，wordsegment 数据仅本机测试（P7 三选一待决），上游派生词表以固定仓库 MIT 为使用依据并在 P7 复审。完整文件/来源/原因/影响面见 [P7-PENDING.md](gsv-v5-resume-09/P7-PENDING.md)。此前 A/B/C 本机阻塞解除。明确禁止的仍停；其他灰色项登记后继续。本轮继续 lid.176.bin 断点和 CMUdict 预置，随后禁网 CPU 冒烟，尚未执行 GPU。

## 2026-11-03 资源预置完成

lid.176.bin 固定地址 https://dl.fbaipublicfiles.com/fasttext/supervised-models/lid.176.bin 已从原断点完成，大小 131266198 B，SHA-256 7e69ec5451bc261cc7844e49e4792a85d7f09c06789ec800fc4a44aec362764e；源 URL 无版本，按本地完整哈希固定，不把它说成发布者公布的 SHA。文件长期保存在上游 pretrained_models/fast_langdetect 缓存目录，不修改、不入仓库。CC-BY-SA-3.0 署名及许可链接沿用统一审计表。

前四次连接分别 exit 28/35/35/35，第五次成功；间隔 30 秒，未超过五次，原始日志齐全。CMUdict 固定 revision XML 第三次成功，ZIP 第二次成功；ZIP SHA-256 d07cca47fd72ad32ea9d8ad1219f85301eeaf4568f8b6b73747506a71fb5afd6，已解压到独立 nltk_data/corpora。CMU 出处声明保留在固定 XML 和包内材料。

英文前处理在 socket 禁网、无 Distance 环境通过：本地三份 NLTK 资源可找到，hello world 转为音素成功、无网络请求。见 english-offline.log。SyntaxWarning 保留，不改上游。真实中文与中英混合 TTS 待下一阶段。

前阶段 f0390565 的 git push 遇 TLS 握手失败（原工具输出：schannel: failed to receive handshake, SSL/TLS connection failed）；本阶段产生新资源证据后再次按阶段推送，并携带此前本地提交，不宣称上次已推送。
## 2026-11-03：禁网 CPU 中文/中英混合真实推理通过，等待 GPU 窗口

进程退出 0，固定 v5turbo、float32 CPU、4 线程、seed 42、speed_factor 1.0、非流式/parallel_infer=False，参考为缓存里的纯合成 zh.wav。模型加载 4.319383 秒；中文句“你好，今天的天空很明亮。”生成 48000 Hz / 138240 samples / 2.88 秒音频，合成总耗时 14.566990 秒；混合句“你好，hello world。”生成 48000 Hz / 94080 samples / 1.96 秒音频，总耗时 12.867973 秒。单次 CPU 样本，不代表 P50/P90、GPU 速度或语音质量验收。

G2PW 实际 session provider 断言为 ['CPUExecutionProvider']，全过程记录 torch.cuda.is_initialized=False；无 Distance；socket 连接/解析阻断守卫从第三方导入前启用，完成断言未出现 blocked-network.jsonl，日志无下载/低内存回退。证据 [cpu-smoke.jsonl](gsv-v5-resume-09/cpu-smoke.jsonl) 和 [完整日志](gsv-v5-resume-09/cpu-smoke-offline.log)。输出 WAV 仅在 D:\Dev\cache\p4-02\gsv-v5-resume-09，未入库，SHA-256 在事件中；没有人工听感质量或跨语言能力泛化结论。

A/B/C 本机许可阻塞、资源下载阻塞和 CPU 冒烟阻塞现已解除。P7 待决清单继续有效。正式旁路 provider 替换、取消/健康/离线回归、3080 显存/首个声音分位数/速度/30 分钟稳定性均未完成，PR 保持草稿，不请求整卡完成审查。

下一步申请约 45–60 分钟 GPU 安静窗口（含连续 30 分钟合成；仅独立 TTS，不含载体同驻、不新增阈值）。本次只读进程检查未发现 Runner.Listener/Runner.Worker，但按用户要求仍在 GPU 步骤停下，等待用户确认窗口；执行前再核对，代理不自行停 runner。测完通知用户恢复 runner。

f45406a3 阶段推送另遇 Git LFS locks/verify EOF，未关闭校验。当前再按 CPU 阶段提交并推送，若成功将携带此前本地提交；不等待或查询 CI。
## 2026-11-04 GPU 窗口预检停止：仍有其他 GPU 占用

用户已确认约 45–60 分钟窗口，并明确要求开始前 GPU 上没有其他占用进程。Runner.Listener/Runner.Worker 均未发现；但 nvidia-smi 首次观察为 1177 MiB、7% GPU 利用率，保存证据时为 1178 MiB、7%，进程表包含 Chrome、Edge、ChatGPT、微信、Windows 桌面等 C+G 进程。WDDM 未给出各进程显存，不能据此判断哪个进程承担这些占用，也不能把 C+G 条目当成另一个模型已运行的证据。

因此按本次明确预检条件停止，没有加载 TTS 或初始化 CUDA，没有进行显存、首声 P50/P90、速度或 30 分钟稳定性测量。证据：[nvidia-smi 原始输出](gsv-v5-gpu-preflight-20261104/nvidia-smi.txt)、[预检结果](gsv-v5-gpu-preflight-20261104/result.json)。git fetch origin / git merge origin/main 返回 Already up to date。只记录本次阻塞，保留此前 CPU 通过结果及草稿状态；没有停止其他进程或改变全局设置。

待用户决定：① 用户关闭可关闭的 GPU 应用后重新安排窗口（Windows 桌面进程可能仍常驻）；② 用户允许常驻桌面/交互界面作为明确记录的背景基线，同时排除其他计算与主动渲染负载，再复核开始。选择②会改变本次“没有其他占用进程”的字面条件，代理不自行采用。本次没有启动测量进程，用户可恢复 runner；再次测量前需要重新确认窗口。首声仍按请求发出到第一段可播放音频的墙钟耗时，不以内部生成 token 或排除前处理的耗时替代。
## 2026-11-04：允许桌面基线后的首轮 GPU 测量因监视脚本错误停止

用户已允许常驻桌面/交互程序作为背景，只排除其他推理、游戏和视频播放。前检未发现 runner、其他推理/游戏进程或活动视频解码/处理计数器；Video.UI 常驻无窗口，不据进程存在判作播放。已有 Python 命令为桥接、商务服务、工具或该项目测试，没有 GPU 推理证据。D 盘剩余约 131 GiB，预计本轮证据不足 100 MiB。git fetch/merge 返回 Already up to date。

本轮失败属于代理编写的监视器错误：Start-Process 返回启动器 PID 34876，真实 Python 在 imports_ready 事件报告 PID 26476。监视器只排除 34876，随后把 26476 的 CUDA 引擎 5% 利用率误判为外部重负载，停止本轮。相同错误也使 RSS/Private 采样落在启动器，不能作为 TTS 内存数据。原始日志保留，未自行重跑；这不是 GPT-SoVITS 推理失败或已确认的外部干扰。

证据：[启动脚本](gsv-v5-gpu-20261104/run.ps1)、[事件](gsv-v5-gpu-20261104/events.jsonl)、[负载守卫](gsv-v5-gpu-20261104/load-guard.jsonl)、[原始内存样本](gsv-v5-gpu-20261104/memory.jsonl)、[修正后的部分摘要](gsv-v5-gpu-20261104/partial-summary-corrected.json)。最初 partial-summary.json 同样按错误启动器 PID 统计，保留作失败诊断，不作为测量结果；修正版仅离线从已存 WDDM 行提取实际 Python PID 的加载期间部分值。

[nvidia-smi 开始前](gsv-v5-gpu-20261104/nvidia-before.txt)：背景 1091 MiB / 12%；[停止后](gsv-v5-gpu-20261104/nvidia-after.txt)：1117 MiB / 12%，含完整进程表。这两个值都是整卡桌面背景观察，不能用它们的差值代替 TTS 占用。实际 TTS 的部分 WDDM 专用/共享值单独见修正摘要；未得到 model_loaded 事件，不宣称加载后驻留或完整峰值。未产生首声样本，P50/P90、合成速度与 30 分钟稳定性全部未完成；此前 CPU 通过不受影响。

准备的请求计时从调用 model.run 前开始，到首段完整音频转成 24 kHz little-endian float32 PCM bytes 可供播放为止，包含前处理、非流式整句合成与重采样，不包含尚未接入的 WebSocket/声卡启动。中文与混合各 20 个短句，冷请求另列，连续阶段计划 1800 秒；这些均尚未执行完成。G2PW 保持 CPU provider，TTS 拟用 CUDA float16，speed_factor=1.0，合成参考音频仍仅缓存。

待恢复方案：修正监视器为追踪实际 Python PID/子进程树，先用无 GPU 的进程夹具验证自身排除、外部负载检测和内存归属，再申请重跑同一测量。另一选项是先仅完成监视器修复验收，将 GPU 测量另安排窗口。按“失败即停”，本轮不继续 GPU；已确认没有本轮 Python 残留，通知用户可恢复 runner。PR 保持草稿。
## 2026-11-06：进程归属修复与无 GPU 自检通过

已 merge 最新 origin/main。监视器以 imports_ready 的实际 PID 为根，先沿父链核对至 Start-Process 返回的启动器，再展开实际根的子进程树；启动器只作父链证明，不替代实际进程。内存逐 PID 读取后汇总；负载判定排除整棵已核验子树。

[无 GPU 自检](gsv-v5-gpu-20261106/selftest/result.json)通过三项：自身根/两级子进程排除、外部进程及模拟 GPU 引擎检出、实际 Python 内存归属。夹具仅 Python 标准库，实际根分配 64 MiB、子进程 32 MiB；GPU 引擎行明确是模拟输入，未导入 torch 或初始化 CUDA。根 PID 38092、启动器 19808，父链核对通过，根 RSS 79888384 B / private 75124736 B。脚本与实际证据均保留。此结果验证监视器，不代表 GPU 测量通过。按用户授权，同窗口继续重跑前先核对 runner/负载与背景基线。
