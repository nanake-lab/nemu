# P0-08 证据索引

按 AGENTS.md 第 6 节「证据放哪里、记什么」（2026-11-10 用户决定），下列原始证据已移到私有证据仓库 `nanake-lab/nemu-evidence`（提交 `50f5ea69cc4d13fb93a579e458eda83346e4ab04`），内容逐字节未改。主仓库里同名文件是脱敏版：只去掉了用户电脑上其他程序的名称与路径，项目自己的进程（electron、node、chrome-headless-shell、Runner.Listener、Runner.Worker）和端口等其余字段原样保留。

| 文件（相对本目录） | 原件 SHA-256 | 主仓库中的处理 |
| --- | --- | --- |
| `part2/protocol/gpu-processes-before.csv` | `fa6354057b435eae1ad8d9baca111339801e6b99cb00b268b3cc83a3e72311ae` | 脱敏：原 27 行均不属于保留的项目进程，只留总数 |

原件位置：`nanake-lab/nemu-evidence` 仓库的 `P0-08/` 下，路径与上表相同。主仓库 git 历史中的旧版本未改写。
