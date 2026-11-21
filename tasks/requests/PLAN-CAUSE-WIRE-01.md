# PLAN-CAUSE-WIRE-01：把「嘴」和「心」从演示接到 Echo 核心上——盘点与接线计划

- **线：** 交付线（只读规划）
- **执行者：** Codex，**6.2 Astra，高**
- **分支：** `task/PLAN-CAUSE-WIRE-01`，从 `origin/main` 新建
- **目录：** `reports/PLAN-CAUSE-WIRE-01/`；`tasks/requests/PLAN-CAUSE-WIRE-01.md`（本卡原文）
- **只读：** 不改代码、契约、ADR、规格、`tasks/cards.json` 或任何已有卡片文字。需要改的地方写成「候选改动」，交给用户决定。

## 背景
这几天在 `tools/cause-demo/` 里做了两件事，都只是演示，没有接到核心上：
- **嘴：** 组包器（`task-layer/src/cause_projection/`，ADR 0042）按分量挑往事原话，交给 DeepSeek 说话。实测「先列要点 → 代码核对 → 拼接」的两步法（DEMO-CAUSE-05）效果最好。用户定了两点：
  - 她引用自己的念头后，润色的严格程度做成用户可选的档位，默认放宽；
  - 凡是有选择的地方，都做成用户可选，默认值由用户定。
- **心：** 她独处时读组包结果，写下一个念头，念头再作为她自己的原话写回历史（DEMO-THINK-01～03）。本地 Qwen3.5-4B 未训练时编造多。**用户已决定：** 先用 DeepSeek（不思考）来想，同时研究线训练本地 4B，以后换回本地。

演示里的往事是手写的，分量 h 是写死的数，念头直接追加进 JSON。本卡要回答：接到真实核心上，现在有什么、缺什么、按什么顺序做。

## 先读（按顺序）
1. `AGENTS.md`、`.agents/skills/echo-core-work/SKILL.md` 第 1 节。
2. `docs/adr/0042-cause-projection.md` 全文；`docs/adr/0043-two-lines.md`；`docs/adr/0026-continuous-cognition.md`。
3. **契约：**
   - `core/contracts/records.proto`、`state.proto`、`terminal.proto`、`terminal_ingress.proto`：H 记录、`source_refs`、`correction_of`、`correction_chain`、撤回等与「关系」有关的字段；
   - `law_state.proto`：基底槽、h；
   - `cognition.proto`：CognitiveInput、InternalActivityRecord。
4. **代码：**
   - `core/crates/task-layer/src/cause_projection/` 全部；
   - `tools/cause-demo/` 全部（`cause_demo.rs` 的 DemoCodec、`plan_speak.py`、`think.py`）；
   - `core-store` 里 H 的写入与查询入口（只看接口，不必读完实现）。
5. **卡：** `tasks/P6-presence.md` 的 P6-01、P6-04；`tasks/P5-echo-core.md` 的 P5-13、P5-14、P5-37～P5-40；`tasks/cards.json` 里这些卡的依赖。
6. **报告：** `reports/` 下 DEMO-CAUSE-01～05、DEMO-THINK-01～03 的 REPORT.md。

## 要回答的问题

### Q1 组包器的每个输入，在真实核心里从哪来
对 `AssemblyInput` 的每个字段写一行：
- `records`（含 id、content_version、h_pos、body、eligibility）
- `relations`
- `frontier_complete`
- `slots`（basis、h、first_f_pos、members、sources、merge_undecided）
- `decomposition_unknown`
- `dialogue`（含 delivered）
- `latest`
- `required_facts`
- `protocol_cited`
- `budget`

每行给出：真实来源（契约消息与字段、crate 与函数，带文件和行号）；现状（已有 / 部分有 / 没有）；缺的话缺什么。

### Q2 关系类型映射
ADR 0042 要求运行前发布「关系类型映射」资源。
1. 列出契约和存储里所有能表示「记录之间关系」的字段，包括引用、来源、更正、撤回、回复、合并。
2. 逐个对到组包器的五类：BodySource、Continuation、Correction、LocalOnly、Unknown。对不上的，写明原因。
3. 列出组包器需要、但核心里还没有的关系。比如「回复上一条」在终端接入里有没有存。

### Q3 念头怎么写回历史
1. 独处念头写回 H，应走哪个契约消息（InternalActivityRecord 还是别的）、由哪张卡负责。
2. 写回后，它的说话人身份、它属于哪个槽（演示里是简化为「本次排第一的槽」，真实系统由核心决定）、资格，各由核心哪一部分给出。
3. 「她在组包里能被认出是自己的记录」需要什么字段。

### Q4 说话这条路：P6-04 与现状的差距
P6-04（模式 B）写在 ADR 0042 之前。逐条对比 P6-04 的卡片文字与 ADR 0042 及演示结论：两步法、`fact_mode per_event`、`self_quote` 档位、快慢分路的结论（闲聊也可能触发往事）。列出哪些文字需要改，写成「候选改动」，原文和建议新文字并列。

### Q5 正式的编码与计量
DemoCodec 是演示用的。列出正式版需要什么：
- 发给 DeepSeek 的消息格式；
- 两步法的 JSON 要点与核对；
- 按 DeepSeek 实际 tokenizer 计量，还是保守估计；
- 这些资源归哪张卡。

### Q6 与已有决定的偏离
用户的「先用 DeepSeek 来想」与 ADR 0042 U5（本地模型负责独处思考）不一致。列出这类偏离：原文、新决定、需要登记到哪里（ADR 修订还是差异登记）。不要替用户写决定，只列出来。

## 交付 `reports/PLAN-CAUSE-WIRE-01/REPORT.md`
- **开头白话三段：** 现在离「她真的用核心说话、想事」还差什么；最短的路线；需要用户决定的事。
- **Q1～Q6 各一节。** 表格为主，每个结论都带文件和行号。
- **接线卡清单：** 按先后顺序列出。每张写：目标（一句）、目录、依赖、大致工作量、建议执行者与模型档位、验收的一句话（写明用户最终能体验到什么）。能并行的标出来。
- **「需用户决定」一节：** 每项写清候选选项和各自后果。
- **不写：** 工作过程，以及对 ADR、卡片原文的大段复述。

开 draft PR，标题 `docs(PLAN-CAUSE-WIRE-01): 嘴与心接到 Echo 核心的盘点与计划`，写「请审查」后停下。不合并，不取下一张卡。
