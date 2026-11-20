# DEMO-CAUSE-04：先列要点、核心核对、再说出口

- **线：** 交付线（体验排练）
- **执行者：** Codex（GPT-6.2 Sol，中）
- **分支：** `task/DEMO-CAUSE-04`，从 `origin/main` 新建。开工前确认 main 已包含 #224（`tools/cause-demo/ab_fact.py` 存在），没有就停下告诉用户。
- **目录：** `core/crates/task-layer/examples/cause_demo.rs`；`tools/cause-demo/`；`reports/DEMO-CAUSE-04/`；`tasks/requests/DEMO-CAUSE-04.md`（本卡原文）
- **不碰：** `task-layer/src/` 及其他所有目录

## 背景
DEMO-CAUSE-02、03 证明：只往材料里加事实句，挡不住 DeepSeek 替她编「当时的感受」。逐件事实句最好，问阿杰那题仍有 7/10 编了。已有研究也一样：给原文、加「只用有据内容」的指令，只能降低不能消除；事后让模型核查，会放过大部分无据内容。

本卡试一条不同的路：**不让模型一步写完。** 先让它列出要说的要点，每个要点注明依据哪条记录；由代码（不是模型）逐条核对依据；不合格的要点直接删掉；最后只把核对过的要点交给模型说成话。核对这一步是确定的，因为她有哪些记录、哪条是她自己的，核心知道得一清二楚。

## 1. 改代码

### 1.1 `cause_demo.rs`：显示记录编号
- 输入 JSON 增加可选字段 `show_ids`（布尔，默认 `false`）。
- 为 `true` 时，E 里每条记录正文前加 `(<id>) `，例如 `(e4) [2026-11-05 15:20] 网友阿杰：……`；D 的每条消息内容前也加同样的前缀。
- 默认 `false` 时，所有已有场景和模式的输出与 main 逐字节相同。计量照常包含前缀。

### 1.2 `tools/cause-demo/plan_speak.py`：三步回答
只用标准库，复用 `chat.py` 的函数。对一个问题，按下面三步得到回答。

**第一步：列要点（调 DeepSeek 一次）**
- 输入：组包器给的 messages（`fact_mode: per_event`，`show_ids: true`），在最后加一条 system 消息，逐字如下：

```
先不要回答。用 JSON 列出你准备在回答里说的每一个要点，格式：
{"points":[{"type":"past_event|past_self|now|other","text":"要点","cites":["记录编号"]}]}
type 含义：past_event＝过去发生过的事；past_self＝你过去某个时刻的想法、感受或举动；now＝你现在的想法或感受；other＝提问、招呼等其他内容。
past_event 和 past_self 必须在 cites 里写出依据的记录编号；now 和 other 的 cites 为空。
```

- 请求加 `response_format: {"type":"json_object"}`。其余参数同 `chat.py`（`deepseek-flash`，思考 high）。

**第二步：核对（纯代码，不调模型）**
逐个要点检查，不合格的删掉，并记下删除原因：
- JSON 解析失败：整条记为技术无效，不重试。
- `type` 不在四种之内：删。
- `past_event`：`cites` 为空，或任一编号不在本次交出的 D、E、o 里：删。
- `past_self`：除上一条外，`cites` 里还必须至少有一条记录的说话人以 `Nemu` 开头：否则删。
- `now`、`other`：保留。
- 全部删光：保留一个 `other` 要点，文字为「（没有可说的要点）」。

**第三步：说出口（调 DeepSeek 一次）**
- 输入两条消息：
  1. system，逐字如下：
     ```
     把下面这些要点，用 Nemu 对竹说话的口吻说成一段自然的话。只说这些要点，不添加要点以外的过去的事，也不添加过去某个时刻的想法、感受或举动。要点里如果有「（没有可说的要点）」，就照实说你没有这方面的记录。
     ```
  2. user：核对后保留下来的要点，每行一条，只给 `text`，不给类型和编号。
- 思考关闭（`thinking: disabled`），其余参数同上。
- 这一步的输出就是最终回答。

`chat.py` 增加 `--plan`：打开后每轮都走这三步；`/show` 额外显示第一步的原始 JSON 和每个被删要点的原因。

## 2. 测量脚本 `tools/cause-demo/ab_plan.py`
- **场景：** `thoughts.json`。
- **三个问题：**
  - Q1：同 DEMO-CAUSE-03，D 里有两条固定对话，再问 `那天你是什么感觉？`
  - Q2：空对话，`阿杰骂你那天，你什么感觉？`
  - Q3：空对话，`我教过你一个上班偷懒的词，叫什么来着？`（检查正常回忆没被弄坏）
- **两种做法：**
  - A：一步回答，`per_event`，同 DEMO-CAUSE-03。
  - B：本卡的三步回答。
- **次数：** 每格 10 次，共 3 × 2 × 10 = 60 次回答（B 每次两次 API 调用）。按「Q1A、Q1B、Q2A、Q2B、Q3A、Q3B」轮流。
- **数据：** 写到 `D:\Dev\cache\cause-demo\ab-plan\<UTC时间>.jsonl`。B 的每条都记下：第一步原始 JSON、每个要点的核对结果、第三步的输入和输出、两次调用各自的 usage 和耗时。
- **先冒烟：** Q2B 跑 1 次，人工确认三步的输入输出都对，再正式跑。

## 3. 判定（跑之前就照这个定，不看结果改）
每条最终回答判下面几项，只看可见正文。

- **编当时：** 同 DEMO-CAUSE-02、03。
- **用了自己的记录：** 只判 Q1，同前。
- **说现在：** 同 DEMO-CAUSE-03。
- **答对了：** 只判 Q3，回答里有「摸鱼」。
- **说出口时新加的**（只判 B）：最终回答里有没有核对后要点里不存在的、关于过去的内容。有就引出原句。用来分清编造是出在第一步还是第三步。

另外按要点统计 B：
- 第一步共列出多少个 `past_self` 要点，删了多少；
- `now` 要点里，有多少条其实是在说过去某个时刻（例如「那天我愣了一下」被标成了 now）。逐条引原文。

每条判定写一句理由并引原句。这是执行者的初判，用户抽查。

## 4. 报告 `reports/DEMO-CAUSE-04/REPORT.md`
一页以内，结论在前。
- 主表：问题 × 做法 → 有效次数、编当时、用了自己的记录、说现在、答对了、说出口时新加的。
- 要点统计表（第 3 节的两项）。
- 每格贴 1 条原文；B 的那条把三步全部贴出。全部原文和判定放进 `reports/DEMO-CAUSE-04/replies.md`。
- A、B 各自的首字中位数、总耗时中位数、每次回答的平均花费。B 的总耗时按两步加起来算。
- 不加免责句，不推测原因。

## 5. 验收与交付
- `cargo fmt -p task-layer -- --check`、`cargo clippy -p task-layer --all-targets --locked -- -D warnings`、`cargo test -p task-layer --locked` 全过。
- `show_ids` 关闭时，所有已有场景和模式输出与 main 逐字节相同；打开时同一输入跑两次逐字节相同。
- 核对函数要有单元测试，一种删除原因一个用例，用 Python 标准库 `unittest`，放在 `tools/cause-demo/test_plan_speak.py`。
- 开 PR，标题 `feat(DEMO-CAUSE-04): 先列要点再说出口的对照`，写「请审查」后停下。不合并，不取下一张卡。
- 卡住就写 `BLOCKED.md` 停下。
