# DEMO-THINK-06 证据索引

私有证据仓库提交 `5bb88b42685cc73670f718a0e615153078b06a71`。压缩包内附逐文件 SHA-256。

| 文件 | 内容 | SHA-256 |
| --- | --- | --- |
| [history.zip](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/history.zip) | 449 条历史输入、初判映射、逐次 HTTP 结果及重算输入脚本 | `b44f329949fa9f982ca94bf651c4409fdacd1a6bf18ae715ef76270479383d7d` |
| [chains.zip](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/chains.zip) | 三条链的全部请求、响应、决定和最终场景 | `fac5af5a7d3bfa0690245fd8e5e7ab6a7a1e813992fa194957054e1999339ba7` |
| [audit.zip](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/audit.zip) | 逐条判定底稿、抽样、复算脚本、哈希重放及静态检查与单测输出 | `6bc44a8e846d0691ececbd5cd683805163fe0ea1acd14c9a63a1a66b1ab07bf5` |
| [examples.md](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/examples.md) | 历史漏判、误拦各 5 条完整原文及原初判 | `901fb2a87a390ad20d4deea2960d18eb97cb16d98486ebf610739ee55db6e139` |
| [judgments.md](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/judgments.md) | 90 条连写原文、执行者初判和同链对照轮号 | `e5fa48de64d87598e259a012f8b4336b811c471b125b74f84fd588da27512c05` |
| [after-block.json](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/after-block.json) | 每次拦下及同链下一轮原文和结果 | `651fdfef8c51fa5b4464e51b088bc76e603ad9b0185f1c25596a5c67224854c7` |
| [sample-review.md](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/sample-review.md) | 固定抽样的 10 条原文、人工结论和理由 | `623bfecea644e700ad3cd156a7400e167e7ed4ce2baa3f07eb201bfd48c3e57b` |

三条链的命令（在主仓库本卡工作区运行）

```powershell
python tools/cause-demo/think_chain.py --scenario tools/cause-demo/scenarios/thoughts.json --start "2026-11-05 15:25" --step-min 10 --rounds 30 --mode append --chains 3 --thinker deepseek --ds-think off --check
```

离线重放、抽样与统计的输入和脚本见 audit.zip、history.zip 及 chains.zip。
