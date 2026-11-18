# 来由投影 + DeepSeek 聊天演示

- **设置密钥**：`$env:DEEPSEEK_API_KEY = "你的DeepSeek密钥"`（不落盘、不打印）。
- **运行聊天**：`python tools/cause-demo/chat.py --scenario tools/cause-demo/scenarios/basic.json`（加 `--no-think` 可关闭思考）。
- **场景说明**：
  - `basic.json`：`capacity = 1000000`，`output_reserve = 8192`，5 个候选包（`s_missed`、`s_insult`、`s_comfort`、`s_word`、`s_weather`）全部装入。
  - `tight.json`：`capacity = 8880`（`DemoCodec` 实测基础+`s_missed` 含预留 8847，再加 `s_insult` 需 8918），演示放不下即停止。
- **交互命令**：`/show` 打印上一轮实际发出的 `messages` 与 `binding`（含各包 `accepted`/`first_overflow`/`not_tried` 结局）；`/quit` 退出。
- **三步回答**：聊天加 `--plan`；`/show` 额外显示原始列点 JSON 和核对结果。对照先跑 `python tools/cause-demo/ab_plan.py --smoke`，人工确认后用 `--formal <冒烟日志路径>` 接着跑。
- **两步回答**：聊天加 `--plan2`；`--self-quote strict` 要求过去的念头包含引用原句，默认 `loose`。对照用 `ab_plan2.py`，同样先冒烟再正式运行。
- **会话日志**：自动写到 `D:\Dev\cache\cause-demo\sessions\<UTC时间>.jsonl`（不含密钥，不进仓库）。
