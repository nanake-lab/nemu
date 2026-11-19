# P4-01 证据索引

按 AGENTS.md 第 6 节「证据放哪里、记什么」（2026-11-10 用户决定），下列原始证据已移到私有证据仓库 `nanake-lab/nemu-evidence`（提交 `50f5ea69cc4d13fb93a579e458eda83346e4ab04`），内容逐字节未改。主仓库里同名文件是脱敏版：只去掉了用户电脑上其他程序的名称与路径，项目自己的进程（electron、node、chrome-headless-shell、Runner.Listener、Runner.Worker）和端口等其余字段原样保留。

| 文件（相对本目录） | 原件 SHA-256 | 主仓库中的处理 |
| --- | --- | --- |
| `adr0017-e2e/nemu-evidence/local-p401-adr0017/1/windows/e2e-after.json` | `618961c3867771b3d0401a3c51b5ecb88424fc265e1d17b6925ca40ffa2a6a5f` | 脱敏：保留 38 个项目进程，去掉 635 个其他进程 |
| `adr0017-e2e/nemu-evidence/local-p401-adr0017/1/windows/e2e-before.json` | `8a63e051cbd2d74a01ee9cf294d569ec8797bb9edb7b816fdef1add2fa00d522` | 脱敏：保留 38 个项目进程，去掉 637 个其他进程 |
| `adr0017-e2e/nemu-evidence/local-p401-adr0017/1/windows/electron-after-window-wait.json` | `0c5edebed6bb63323471f83a5830546432dcdc5df93101736151f883fcb411d9` | 脱敏：保留 54 个项目进程，去掉 631 个其他进程 |
| `adr0017-e2e/nemu-evidence/local-p401-adr0017/2/windows/e2e-after.json` | `b4fa6b20e85e72799903ccb180185bc2d8afea9e0be0d1173695654470456feb` | 脱敏：保留 38 个项目进程，去掉 637 个其他进程 |
| `adr0017-e2e/nemu-evidence/local-p401-adr0017/2/windows/e2e-before.json` | `a527e7a5abdfbba777b9d40701c765bb9a7294add1e061072ad4be3bc1a417cc` | 脱敏：保留 38 个项目进程，去掉 644 个其他进程 |
| `adr0017-e2e/nemu-evidence/local-p401-adr0017/2/windows/electron-after-window-wait.json` | `c01c5911ea354f2afd3264afca02061a1a51fd288fb2dc267fead293eb25806d` | 脱敏：保留 54 个项目进程，去掉 641 个其他进程 |
| `resume-e2e-20261021T233633/electron-after-window-wait.json` | `d0d1eb6e441007fb455be8d55950d9c23bd9098839a12378239b88173b45d94c` | 脱敏：保留 56 个项目进程，去掉 487 个其他进程 |

原件位置：`nanake-lab/nemu-evidence` 仓库的 `P4-01/` 下，路径与上表相同。主仓库 git 历史中的旧版本未改写。

## PRIVACY-CLEANUP-01（2026-11-11）

本次迁移原件的位置、SHA-256 与私有证据提交见 [逐项索引](../PRIVACY-CLEANUP-01/EVIDENCE-INDEX.md)。既有索引与原始证据保留。
