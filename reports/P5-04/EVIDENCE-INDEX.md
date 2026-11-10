# P5-04 证据索引

按 AGENTS.md 第 6 节「证据放哪里、记什么」（2026-11-10 用户决定），下列原始证据已移到私有证据仓库 `nanake-lab/nemu-evidence`（提交 `50f5ea69cc4d13fb93a579e458eda83346e4ab04`），内容逐字节未改。主仓库里同名文件是脱敏版：只去掉了用户电脑上其他程序的名称与路径，项目自己的进程（electron、node、chrome-headless-shell、Runner.Listener、Runner.Worker）和端口等其余字段原样保留。

| 文件（相对本目录） | 原件 SHA-256 | 主仓库中的处理 |
| --- | --- | --- |
| `main-sync-20261024/windows-evidence/isolation-after.json` | `ab456d1c67da3e055afeed00198effd4f6b58b8c78609b3d8d60073bb7210a29` | 脱敏：保留 85 个项目进程，去掉 787 个其他进程 |
| `main-sync-20261024/windows-evidence/isolation-before.json` | `34e5be424de6b513d12fc48ab3b29e4b3c63934b909c889c4aa4e54485082c84` | 脱敏：保留 85 个项目进程，去掉 787 个其他进程 |
| `main-sync-20261024/windows-evidence/transport-after.json` | `bf2dd8ecf32aaf33433eedb48b7997d7fe912487c58173b451c24182df443c42` | 脱敏：保留 85 个项目进程，去掉 787 个其他进程 |
| `main-sync-20261024/windows-evidence/transport-before.json` | `b47819197f0e132d628515424058c5ea0ebf6e3d89e40b9c4d7f51baed878418` | 脱敏：保留 85 个项目进程，去掉 787 个其他进程 |
| `main-sync-20261024/windows-evidence-original.zip` | `c3aa271e56e25511e9e53ef7be44497f6f922935d33614578127aafaa061ecb8` | 已移出主仓库（原始归档） |

原件位置：`nanake-lab/nemu-evidence` 仓库的 `P5-04/` 下，路径与上表相同。主仓库 git 历史中的旧版本未改写。
