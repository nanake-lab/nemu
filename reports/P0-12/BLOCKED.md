# P0-12 阻塞状态

## 测量运行阻塞已解除

原 curl 56 下载中断，经用户批准续传后完成。原第四项 `POST /slots/0?action=erase` 的 HTTP 501 来自本代理漏传 `--slot-save-path`；用户本次明确批准修正后，实际命令仅增加该参数对，指向 P0-12 缓存的新建空目录。没有 save/restore，旧命令和原始失败不覆盖。

本次④–⑦预定请求均已执行完毕，无新的运行失败。前三项没有重跑，GPU 已退出，已通知恢复 runner。证据与原始失败见 [REPORT.md](REPORT.md)、[新旧命令](continuation-02/command-comparison.json)、[raw](raw/raw-filemap.json)、[raw-02](raw-02/raw-filemap.json)。

## 尚未完成的验证与观测

- `mypy --strict` 未运行：既有环境没有 mypy；没有自行安装新依赖。因此不声称全部工程验证完成，PR 仍为 draft。
- PNG 解码/resize/归一化没有独立计时；图像编码只给出可观测日志区间，包含主机开销。未做图片识别/OCR准确率或 TTS/渲染同驻验收。
- 不为来源检查计数、显存、速度或稳定性设阈值，不判定模型通过，不触发回退。

用户可审查已有测量与这些限制，另行决定是否要求补静态检查或额外观测；本次不再运行，不取下一卡。
