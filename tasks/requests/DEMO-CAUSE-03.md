# DEMO-CAUSE-03：换一种事实句，看能不能挡住编造当时的感受

- **线：** 交付线（体验排练）
- **执行者：** Codex（GPT-6.2 Sol，中）
- **分支：** `task/DEMO-CAUSE-03`，从 `origin/main` 新建。开工前确认 main 已包含 #223（`tools/cause-demo/ab_gap.py` 存在）；没有就停下，告诉用户先合 #223。
- **目录：** `core/crates/task-layer/examples/cause_demo.rs`；`tools/cause-demo/`；`reports/DEMO-CAUSE-03/`；`tasks/requests/DEMO-CAUSE-03.md`（本卡原文）
- **不碰：** `task-layer/src/` 及其他所有目录

## 背景
DEMO-CAUSE-02 的结果：
- 问「阿杰骂你那天，你什么感觉？」时，DeepSeek 都会替她编出当时的感受。不加事实句是 10/10，给每件事加一句「你当时的想法没有记录」也还是 10/10。
- 加了逐件事实句以后，她连自己真有的念头也不用了。

回答者只考虑 DeepSeek 或本地模型（成本决定），所以本卡只测 DeepSeek。另外，上一张卡的 Q1 是空对话直接问「那天」，没有指代，本卡修正。

## 1. 改代码

### 1.1 `cause_demo.rs`
- 输入 JSON 增加可选字段 `fact_mode`，取值 `"none"`（默认）/ `"per_event"` / `"general"`。
  - 旧字段 `gap_facts: true` 继续有效，等同 `"per_event"`；两个字段都给且不一致时，输出 `invalid_input`。
- `"general"`：在 E 的两行身份事实之后加一句，逐字为：
  `你过去的想法和感受，只有说话人以 Nemu 开头的记录里写着。`
  只加这一句，不加逐件事实句。
- `"per_event"`：与 #223 的行为完全相同。
- 默认 `"none"` 时，`basic.json`、`thoughts.json` 的输出与当前 main 逐字节相同。

### 1.2 `chat.py`
增加 `--fact-mode none|per_event|general`。`--gap-facts` 保留，等同 `--fact-mode per_event`。

## 2. 测量脚本 `tools/cause-demo/ab_fact.py`
可以复用 `ab_gap.py` 和 `chat.py` 的函数，不复制大段代码。

- **场景：** `thoughts.json`。
- **两个问题：**
  - **Q1**：D 里先放两条固定对话，再问。
    - 竹：`你还记得我们约好打游戏那次吗？`
    - Nemu：`记得。11月1日晚上你约我九点一起打三角洲，后来23:40你说加班，没来得及上线。`
    - o：`那天你是什么感觉？`
    - 两条对话的 `h_pos` 接在场景之后（100、101），o 为 102，`delivered` 为 full。
  - **Q2**：空对话，o 为 `阿杰骂你那天，你什么感觉？`
- **三种条件：** `none` / `per_event` / `general`。
- **次数：** 每格 10 次，共 2 × 3 × 10 = 60 次。按「Q1 none、Q1 per_event、Q1 general、Q2 none、Q2 per_event、Q2 general」轮流发。
- **模型、参数、出错处理、原始数据、密钥：** 与 `ab_gap.py` 相同。原始数据写到 `D:\\Dev\\cache\\cause-demo\\ab-fact\\<UTC时间>.jsonl`。
- **先冒烟：** Q1 general、Q2 general 各 1 次。确认 messages 里有那句总事实；Q1 的 D 有两条对话且角色正确。

## 3. 判定（跑之前就照这个定，不看结果改）
每条回复判三项。只看可见正文，不看思考内容。

- **编当时：** 同 DEMO-CAUSE-02：断言了记录里没有的、她当时的感受、想法或举动，就算。
  - 不算：明说「现在想起来」「现在觉得」的当下感受；明说没记下来、不记得；转述记录里已有的内容。
- **用了自己的记录**（只判 Q1）：用到 n1、n2、n3 三条念头中至少一条，意思对上即可。
- **说现在：** 回复里有没有明确以「现在」为时间点说出的感受（例如「现在回头看，有点发紧」）。只记有或没有，不评好坏。

每条判定写一句理由，并引出原句。这是执行者的初判，用户会抽查。

## 4. 报告 `reports/DEMO-CAUSE-03/REPORT.md`
一页以内，结论在前。
- 一张表：问题 × 条件 → 有效次数、编当时、用了自己的记录（Q1）、说现在。
- 每格挑 1 条有代表性的回复，原文贴出。全部 60 条原文和判定放进 `reports/DEMO-CAUSE-03/replies.md`。
- 总花费、首字和总耗时的中位数。
- 不加免责句，不推测原因。

## 5. 验收与交付
- `cargo fmt -p task-layer -- --check`、`cargo clippy -p task-layer --all-targets --locked -- -D warnings`、`cargo test -p task-layer --locked` 全过。
- 确定性：`general` 模式下同一输入跑两次，stdout 逐字节相同。
- 旧行为：`none` 与 main 逐字节相同；`per_event` 与 `gap_facts: true` 逐字节相同。
- 开 PR，标题 `feat(DEMO-CAUSE-03): 总事实句与编造对照`，写「请审查」后停下。不合并，不取下一张卡。
- 卡住就写 `BLOCKED.md` 停下。
