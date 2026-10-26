# P0-10 载体推理引擎实测

状态：Q4_K_M 注入配对已实测；server 速度/IRL 在日志证据检查处阻塞并停止，等待用户决定。PR 保持草稿。

本目录只做 llama.cpp 可复现实测，不选择生产引擎，不修改载体或 IRL 契约。

- 固定载体：Qwen3-4B-Instruct-2507，修订 `cdbee75f17c01a7cc42f958dc650907174af0554`。
- 依任务卡比较量化速度、注入位置、零调制逐 token 一致性、每请求更新向量和受约束 IRL 输出。
- 同驻目标按用户 2026-10-25 指示改为 GPT-SoVITS V5 与渲染；未部署时先只测载体，明确记录缺项。
- GPU 测量须先通知用户测量内容与预计时长，等待用户停掉 runner，并确认 `Runner.Listener` / `Runner.Worker` 均不存在。
- 未测项目一律写「未实测」；遇到阻塞写入 `reports/P0-10/BLOCKED.md` 并停止。

计划见 [MEASUREMENT-PLAN.md](MEASUREMENT-PLAN.md)；只读盘点可用 `preflight.ps1` 复现。

完整入口、固定配置与 CPU 检查见 [FULL-RUN.md](FULL-RUN.md)。最小结果见报告的 SMOKE-REPORT.md；不得把它当作完整任务验收。

报告：[REPORT.md](../../reports/P0-10/REPORT.md)。阻塞：[BLOCKED.md](../../reports/P0-10/BLOCKED.md)。
