# DEMO-CAUSE-01：用组包器和 DeepSeek 跟「拿到原话的她」聊天

- **线：** 交付线（体验排练，不是她的诞生，也不是产品接线）
- **建议执行者：** Antigravity Orchestrator (Carbon)
- **分支：** `task/DEMO-CAUSE-01`，从 `origin/main` 新建
- **目录：** `tools/cause-demo/`（新建）；`core/crates/task-layer/examples/cause_demo.rs`（新建）；`core/crates/task-layer/Cargo.toml` 只加 `[dev-dependencies]`；`core/Cargo.lock` 随之变化；`tasks/requests/DEMO-CAUSE-01.md`（本卡原文）；`reports/DEMO-CAUSE-01/`
- **不碰：** `task-layer/src/` 下任何文件、其他 crate、契约、ADR

## 1. 要做什么

一个本机命令行小工具：用户在终端里打字，跟「她」聊天。每一轮：

1. 把编好的往事（场景文件）、到目前为止的对话、用户刚输入的这句，交给已合入的 `task_layer::cause_projection::assemble`；
2. 组包器挑出往事，排成 P → D → E → o；
3. 发给 DeepSeek，把回复打印出来，并把这条回复记进对话。

目的只有一个：让用户亲身感受「按 ADR 0042 交原话」时她说话是什么样。往事是编的，h 是场景文件里写死的数，没有核心状态在变化。

## 2. 先读
1. `AGENTS.md`。
2. `core/crates/task-layer/src/cause_projection/`：`mod.rs`、`types.rs`，知道输入输出长什么样。
3. `reports/R-FABRICATE-01/PLAN.md`、`V2-PLAN.md`、`V3-PLAN.md`：协议原文和 DeepSeek 调用参数从这里来。
4. 私有证据仓 `nanake-lab/nemu-evidence` 下 R-FABRICATE-01 的调用器（报告里叫 `code-reviewed/run.py`）：照它的 DeepSeek 请求写法，不要自己猜接口参数。

## 3. 组成

### 3.1 Rust 示例 `core/crates/task-layer/examples/cause_demo.rs`
- `Cargo.toml` 加 `[dev-dependencies]`：`serde = { version = "=1.0.229", features = ["derive"] }`、`serde_json = "=1.0.154"`（与其他 crate 相同的版本，不引入新包）。
- 从 stdin 读一个 JSON（格式见 3.3），在示例文件里用带 serde 的镜像结构解析后转成 `AssemblyInput`，调用 `assemble`，往 stdout 写一个 JSON：
  - `outcome`：`sent` / `projection_unavailable` / `base_request_failed` / `invalid_input`，加原因；
  - `messages`：发给 API 的消息数组（只在 `sent` 时有）；
  - `binding`：每个候选包的 basis 和结局（接纳 / 溢出 / 未试 / 排除），接纳了几个；
  - `measured`。
- **DemoCodec**（只在示例里，标注「演示用，不是发布的投影资源」）：
  - `encode` 产出 JSON 消息数组，顺序固定：
    1. `{"role":"system","content": P}`；
    2. D 的每条按说话人：用户 → `user`，Nemu → `assistant`，内容是已投递的那段正文；
    3. 一条 `system` 消息放 E：先写两行身份事实（见 3.4），再按 E 的顺序每条一行写记录正文；E 为空时只写身份事实；
    4. `{"role":"user","content": o 的正文}`。
  - `measure` = UTF-8 字节数 + 每条消息 64（照 R-FABRICATE-01 的保守计量）。
- 记录正文 `body` 由场景里的 `time`、`speaker`、`text` 拼成 `[时间] 说话人：原文`，原文一字不改。

### 3.2 聊天程序 `tools/cause-demo/chat.py`
只用 Python 标准库，不加依赖。
- 启动：`python tools/cause-demo/chat.py --scenario tools/cause-demo/scenarios/basic.json`。
- 每轮：
  1. 读用户输入；
  2. 组 3.3 的 JSON：场景记录 + 已有对话（每句话是一条记录，`h_pos` 接着场景往后递增）+ 这句输入作 o；
  3. 调 `cargo run -q -p task-layer --example cause_demo`，经 stdin/stdout 交换；
  4. `sent` 才调 DeepSeek，其他结果把原因原样打印出来，本轮不回话；
  5. 打印回复，记进对话。
- 对话计划：D 保留全部已有对话，不裁剪。
- 命令：
  - `/show`：打印上一轮实际发出的完整 messages 和 binding；
  - `/quit`：退出。
- DeepSeek：
  - 模型 `deepseek-flash`；
  - 默认思考 high，`--no-think` 切成不思考；
  - `max_tokens` 8192，不设 temperature、top_p、seed；
  - 流式输出，边到边打印。
