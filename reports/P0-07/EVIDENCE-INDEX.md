# P0-07 证据索引

按 AGENTS.md 第 6 节「证据放哪里、记什么」（2026-11-10 用户决定），下列原始证据已移到私有证据仓库 `nanake-lab/nemu-evidence`（提交 `50f5ea69cc4d13fb93a579e458eda83346e4ab04`），内容逐字节未改。主仓库里同名文件是脱敏版：只去掉了用户电脑上其他程序的名称与路径，项目自己的进程（electron、node、chrome-headless-shell、Runner.Listener、Runner.Worker）和端口等其余字段原样保留。

| 文件（相对本目录） | 原件 SHA-256 | 主仓库中的处理 |
| --- | --- | --- |
| `direction-v1/Qwen--Qwen3-4B-Instruct-2507.meta.log` | `adab8f6188ebabfd9378c29dfde5d4aa5fafb04158138fbbe87d885df822e5f5` | 脱敏：一行进程名改为「4 类共 23 个」 |
| `direction-v1/Qwen--Qwen3-4B.meta.log` | `c4daa84c075c5466a0c48656f389dc1eac13f3ab7ae7047c72dd3eb534102daa` | 脱敏：一行进程名改为「4 类共 23 个」 |
| `direction-v1/microsoft--Phi-4-mini-instruct.meta.log` | `e989ec8d7bc96943155603ab0eb08773b4efe1cada439922f5aaf837b9dfeacc` | 脱敏：一行进程名改为「4 类共 23 个」 |

原件位置：`nanake-lab/nemu-evidence` 仓库的 `P0-07/` 下，路径与上表相同。主仓库 git 历史中的旧版本未改写。
