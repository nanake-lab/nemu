重档：修复 executors/ 的零工具守卫，保持独立交叉审查要求。

# P5-07 R1 审查修复

## 当前状态

三项审查阻塞已修复并完成本机验收，等待增量复审。P5-07 整卡仍为部分交付；正式 IPC、真实模型/语义质量与人工金标准验收未完成。本轮不接生产、不合并。

## 逐条回应与规格对照

| 审查项 | 修复 | 回归证据 |
| --- | --- | --- |
| P1：userMessage 被零工具守卫误拦 | bridge.ts 仅将工具相关 ThreadItem 判违规；普通消息、计划、上下文压缩等生命周期不再被当作工具。身份校验仍保留 | replay.test.ts 的 R1-P1：依据 P0-08 真实成功审计中的 userMessage 类型重建启动/完成正例；另测 5 类非工具项、11 类工具项拒绝及用户消息身份错配 |
| P2：失败后同批 delta 泄漏 | 通知入口检查首个失败；失败后不再处理后续通知、不覆盖首个原因，原失败路径关闭连接且不重提 | R1-P2：同批 function_call → delta → completion → reroute；回调为空，首个原因仍是工具违规，只发一次 turn/start |
| P2：原始违规证据丢失 | 错误 cause 保留首个违规通知或 MCP 响应的原始解码对象及已知 thread/turn ID。MCP 在验证前保留响应，未知字段也保留。每个对象已由 RPC 帧大小限制约束；不存数据库、不写普通日志 | R1-P3：工具名、call ID、参数保留；非零及无法解析的库存连同未知字段保留；用量账不混入内容 |

依据：P0-08 REVIEW-R1-REPORT 的“零工具/零 MCP、保留原始证据并中止”，ADR 0004、CONTRACTS §6、I16 及 AGENTS §5。证据是解码后的原始 JSON 对象，不声称逐字节传输录制。没有新增依赖、IDL、黄金向量或阈值，也没有通用审计框架。手写文件均小于 400 行。

## 验收与失败

先只加入回归测试，原实现 **10 failed / 17 passed**，分别复现上述三类问题：[失败原日志](regression-before.log)。修复后三类测试通过；随后补足工具拒绝/身份反例，完整工作区验证如下。合成轨迹不冒充真实 provider 帧录制；P0-08 文件只用于确认真实成功记录确实包含 userMessage。

| 命令（仓库根，Python 命令在 workers/irl） | 结果 |
| --- | --- |
| pnpm lint | passed |
| pnpm typecheck | passed |
| pnpm test | passed：54 文件、620 项（bridge 40 项） |
| pnpm exec tsc -p executors/codex-bridge/tsconfig.json --outDir D:/Dev/cache/P5-07/bridge-review-r1 | passed |
| uv run --offline ruff check . | passed |
| uv run --offline mypy --strict src tests | passed |
| uv run --offline pytest | passed：28 项 |

结果及原日志：[TS](results.json)、[Python](python-results.json)、[全工作区测试](test.log)、[Python 测试](pytest.log)。进程内 TEMP/TMP、UV 缓存/虚拟环境/pycache 和编译产物均在 D:/Dev/cache/P5-07（共享 UV 缓存 D:/Dev/cache/uv）；仅合成输入与临时目录，无真实推理或用户存储。

## main 与范围

上次被测 head 02033260、实际 main 基线 B=acc71dad；本次 fetch 得到 M=8fed313b7ca9c31423d0ad442789ddef7e6b063b，已 merge 为 5d529e32，不 rebase、不强推。B→M 七个提交涉及 ADR/研究记录、独立 Rust recompute 与看板；worker、bridge、IDL 和 Node 依赖无增量。仍按本轮桥接源码修复重跑上述检查，未把独立 Rust 实现算成本次验收。ADR 0031/0034 的条件性模型决定留待正式模型后继；#129 已解决看板部分交付状态支持。

本轮仅修改 executors/codex-bridge/src/base/ 和 reports/P5-07/。旧原始证据不覆盖。生成类型 45 份仍原样；范围、哈希、JSON 与 Markdown 链接核验见 [核验记录](verification.json)。

需要用户安排独立增量复审。正式 IPC 与人工金标准后继仍在原两份请求中，人工标注需要用户参与，不代标、不降低验收要求。