- 密钥只从环境变量 `DEEPSEEK_API_KEY` 读。不打印、不写文件。没有这个变量就直接退出并提示。
- 每次会话把完整请求、回复、usage、首字与完整耗时写到 `D:\Dev\cache\cause-demo\sessions\<UTC时间>.jsonl`。不进仓库。
- 回复结束后打印一行：本轮首字秒数、总秒数、估算花费。

### 3.3 场景与输入格式（`tools/cause-demo/scenarios/basic.json`）
场景字段：`records`（id、h_pos、time、speaker、text）、`relations`（from、to、kind）、`slots`（basis、h、first_f_pos、sources）、`budget`（capacity、output_reserve）。示例程序读到的输入，再多 `dialogue`（id 加 full/partial）、`latest`。`frontier_complete` 固定为 true，`decomposition_unknown` 固定为 false，`protocol` 用 3.4 的 P。

**basic 场景**（全部是编的。h 用一维，正负只表示方向。h_pos 按表格顺序取 1–7；对话里的每句话从 100 起往后递增）：

| id | 时间 | 说话人 | 原文 | 关系 |
| --- | --- | --- | --- | --- |
| e1 | 2026-11-01 20:10 | 竹 | 今晚九点上线，我们一起打一局三角洲。 | |
| e2 | 2026-11-01 23:40 | 竹 | 抱歉，今晚加班，没来得及上线。 | e2 BodySource → e1 |
| e3 | 2026-11-03 21:00 | 竹 | 教你一个词：「摸鱼」，就是上班偷懒。 | |
| e4 | 2026-11-05 15:20 | 网友阿杰 | 你就是个程序，装什么人。 | |
| e5 | 2026-11-05 15:30 | 竹 | 刚才那个人说的话别往心里去。 | e5 BodySource → e4 |
| e6 | 2026-11-06 10:00 | 竹 | 你上次说你喜欢下雨天。 | |
| e7 | 2026-11-06 10:05 | 竹 | 我记错了，你说的是喜欢晴天。 | e7 Correction → e6 |

槽（h、首次作用位置）：

| basis | h | first_f_pos | sources |
| --- | --- | --- | --- |
| s_missed | -0.8 | e1 的 h_pos | e1, e2 |
| s_insult | -0.6 | e4 的 h_pos | e4 |
| s_comfort | +0.5 | e5 的 h_pos | e5 |
| s_word | +0.3 | e3 的 h_pos | e3 |
| s_weather | +0.1 | e6 的 h_pos | e6 |

`budget`：capacity 1000000，output_reserve 8192。

再给一个 `tight.json`：同样的记录和槽，只把 capacity 调到「装得下基础请求和 s_missed，装不下再加 s_insult」。用 DemoCodec 实际量出来再定这个数，写进 README。用来体验「放不下就停」。

### 3.4 协议 P 与身份事实
- P 逐字取 `reports/R-FABRICATE-01/PLAN.md` 的「P 原文」。
- E 开头的两行身份事实逐字取 V2-PLAN 与 V3-PLAN：「本轮说话者是 Nemu，对话对象是用户竹。」「你知道的过去，只有下面记录里的内容。」
- 不加人设、性格、心情、示范、反编造训诫，也不加「别像助手」之类的话。

## 4. 验收
- `cargo fmt -p task-layer -- --check`、`cargo clippy -p task-layer --all-targets --locked -- -D warnings`、`cargo test -p task-layer --locked` 全过。
- 示例的确定性：同一份输入跑两次，stdout 逐字节相同。写成一个小脚本或测试，放进报告。
- 不调 API 的离线检查：对 basic 场景、空对话、o 为「你还记得我们约好打游戏那次吗」，打印 messages。确认五个包都被接纳，E 依次为 e1–e7（按 h_pos），e6 与 e7 同时出现。原样贴进报告。
- 真实跑一次：用 basic 场景聊 3 轮，问题由你自己定。完整对话贴进报告，附首字和总耗时、花费。tight 场景跑 1 轮，贴 `/show` 的 binding。
- 日志、报告、stdout 里都没有密钥。

## 5. 报告与交付
- `tools/cause-demo/README.md`：怎么设密钥、怎么跑、两个场景各是什么、`/show` 怎么看。写给用户看，十行以内。
- `reports/DEMO-CAUSE-01/REPORT.md`：结论在前，一页以内；第 4 节的检查结果；真实对话原文。
- 本卡原文存为 `tasks/requests/DEMO-CAUSE-01.md`。
- 开 PR，标题 `feat(DEMO-CAUSE-01): 组包器接 DeepSeek 的聊天演示`，写「请审查」后停下。不合并，不取下一张卡。
- 卡住（例如证据仓里找不到调用器、DeepSeek 参数和报告对不上）就写 `BLOCKED.md` 停下，不要自己猜。
