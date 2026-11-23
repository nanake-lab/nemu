# DEMO-THINK-07 证据索引

私有证据仓库提交 `0e3022c059ae8475dd90cfdbde49a22b173b0353`。压缩包内附逐文件 `sha256.json`。

| 文件 | 内容 | SHA-256 |
| --- | --- | --- |
| [split.zip](https://github.com/nanake-lab/nemu-evidence/blob/0e3022c059ae8475dd90cfdbde49a22b173b0353/DEMO-THINK-07/split.zip) | 539 条全集、种子 7 开发集（333）/ 测试集（206）完整输入、划分脚本与汇总 | `721591d5dc1fddb9c70eb8730ca8d87e54edbaae7d253edb0fa323b1d7ff5d35` |
| [dev.zip](https://github.com/nanake-lab/nemu-evidence/blob/0e3022c059ae8475dd90cfdbde49a22b173b0353/DEMO-THINK-07/dev.zip) | 开发集基线 `v1-off` 及 5 轮 v2 调试（`d1-off`/`d1-high`/`d2-off`/`d3-off`/`d4-off`/`d4-high`/`d5-off`）全部逐请求记录 | `841517506a1c2b42ea6f25c5177e6f4f42ecb34b363c8e55be217cfae849f311` |
| [test.zip](https://github.com/nanake-lab/nemu-evidence/blob/0e3022c059ae8475dd90cfdbde49a22b173b0353/DEMO-THINK-07/test.zip) | 提交 `734b180afc609dc98d927ba01d908c19d1a36a49` 单次测试集运行（`v1-off`、`v2-off`、`v2-high`）全部逐请求记录（含失败尝试） | `12afb7cea8ef50a24831c00aad1ffc3d4fb5356a81a8934101af5c2359a0ca2c` |
| [audit.zip](https://github.com/nanake-lab/nemu-evidence/blob/0e3022c059ae8475dd90cfdbde49a22b173b0353/DEMO-THINK-07/audit.zip) | 评测脚本 `eval_split.py`、开发/测试集汇总 `eval-summary.json`、42 项单测与 Ruff/mypy 输出 | `b3e82fe8dd6a2781e9f956369f5280731ebfff807129d043fa04d82d4c2d03aa` |
| [test-fn-breakdown.md](https://github.com/nanake-lab/nemu-evidence/blob/0e3022c059ae8475dd90cfdbde49a22b173b0353/DEMO-THINK-07/test-fn-breakdown.md) | 测试集按卡/链分组命中率与误拦率，以及 `v2-high`、`v2-off` 全部漏判与未计分原文 | `a8757e8a6d04f1b25cf83f07ff0b0b0df1bd8b0725df8cd2e00cfb9c71904e6a` |

测试集运行提交：`734b180afc609dc98d927ba01d908c19d1a36a49`（UTC `2026-11-21T10:01:48Z` – `2026-11-21T10:12:27Z`）。开发/测试清单 SHA-256 见 [PROTOCOL.md](PROTOCOL.md)。
