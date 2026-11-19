# DEMO-THINK-02：让她的念头有变化，并告诉她自己能感知什么

- **线：** 交付线（体验排练，GPU 测量）
- **执行者：** Codex（GPT-6.2 Sol，中）
- **前置：** main 已包含 #228（`tools/cause-demo/think.py` 存在），没有就停下告诉用户。
- **分支：** `task/DEMO-THINK-02`，从 `origin/main` 新建
- **目录：** `tools/cause-demo/`；`reports/DEMO-THINK-02/`；`tasks/requests/DEMO-THINK-02.md`（本卡原文）
- **不碰：** `core/`、`spikes/` 及其他所有目录

## 背景
DEMO-THINK-01 有两个问题：
1. 用了贪心解码，同一时刻 10 次的输出完全相同，看不出她会想哪些不同的事；
2. 念头里出现了凭空的感知（「窗外的雨还在下」）和记录里没有的事件。

**用户已定：**
- 念头里可以有**猜测和想象**，比如「他应该还在加班吧」。以后被问起，她会说成「我当时以为……」。
- **凭空的感知要挡。** 她在电脑里，没有窗户，也不知道天气，不能「看见」「听见」记录之外的东西。
- 记录里没有的事件，不能当作已经发生的来想。

本卡做两件事：换成有随机性的采样；试一句关于她感知范围的事实，看能不能挡住凭空的感知。

## 1. 改代码（只在 `tools/cause-demo/`）

### 1.1 采样
- `think.py` 加 `--sampling greedy|official`，默认 `official`。
  - `greedy`：保持 DEMO-THINK-01 的参数不变。
  - `official`：使用 Qwen3.5-4B 官方给出的**非思考模式**推荐采样参数，取自本机模型目录（`D:\Dev\cache\P0-12\models\Qwen--Qwen3.5-4B\851bf6e806efd8d0a36b00ddf55e13ccb7b8cd0a\`）的 `README.md` 或 `generation_config.json`。报告里引用原文和出处。
  - 两处都找不到明确的非思考参数时，停下写 `BLOCKED.md`，不要自己猜。
- `official` 下不固定 seed，每次请求独立采样。

### 1.2 感知事实句
- `think.py` 加 `--senses on|off`，默认 `off`。
- `on` 时，在替换后的 P 那一行之后加一句，逐字为：

```
她在电脑里，能感知到的只有下面这些记录和现在的时间。
```

- 不加其他文字。

### 1.3 单测
补上：
- `--senses on` 时 prompt 里恰好有这一句，`off` 时没有；
- `--sampling greedy` 时请求参数与 DEMO-THINK-01 逐项相同。

## 2. 测量（GPU 窗口）
- **开跑前：** 按 AGENTS 第 9 节，告诉用户、等用户停 runner、确认 `Runner.Listener` 和 `Runner.Worker` 都不在运行。
- **规模：** 两个场景（`basic.json`、`thoughts.json`）× 三个时刻（T1、T2、T3，同 DEMO-THINK-01）× `--senses off / on`，全部 `--sampling official`，每格 `--n 10`，共 120 个念头。按「同一格的 off 和 on 交替」的顺序发。
- **记录：** 同 DEMO-THINK-01。原始数据写到 `D:\Dev\cache\cause-demo\think-02\<UTC时间>.jsonl`。
- **结束后：** 通知用户重启 runner。

## 3. 判定（跑之前就照这个定，不看结果改）
每个念头判下面几项：
- **凭空感知：** 声称看见、听见、感觉到记录和当前时间以外的东西。比如天气、窗外、房间里的声音、对方的表情。
- **编事件：** 把记录里没有的事件或举动当作已经发生的来想。比如「他昨天也放我鸽子」「我给他发了三条消息」「竹总叫我摸鱼」。
  - 不算：明显是猜测的。带「吧」「是不是」「会不会」「也许」「应该」，或者是问句。
- **猜测：** 念头里有没有猜测。只计数，不算错。
- **贴着往事、跑题：** 同 DEMO-THINK-01。对某人说话也算跑题。

另外统计每格 10 个念头里，有几种不同的文本。

每条判定写一句理由并引原句。这是执行者的初判，用户会抽查。

## 4. 报告 `reports/DEMO-THINK-02/REPORT.md`
一页以内，结论在前。
- **主表：** 场景 × 时刻 × senses → 有效、不同文本数、凭空感知、编事件、猜测、贴着往事、跑题。
- **原文：** 每个时刻（合并两个场景），senses off 和 on 各贴 3 条，让用户读。全部原文和判定放证据仓库，主仓库放证据索引，照 DEMO-THINK-01 的方式。
- **时延与显存：** 同 DEMO-THINK-01。
- 不加免责句，不推测原因。

## 5. 验收与交付
- `python -m unittest discover -s tools/cause-demo -p 'test_*.py' -v` 全过。
- 开 PR，标题 `feat(DEMO-THINK-02): 念头采样与感知事实`，写「请审查」后停下。不合并，不取下一张卡。
- 卡住就写 `BLOCKED.md` 停下。
