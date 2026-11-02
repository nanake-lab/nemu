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
