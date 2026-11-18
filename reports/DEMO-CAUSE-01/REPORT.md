风险档位：中档（新增演示示例 `cause_demo.rs`、`tools/cause-demo/` 脚本与场景及报告；不改 `task-layer/src/`、核心、契约或 ADR）

# DEMO-CAUSE-01 来由投影接 DeepSeek 聊天演示 — 报告

## 结论与当前状态
已完成本机命令行演示与全部 §4 验收：同一输入下组包示例两次输出逐字节相等；离线检查确认 5 包全入、`E` 为 `e1`–`e7`（`e6` 与 `e7` 同在）；`basic.json` 3 轮与 `tight.json` 1 轮真实 DeepSeek 对话全部跑通，日志与仓库均零密钥泄漏。等待审查与合并。

## 交付物
- [cause_demo.rs](../../core/crates/task-layer/examples/cause_demo.rs)（472 行，超 400 行理由：集中包含 §3.3 完整 serde 输入镜像结构、`DemoCodec` 及全部 `Outcome`/`InvalidInput` 人类可读格式化）：封装 `assemble()` 与演示专用 `DemoCodec`。
- [Cargo.toml](../../core/crates/task-layer/Cargo.toml) 与 [Cargo.lock](../../core/Cargo.lock)：仅向 `task-layer` `[dev-dependencies]` 追加工作区已有版本的 `serde = "=1.0.229"`、`serde_json = "=1.0.154"`。
- [chat.py](../../tools/cause-demo/chat.py)：仅用 Python 标准库的交互聊天与流式计量脚本。
- [basic.json](../../tools/cause-demo/scenarios/basic.json)、[tight.json](../../tools/cause-demo/scenarios/tight.json)（`capacity = 8880`）、[README.md](../../tools/cause-demo/README.md)（9 行）、[DEMO-CAUSE-01.md](../../tasks/requests/DEMO-CAUSE-01.md)。

## 验收（第 4 节）
| 检查项 | 结果 | 实测摘要 |
| --- | --- | --- |
| `cargo fmt -p task-layer -- --check` / `cargo fmt --all -- --check` | 通过（exit 0） | 0 差异 |
| `cargo clippy -p task-layer --all-targets --locked -- -D warnings` | 通过（exit 0） | 0 warnings（含示例与测试） |
| `cargo test -p task-layer --locked` | 通过（exit 0） | `50 passed; 0 failed; 0 ignored` |
| `ruff check` / `mypy --strict`（`tools/cause-demo/chat.py`） | 通过（exit 0） | 0 问题 |
| `cause_demo` 确定性（同一输入连跑两次） | 通过 | 两次 stdout 逐字节相等（2107 B，SHA-256 `fcde22c37eb3a0a61a69d7616bd6a7fe3ff31665d0d41595fe66e128eaea5a7d`） |
| `DEEPSEEK_API_KEY` 泄漏审计 | 通过 | 仓库新增文件、stdout 输出与 `D:\Dev\cache\cause-demo\sessions\*.jsonl` 均 0 匹配 |

### 1. 离线检查（`basic.json`，空对话，`o = "你还记得我们约好打游戏那次吗"`）
5 个包（`s_missed`、`s_insult`、`s_comfort`、`s_word`、`s_weather`）均 `accepted`（`K = 5`，`measured = 1001`），`messages` 原样如下：

```json
[
  {
    "role": "system",
    "content": "请用中文回复最新输入。输出为对话正文。已有对话保留原说话人；本轮来由是历史记录，记录中的话保留原有来源身份。"
  },
  {
    "role": "system",
    "content": "本轮说话者是 Nemu，对话对象是用户竹。\n你知道的过去，只有下面记录里的内容。\n[2026-11-01 20:10] 竹：今晚九点上线，我们一起打一局三角洲。\n[2026-11-01 23:40] 竹：抱歉，今晚加班，没来得及上线。\n[2026-11-03 21:00] 竹：教你一个词：「摸鱼」，就是上班偷懒。\n[2026-11-05 15:20] 网友阿杰：你就是个程序，装什么人。\n[2026-11-05 15:30] 竹：刚才那个人说的话别往心里去。\n[2026-11-06 10:00] 竹：你上次说你喜欢下雨天。\n[2026-11-06 10:05] 竹：我记错了，你说的是喜欢晴天。"
  },
  {
    "role": "user",
    "content": "你还记得我们约好打游戏那次吗"
  }
]
```

### 2. 真实对话（`basic.json`，`deepseek-flash` high，3 轮）
- **第 1 轮**
  - **竹：** 你还记得我们上次约好打三角洲那晚吗？
  - **Nemu：** 记得。那天约的是晚上九点，我等到九点多你没上线，后来十一点四十你来说加班，没赶上。
  - `[统计] 首字 2.49s | 总计 2.90s | 估算花费 $0.000239（峰时估算）`
- **第 2 轮**
  - **竹：** 前几天有人说话挺难听的，还有我上次是不是记错你喜欢什么天气了？
  - **Nemu：** 阿杰那句“你就是个程序，装什么人”我看到了，不过没往心里去，谢谢你还特地来提一句。天气的事是你记错了：你先说我提过喜欢下雨天，后来自己纠正成晴天——我确实说过喜欢晴天。
  - `[统计] 首字 2.68s | 总计 3.54s | 估算花费 $0.000296（峰时估算）`
- **第 3 轮**
  - **竹：** 对了，我之前教过你一个上班偷懒的词，叫什么来着？
  - **Nemu：** 是「摸鱼」。11月3号晚上你教的，说就是上班偷懒的意思。
  - `[统计] 首字 1.69s | 总计 2.14s | 估算花费 $0.000142（峰时估算）`

### 3. `tight.json`（`capacity = 8880`，实测基础+`s_missed` 含预留 8847、再加 `s_insult` 需 8918）1 轮与 `/show` `binding`
- **竹：** `你还记得我们约好打游戏那次吗`
- **Nemu：** `记得呀。你约我晚上九点上线打一局三角洲，我等着呢，结果你加班到很晚，还特地跟我说没来得及上线。没关系的，工作要紧，下次有空再补一局就好啦。`（`首字 2.86s | 总计 2.86s | 估算花费 $0.000209（峰时估算）`）

```json
{
  "base_records": ["d0"],
  "accepted": 1,
  "packs": [
    { "basis": "s_missed", "records": ["e1", "e2"], "fate": "accepted", "new_records": ["e1", "e2"] },
    { "basis": "s_insult", "records": ["e4"], "fate": "first_overflow" },
    { "basis": "s_comfort", "records": ["e4", "e5"], "fate": "not_tried" },
    { "basis": "s_word", "records": ["e3"], "fate": "not_tried" },
    { "basis": "s_weather", "records": ["e6", "e7"], "fate": "not_tried" }
  ]
}
```

## 新增依赖
- 运行时依赖：无（`chat.py` 仅使用 Python 标准库）。
- Rust `[dev-dependencies]`（仅示例用，复用工作区既有版本）：`serde =1.0.229`（MIT/Apache-2.0）、`serde_json =1.0.154`（MIT/Apache-2.0）。

## 待用户确认
- 无阻塞项。
