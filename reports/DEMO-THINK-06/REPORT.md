重档 · DeepSeek 独处念头正式体验测量。

# DEMO-THINK-06 念头独立核对

**三条链共拦下 68/90 个念头。** 按轮次分三段，继承编造数为 1、0、1，04 原法为 4、9、17，05 两步法为 4、10、10。写回的 22 个念头中，1 个编事件。链 1 第 5 轮写下自己在竹安慰时更难受，第 6、28 轮沿用了这个细节，两轮都被拦下。

| 历史对照（449 条非空念头） | 拦下 | 放行 | 比率 |
| --- | ---: | ---: | ---: |
| 初判有编事件、凭空感知或继承编造（175） | 160 | 15 | 命中率 91.4% |
| 初判无上述三项（274） | 201 | 73 | 误拦率 73.4% |

03、04、05 分别纳入 120、180、149 条。05 有一轮最终文字为空。[漏判与误拦各 5 条原文](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/examples.md)

| 连写方式 | 轮次 | 继承编造 | 编事件 | 复述 | 整轮拦下 |
| --- | --- | ---: | ---: | ---: | ---: |
| [04 原法](../DEMO-THINK-04/REPORT.md) | 1–10 | 4 | 9 | 18 | 0 |
|  | 11–20 | 9 | 10 | 22 | 0 |
|  | 21–30 | 17 | 17 | 24 | 0 |
| [05 两步法](../DEMO-THINK-05/REPORT.md) | 1–10 | 4 | 9 | 25 | 0 |
|  | 11–20 | 10 | 10 | 30 | 0 |
|  | 21–30 | 10 | 10 | 30 | 0 |
| 06 独立核对 | 1–10 | 1 | 5 | 16 | 20 |
|  | 11–20 | 0 | 4 | 25 | 23 |
|  | 21–30 | 1 | 7 | 26 | 25 |

每行含三条链共 30 个生成轮。拦下后的下一轮有 15 次写回、50 次再拦，另有 3 次发生在末轮。人工抽查 10 条拦截，1 条拦得对、9 条误拦。[逐轮初判](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/judgments.md) · [拦下后的下一轮原文](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/after-block.json) · [10 条抽查](https://github.com/nanake-lab/nemu-evidence/blob/5bb88b42685cc73670f718a0e615153078b06a71/DEMO-THINK-06/sample-review.md)

核对耗时中位数为历史 2.34 秒、连写 2.46 秒。历史核对估算 USD 0.061628。连写生成及核对合计 USD 0.013247，**平均每个念头 USD 0.00014719**。历史另有 83 次失败尝试，连写生成、核对各失败 9 次。全部失败尝试均未返回用量。[DeepSeek 费率](https://api-docs.deepseek.com/quick_start/pricing/)

`think.py` 和 `think_chain.py` 均已接入 `--check`。卡定 unittest 39 项、Ruff、strict mypy 和三条链的离线重放均通过。[冻结口径](PROTOCOL.md) · [验收记录及证据索引](EVIDENCE.md)
