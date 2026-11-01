# P0-12 BLOCKED — 第四项 slot erase 返回 HTTP 501

## 当前阻塞

2026-11-02 本地时间，前三项采集结束后，第四项启动的固定 b11669 server 已就绪，但第一次 `POST /slots/0?action=erase` 返回 **HTTP 501 Not Implemented**。4 条预热与 270 条正式文字请求均未发送，第五至七项未运行。原始 traceback、启动 argv、server 日志和 shutdown 记录均在 [raw-filemap.json](raw/raw-filemap.json) 中。

**本代理的测量配置遗漏：** 只传入 `--slots`，漏了启用 slot action 所需的 `--slot-save-path`。固定提交源码的 guard 与实际 argv 支持此原因，见 [server-failure-diagnosis.json](results/server-failure-diagnosis.json)。HTTP 响应正文没有捕获，不能将源码推断冒充响应正文。当前证据不证明 Qwen3.5/b11669 有硬性模型兼容问题。

用户要求“任一步失败即停……不自行换工具、引擎或方法重试”，因此未修改参数或换方法重跑。模型及采样进程均已退出，已通知恢复 runner。

## 已保存的结果

官方 14 个文件全部下载、核验；BF16、Q4_K_M、F16 mmproj 均转换成功。三语零注入各 12/11/12 个位置，所有零注入、清零/撤除配对完整 logits 字节一致；清状态重复及 fresh context 配对一致，保留状态诊断有差异。两个注入位置均保存实际回调和固定 e0 加法证据。这些是观测结果，不签发模型通过结论。

详见 [REPORT.md](REPORT.md)。原先 curl 56 下载阻塞已在用户“继续下载，网络波动时有发生”授权后解决；原失败 partial、日志、哈希均保留，实际续传没有再遇到重试。

## 用户可决定的后续（本次不执行）

1. 授权修正同一 b11669 server 的启动参数，以新输出目录和新的 GPU 安静窗口继续第四至七项；先记录 slot erase 实际状态覆盖与证据，不覆盖本次失败。
2. 保持部分交付，暂不继续本卡；生产仍保留原基线，Qwen3-VL 没有获准下载。

本次不更换工具/引擎、不删缓存、不自动取下一卡。已推送的部分交付可审查，任务仍阻塞。
