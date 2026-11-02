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
