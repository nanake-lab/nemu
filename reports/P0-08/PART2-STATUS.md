# P0-08 第二部分当前状态

- 分支 task/P0-08-part2，从 origin/main 806e09b 开始；草稿 PR #50。
- #29 已补 unverified 和评论；等待第二部分 Claude 审查后再移除。
- 隔离 profile 登录成功；codex login status 为 Logged in using ChatGPT；授权后 git status 干净。未读取认证文件内容或浏览器页面。
- 用户于 2026-10-22 指定远程日常档 gpt-6-luna / max，覆盖卡片的 low 参数；低强度和 schema 修复预检不计入正式批次。
- CLI 0.162.1，同版协议重新生成 888 文件，清单 SHA-256 与第一部分一致。
- 远程模型目录已取得；三语各 50 条短回答已完成。另保留日文原始 9 条连接失败记录；补测单独标记。200 次 IRL 进行中，控制探针待测。
- 第一部分 schema 的 oneOf 被严格接口拒绝；失败原记录在 part2/preflight，已生成兼容 wire schema，仍用原 schema 校验。
- 本地脚本和依赖已准备，权重三分片大小和哈希均匹配；GPU 未启动。用户已确认远程完成后安排安静窗口并停掉相关任务；实际开始前仍核查进程。
- pnpm lint、pnpm typecheck、pnpm test（443/443），JS schema 4 项测试、Python source/parser 3 项 CPU 测试以及 Ruff、strict mypy 已通过。
- 正式配置未改；helpers 候选仅登记 catalog 中实际模型 ID，预算仍空。

## 本轮测量边界

- 首增量：中文 P50/P90 = 5962/8369 ms；英文 5142.76/7060.39 ms；日文（含单列补测成功样本）5561.79/7212.88 ms。每语 50 个成功且互异的夹具；日文共 59 个尝试槽位，其中 9 个失败，不能隐去失败或把这些槽位均当作已启动的模型调用。
- 原连接在日文第 28 条 turn 超时后不再响应 RPC。主代理确认父子进程身份后仅停止本次 app-server；进程占用及失败记录在 part2/server-stall.json。根因尚未确定，不能归因为模型、网络或会话数。
- 之后每个已完成临时线程调用 thread/unsubscribe；连接级超时中止批次，保留原始记录，用 start-index 从后续夹具续跑。不同源代码快照、哈希和数据段清单保存在 part2/protocol/ 与 remote-speech-all-summary.json。
- 原批次在中英文 100 条成功边界有精确额度快照：5h 使用率由 0% 到 1%，周窗口均 51%。最终补测后也是 1%/51%；账号共享，不归因为本探针的独占消耗。
- 当前中间态机械核验委派给 routine_reporter，按角色名调用；读取配置为 gpt-6-luna/max。主代理核对原日志：lint/typecheck 退出 0；29 文件、443 测试通过。未独立证明后端引擎运行时模型身份，不把配置读数当作该项证明。
- 完成前保持 #50 为草稿，#29 保持 unverified。最终合并最新 origin/main 后再跑完整相关检查，之后等 Claude 审查。

## 远程 IRL 第 100 条额度边界

part2/remote-irl-30ed8fe1-5bca-4552-8647-08e13b2431a8-quota-after100.json 已写入：5 小时窗口 3%、周窗口 52%；批次前 1%/51%。账号共享，不能独占归因。批次继续，未据中间结果作后端判定。
