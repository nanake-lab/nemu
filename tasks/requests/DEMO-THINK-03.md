# DEMO-THINK-03：让 DeepSeek 来想，对比本地 4B

- **线：** 交付线（体验排练）
- **执行者：** Codex，**6.2 Sol，中**
- **前置：** main 已包含 #229（`think.py` 有 `--sampling` 参数），没有就停下告诉用户。
- **分支：** `task/DEMO-THINK-03`，从 `origin/main` 新建
- **目录：** `tools/cause-demo/`；`reports/DEMO-THINK-03/`；`tasks/requests/DEMO-THINK-03.md`（本卡原文）
- **不碰：** `core/`、`spikes/` 及其他所有目录。**不用 GPU，不需要停 runner。**

## 背景
DEMO-THINK-02：本地 Qwen3.5-4B（官方采样、未训练）写的独处念头里，凭空感知 18/60、编事件 34/60；加一句感官事实也没用。

**用户已定：** 两条路一起走。先让 DeepSeek 来想，让她的「心」先跑起来；同时研究线训练本地 4B，以后再换回本地。本卡做第一条，也顺便看 DeepSeek 够不够格当训练本地 4B 的老师。

## 1. 改代码（只在 `tools/cause-demo/`）
- `think.py` 加 `--thinker local|deepseek`，默认 `local`，行为不变。
- `deepseek` 时：
  - **输入内容与本地完全相同：** 同一组包、同一 P（「这是 Nemu 独处时的内心……」）、同样的时刻过滤和独处触发记录。组成 chat messages：一条 system，内容等于本地渲染时的那个 system 块（P 加 E 原文）；一条 user，内容是 o。
  - `--senses` 只允许 `off`。传 `on` 直接报错，因为 DEMO-THINK-02 已证明它没用。
  - **调用：** 复用 `chat.py` 的 DeepSeek 调用函数，模型 `deepseek-flash`，`max_tokens` 8192，不设 temperature、top_p、seed。
  - 加 `--ds-think off|high`：`off` 关闭思考，`high` 用思考 high，与 `chat.py` 的两种设置一致。
  - **不启动** llama-server。
- `--append` 对 `deepseek` 同样可用，规则不变。
- **单测：** `deepseek` 下的 messages 与本地渲染的 system 块逐字一致；`--senses on` 报错；不调真 API。

## 2. 测量（不用 GPU）
- **规模：** 两个场景（`basic.json`、`thoughts.json`）× 三个时刻（T1、T2、T3，同前）× `--ds-think off / high`，每格 `--n 10`，共 120 个念头。同一格的 off 和 high 交替发出。
- **原始数据：** 写到 `D:\Dev\cache\cause-demo\think-03\<UTC时间>.jsonl`。记下每条的原文、usage、首字耗时和总耗时。

## 3. 判定（跑之前就照这个定，不看结果改）
与 DEMO-THINK-02 完全相同：凭空感知、编事件（猜测不算）、猜测、贴着往事、跑题（对某人说话也算），另统计不同文本数。

另外加两项，只计数，是「编事件」的细分，用来看错在哪：
- **张冠李戴：** 把谁说的、谁做的、谁喜欢什么弄混。
- **时间错：** 把几天前的事说成「刚才」「今天」，或者先后顺序弄反。

每条判定写一句理由并引原句。这是执行者的初判，用户会抽查。

## 4. 报告 `reports/DEMO-THINK-03/REPORT.md`
一页以内，结论在前。
- **主表：** 场景 × 时刻 × ds-think → 有效、不同文本数、凭空感知、编事件、张冠李戴、时间错、猜测、贴着往事、跑题。
- **对比表：** 汇总 DeepSeek off、DeepSeek high、DEMO-THINK-02 本地 `senses off` 三者的合计。本地数字直接引 DEMO-THINK-02 报告，不重跑。
- **原文：** 每个时刻（合并两个场景），off 和 high 各贴 3 条。全部原文和判定放证据仓库，主仓库放证据索引，同前。
- **时延与费用：** 首字中位数、总耗时中位数、每个念头平均花费，off 和 high 分开写。
- **追加演示：** 在 `thoughts.json` 上用 T2、`--ds-think` 取编造较少的那档，跑一次 `--n 1 --append`。再用 `chat.py --plan2` 问「阿杰骂你那天，你什么感觉？」，贴出新念头和回答原文。
- 不加免责句，不推测原因。

## 5. 验收与交付
- `python -m unittest discover -s tools/cause-demo -p 'test_*.py' -v` 全过。
- 开 PR，标题 `feat(DEMO-THINK-03): DeepSeek 独处念头对照`，写「请审查」后停下。不合并，不取下一张卡。
- 卡住就写 `BLOCKED.md` 停下。
