# 日志 3→4 补测的完整失败证据

前次实际运行提交 `072141f7112728ba05ebe70fe95de7468bb14221`；本次实际运行提交 **`7ab742037a9203e9b5d671d8962598a3bdb0f311`**。`context/run-binding.json` 记录用户批准、完整 HEAD、入口无 runner 的检查和输入目录；`context/exit.json` 记录起止时间和退出码。

本次只把 server 日志级别从 3 改为 4，原判据未改、注入未重跑。Perth 2026-10-26 12:59:48–12:59:59，server 健康就绪后缺少要求的后端加载消息而被入口结束。正式 benchmark 请求 0，Q8_0 未运行；不能用启动耗时代替 TTFT/prefill 或据此推断 CPU 回落。

| 原判据/事实 | 证据 |
| --- | --- |
| verbosity = 4 | raw/Q4_K_M/server/server.stderr.log 第 3 行 |
| 实际使用 CUDA0 / RTX 3080 | 同文件第 67 行 |
| 37/37 层 offload | 同文件第 147 行 |
| `loaded CUDA backend` | 全部原始日志中未匹配，入口因此拒绝 |
| server 退出 | 主动结束，return 1，见 shutdown.json |
| sampler 退出 | return 0；但只有 1 行、匹配本进程 PID 为 0 |
| 显存峰值 | **未测得**；memory-empty-aggregation.json 中 0 是空集合汇总，不能当作实测 |

`raw/` 按原目录结构保留全部 10 文件/28,453 bytes，没有过滤日志或失败。`inventory.json` 是机械哈希清单，复制时再次核对 SHA-256。清单生成命令在末尾打印摘要时误用了不支持的 Select-Object 参数，退出 1；哈希及 JSON 已写出并只读核对，详见 reporter-command-note.json，不将其伪报为命令退出 0。

后续建议只调整参数顺序：`proposed-log-order.patch` 将已批准的日志 4 移到首个选项；参数值和所有 if 条件的 AST 静态检查均未改变。**补丁未应用，调整后的启动行为未实测**。详细静态调用链见上一级 BENCHMARK-RERUN.md。
