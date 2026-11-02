# P5-05 证据归档核验

机械性核验由 `routine_reporter` 角色子代理执行，主代理检查返回证据。
子代理未改代码、未重跑测试、未提交/推送、未操作 PR，也未作验收或审查裁决。
本记录不是高难度卡的独立交叉审查。

| 对象 | 核验结果 |
| --- | --- |
| `W22/rust-20261026-1900/workspace.log` | unit 67、integration 17、doc 1；failed/ignored 均为 0；这是合并 `86779b1` 前的运行 |
| `SYS09/os-20261026-1901/original-logs.zip` | 5 个 entry 均与哈希清单及 cache 原件一致；无缺件、额外 entry 或差异 |
| `W22/rust-20261026-1900/original-logs.zip` | 2 个 entry 均与哈希清单及 cache 原件一致；无缺件、额外 entry 或差异 |
| `source-hashes-20261026.json` | 19 个路径与工作树原始字节哈希一致 |
| `SYS09/os-20261026-1901/isolation.log` | 6 条 PASS，含 14 次 Medium 写尝试拒绝、标签/内容不变、Low 正对照和后代清理；probe 原件为 `PASS` |
| `W22/rust-20261026-1926/results.json` | tested_head 为 79b295b；fmt/Clippy/workspace 三条退出码均为 0 |
| `W22/rust-20261026-1926/workspace.log` | unit 88、integration 17、doc 1 passed；0 failed；1 ignored 为 main P5-25 的旧舍入诊断，理由见 REPORT.md |
| `W22/rust-20261026-1926/original-logs.zip` | 5 个 entry 均与清单及 cache 原件一致；无缺件、额外 entry 或差异 |

日志记录的是 Medium/Low 标签断言通过；数值 RID 未单独输出，不将代码中的
`S-1-16-4096` / `S-1-16-8192` 常量伪装成单独采集的观测值。
退出码由主代理的实际工具调用记录；构建日志本身只有成功摘要、警告/错误数量。
Git 对文本可能规范化换行，字节核验使用 ZIP 中的原件。

后继任务定义位于 [PR #102](https://github.com/nanake-lab/nemu/pull/102)，
提交 `6e77cd8391a346f9db00058a6cac27749a86b0af`。只有两个 `tasks/` 文件变更；
新增 P5-40，已有卡记录无语义变化。共 101 张卡，依赖引用有效、无环；P5-40 为 E4，
依赖 P5-05/P5-21/P5-22/P5-35/P5-14/P5-29/P5-15，均不晚于 E4。
[Docs guard 40309886052](https://github.com/nanake-lab/nemu/actions/runs/40309886052)
在该提交通过。任务定义 PR 保持 `needs-user`，未合并。
