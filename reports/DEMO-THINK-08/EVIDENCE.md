# DEMO-THINK-08 证据索引

私有证据仓库 `nanake-lab/nemu-evidence` 提交 `d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724`。压缩包内均附逐文件 `sha256.json`。

| 文件 | 内容 | SHA-256 |
| --- | --- | --- |
| [blind-prep.zip](https://github.com/nanake-lab/nemu-evidence/blob/d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724/DEMO-THINK-08/blind-prep.zip) | 历史 539 条去标签输入与映射表、`own_thoughts` 重建表、新盲测集 180 条无核对器原始连写及去标签输入/映射表 | `05369a54cb85a8a853afc9c76090d3338e4492b4c1c800aa683b0c6a1943229c` |
| [secondary.zip](https://github.com/nanake-lab/nemu-evidence/blob/d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724/DEMO-THINK-08/secondary.zip) | 新盲测集（180 条）与统一历史集（539 条）上 `v1`/`v2-off`/`v3-off`/`v3-high` 单次运行全部逐请求记录与汇总 | `ec2006b0164b4d5315c9376e297afed8ee9f0d51f9c4717d62c9ac2dbd4706a7` |
| [append.zip](https://github.com/nanake-lab/nemu-evidence/blob/d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724/DEMO-THINK-08/append.zip) | 4 组 `v3` 连写（360 轮）全部逐请求记录、终态场景 JSON、费用时延汇总、360 轮判定 JSON 与 20 条抽样原文 JSON | `3d3bf20f2871ebc24dbaf4b6c2e664b4efd6813b51a3f8e333aedd0bfb9f44ff` |
| [append-judgments.md](https://github.com/nanake-lab/nemu-evidence/blob/d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724/DEMO-THINK-08/append-judgments.md) | 4 组连写 360 轮原文、分段汇总与逐轮判定明细 | `a06321d925ae549ad503b5cd1e1e07814889f37d41efb5f64f02e4792a40ecec` |
| [sample-review.md](https://github.com/nanake-lab/nemu-evidence/blob/d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724/DEMO-THINK-08/sample-review.md) | 连写随机抽查 20 条（种子 `20261121`：10 拦下 + 10 放行）原文、核对器输出与判定 | `7773c4b13c93b8bcb76913c41340e4d2757d213862f9c630e8abde07f35664dd` |
| [audit.zip](https://github.com/nanake-lab/nemu-evidence/blob/d8e91f0ba6e86c3d67e282f8d8fabe9d3740a724/DEMO-THINK-08/audit.zip) | 数据提取/评测/打包脚本，以及 45 项 `unittest`、`ruff`、`mypy --strict` 原始输出 | `e4097936ffe3dba02a04fcf444cf2bad6f2bd02334ff6ae135de37673fe6fc5e` |

## 主仓冻结清单与运行基线

- 历史盲标冻结（提交 `92cb7405`）：[blind-labels.json](blind-labels.json)（`ad4ce7d17ecf7701d01eaccff0b7e7085a0061c47487fd04a56e6c96af4205a5`）
- 统一历史金标准冻结（提交 `e27ad051`）：[gold-historical.json](gold-historical.json)（`183538fc2bb6d4ca0a0a93f8e4463d24ebf7db053bf3d717b2e63029a235a0ad`），逐条调整见 [RELABEL-DIFF.md](RELABEL-DIFF.md)
- 新盲测集冻结（提交 `3167670a`）：[new-blind-labels.json](new-blind-labels.json)（`e51b8c700325baee03da7a14127681781cf30067c412fdaa64e45c70cb477208`）
- 历史 539 条核对运行提交：`6b2d5c6b`（UTC `2026-11-21T16:54:00Z` – `17:14:34Z`）
- 新盲测 180 条核对运行提交：`3167670a`（UTC `2026-11-21T17:03:43Z` – `17:10:15Z`；与 `6b2d5c6b` 的 `tools/cause-demo/` 代码一致）
- 4 组连写运行提交：`3167670a`（UTC `2026-11-21T17:00:07Z` – `18:01:04Z`）
