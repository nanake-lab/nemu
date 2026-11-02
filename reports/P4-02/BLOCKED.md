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
