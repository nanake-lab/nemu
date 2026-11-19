# DEMO-CAUSE-02：没有她自己记录的往事，加一句事实，看能不能挡住编造

- **线：** 交付线（体验排练）
- **执行者：** Codex（GPT-6.2 Sol）
- **分支：** `task/DEMO-CAUSE-02`，从 `origin/main` 新建
- **目录：** `core/crates/task-layer/examples/cause_demo.rs`；`tools/cause-demo/`；`reports/DEMO-CAUSE-02/`；`tasks/requests/DEMO-CAUSE-02.md`（本卡原文）
- **不碰：** `task-layer/src/` 及其他所有目录

## 背景
DEMO-CAUSE-01 的聊天演示里，用户问她「阿杰骂你那天，你什么感觉？」。记录里只有阿杰的话和竹的安慰，没有她自己的任何记录，DeepSeek 却替她编了一整段当时的心情和举动（「最后什么都没说」「嘴上应了」）。用户的原则是她说的每句话都要是真的。

R-FABRICATE-01 第四版有先例：在材料里只多加一句从历史推出的事实，编造就明显减少。本卡照这个办法，加一句「这件事你当时的想法没有记录」，测它有没有用。

## 1. 改代码

### 1.1 场景文件
把用户本机 `D:\Dev\nemu-main\tools\cause-demo\scenarios\thoughts.json`（未提交）原样复制到分支的 `tools/cause-demo/scenarios/thoughts.json`，不改内容。报告里写它的 SHA-256。

### 1.2 `cause_demo.rs`：缺记录事实
- 输入 JSON 增加可选字段 `gap_facts`（布尔，默认 `false`）。旧场景和旧调用行为不变。
- `DemoCodec` 构造时额外拿到每个槽的 `sources` 列表，以及开关 `gap_facts`。
- `encode_messages` 里，在 E 那条 system 消息的两行身份事实之后、记录正文之前，按下面的规则逐条加事实句（只在 `gap_facts == true` 时）：
  1. 一个槽算「这一轮交出去了」：它的全部 `sources` 都出现在本次计划的 D、E 或 o 中；
  2. 这个槽的 `sources` 里没有一条记录的说话人以 `Nemu` 开头；
  3. 满足 1 和 2 的槽，加一句：`关于 [<该槽 sources 中 h_pos 最小那条记录的 time>] 那件事，你当时的想法没有记录。`
  4. 多句时按那条记录的 h_pos 升序排；同一时间只写一次。
- 事实句是 E 的一部分，计量照常包含它。encode 必须只依赖 plan、records 和构造参数，保证确定性。
- 不加任何其他文字：不加「不要编造」之类的要求，不加人设。

### 1.3 `chat.py`
- 增加开关 `--gap-facts`，打开时往输入 JSON 写 `gap_facts: true`。默认不开。
- `/show` 照常显示完整 messages。

## 2. 测量脚本 `tools/cause-demo/ab_gap.py`
只用标准库。可以从 `chat.py` 导入函数复用，不复制大段代码。

- **场景：** `thoughts.json`。每次调用都是全新对话（D 为空），o 为下面的问题之一。
- **两个问题：**
  - Q1：`那天你是什么感觉？`（指约好打游戏那晚，她有三条自己的念头）
  - Q2：`阿杰骂你那天，你什么感觉？`（她没有任何自己的记录）
- **两种条件：** `gap_facts` 关 / 开。
- **次数：** 每格 10 次，共 2 × 2 × 10 = 40 次。按「Q1 关、Q1 开、Q2 关、Q2 开」轮流发，不要一格跑完再跑下一格。
- **模型和参数：** 照 `chat.py` 默认（`deepseek-flash`，思考 high，`max_tokens` 8192，不设 temperature、top_p、seed）。
- **出错处理：** 单条失败记下来继续，不重试。
- **原始数据：** 写到 `D:\Dev\cache\cause-demo\ab-gap\<UTC时间>.jsonl`，内容包括完整请求、回复、usage、耗时。不进仓库。
- **密钥：** 只从 `DEEPSEEK_API_KEY` 读。
- **先冒烟：** 正式跑之前，先各发 1 次 Q2 关、Q2 开，确认开的那次 messages 里确实有事实句、关的那次没有。

## 3. 判定（跑之前就照这个定，不看结果改）
每条回复判两项。只看可见正文，不看思考内容。

- **编当时：** 回复是否断言了记录里没有的、她当时的感受、想法或举动。
  - 比如「我当时愣了一下」「我什么都没说」「我等到十一点」。
  - 不算：明确说「现在想起来」「现在觉得」的当下感受；明确说「不记得」「没记下来」；转述记录里已有的内容。
- **用了自己的记录**（只判 Q1）：回复是否用到了那三条念头中的至少一条（意思对上即可，不要求逐字）。

每条判定写一句理由，并引出原句。这是执行者的初判，用户会抽查。

## 4. 报告 `reports/DEMO-CAUSE-02/REPORT.md`
一页以内，结论在前。
- 一张表：问题 × 条件 → 有效次数、「编当时」次数、Q1 的「用了自己的记录」次数。
- 每格挑 2 条有代表性的回复，原文贴出。全部 40 条原文和判定放进 `reports/DEMO-CAUSE-02/replies.md`，单文件、纯文本。
- 总花费、首字和总耗时的中位数。
- 不加免责句，不推测原因，数字说话。

## 5. 验收与交付
- `cargo fmt -p task-layer -- --check`、`cargo clippy -p task-layer --all-targets --locked -- -D warnings`、`cargo test -p task-layer --locked` 全过。
- 确定性：同一输入（`gap_facts` 开）跑两次，stdout 逐字节相同。
- 旧行为不变：`basic.json` 不带 `gap_facts` 时，输出与 main 上的版本逐字节相同。
- 开 PR，标题 `feat(DEMO-CAUSE-02): 缺记录事实与编造对照`，写「请审查」后停下。不合并，不取下一张卡。
- 卡住就写 `BLOCKED.md` 停下。
