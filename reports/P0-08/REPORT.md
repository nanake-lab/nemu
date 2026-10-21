# P0-08 当前结果：第二部分实测（2026-10-22）

技术测量已完成；整卡仍等待 Claude 审查和 P0-09 用户确认。当前完整报告见 [PART2-REPORT.md](PART2-REPORT.md)，数据见 [benchmark.md](benchmark.md)，候选选型见 [decision.md](decision.md)。本地回退后端候选 P90 截止时间 26168.09769999236 ms；正式配置未改。

---

## 第一部分历史报告（以下原文保留，其未测状态已由第二部分更新）
# P0-08 Codex app-server 与 IRL 路径验证 — 报告

## 结果

部分完成：可复现探针、合成数据和配置候选已建立；账号登录与 P0-07 载体依赖阻止了实测，因此后端决策和验收未完成。

## 交付物

- `spikes/codex-app-server/`：JSON-RPC 探针、同版生成的 app-server 类型、200 条合成语料、实验 SemanticCandidate schema、运行说明和候选配置。
- `reports/P0-08/benchmark.md`：分项实测状态与尚无数据的说明。
- `reports/P0-08/decision.md`：后端和延迟决策；所有未测值明确留空。
- `reports/P0-08/BLOCKED.md`：登录、本地载体和 GPU/runner 前置条件。
- `tasks/requests/P0-08-config-scope.md`：卡外正式配置的范围请求。

## 验收

| 命令或检查（含验收项） | 结果 | 证据 |
| --- | --- | --- |
| 草稿 PR 认领 | 已建立，仍为 draft | PR #29，`task/P0-08-codex-irl-spike` |
| 同版 `codex app-server generate-ts --experimental` | 通过；888 个文件 | `spikes/codex-app-server/drafts/cli-protocol-candidate.json` |
| `node --check spikes/codex-app-server/src/probe.mjs` | 通过 | 本轮命令退出码 0 |
| `node --check spikes/codex-app-server/src/hash-generated.mjs` | 通过 | 本轮命令退出码 0 |
| probe `--help` | 通过 | 列出 catalog、smoke、speech、speech-all、irl、controls 命令 |
| `node spikes/codex-app-server/src/probe.mjs smoke --code-home <isolated path>` | 通过 | `local-smoke.json`：初始化成功并启动 ephemeral thread；未启动推理 turn |
| 合成语料生成 | 通过；200 条（zh 67、en 67、ja 66） | `spikes/codex-app-server/fixtures/irl-cases.json` |
| 远程语音首增量，三种语言各 50 次 | 未运行；隔离 profile 未登录 | `reports/P0-08/BLOCKED.md` |
| 远程 IRL `outputSchema` 200 次 | 未运行；隔离 profile 未登录 | `reports/P0-08/BLOCKED.md` |
| 本地 m=0 IRL 200 次 | 未运行；P0-07 载体未选 | `reports/P0-08/BLOCKED.md` |
| `turn/interrupt`、`turn/steer`、命令/文件审批 | 未运行；隔离 profile 未登录 | `reports/P0-08/benchmark.md` |
| 100 次语音及 100 次 IRL 的额度记录 | 未运行；隔离 profile 未登录 | `reports/P0-08/benchmark.md` |
| GPU/runner 前置检查 | 暂未运行 GPU 测量；届时先核对 Runner.Listener/Worker | `AGENTS.md` 第 9 节；`reports/P0-08/BLOCKED.md` |
| 全仓静态检查及测试 | 未运行；本卡实测被前置条件阻塞 | — |
| 后端阈值、P90 截止时间和 P0-09 延迟目标 | 未判定 | `reports/P0-08/decision.md` |

`probe.mjs` 超过 400 行，是因为单进程内集中处理 JSON-RPC 请求相关、流式增量计时、审批拒绝及批次汇总；测量期间保持同一连接可避免跨进程丢失事件顺序。

## 规格对照

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| ADR 0003：测量线程使用临时空目录、关闭 shell/app/plugin/web-search/multi-agent 能力、无动态工具、只读无网络、`personality: none`、样例协议 p | `spikes/codex-app-server/src/probe.mjs` | 代码配置已核对；模型端运行行为未登录验证 |
| ADR 0004：Codex app-server 使用独立 profile，CLI 与类型生成版本一致 | `spikes/codex-app-server/README.md`、`src/hash-generated.mjs` | CLI 0.162.1，类型生成成功；认证阻塞 |
| SemanticCandidate 为实验候选，不成为 canonical 类型 | `semantic-candidate.schema.json` 与 drafts | JSON 文件由本地静态解析；无远程 schema 输出可验 |
| 对话语义保留来源并禁止补造事实 | `src/prepare-fixtures.mjs`、`src/probe.mjs` prompt | 合成数据已生成；语义质量未测 |

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| 无 | — | — | 脚本仅使用 Node.js 内置模块；Codex CLI 0.162.1 为已安装工具，不新增项目依赖 |

## 待用户确认

- [ ] （阻塞）账号所有者是否启用隔离 CLI profile 所需的设备码登录，或自行在该隔离 profile 完成组织允许的登录。
- [ ] （阻塞）P0-07 载体结果可用后，再进行本地 IRL 对照。
- [ ] （阻塞）实测完成后在 P0-09 确认延迟目标和后端。
- [ ] （不阻塞）P0-09 评审是否接受仅在 spike 范围内保留的实验 schema。

## 后续建议（不在本任务范围内）

- 登录和 P0-07 载体就绪后继续补齐远程额度、语音、IRL、审批及本地测量证据。
- 用户批准范围请求并完成所需 ADR 流程后，再由有权限的卡更新正式配置。

## 下一步

等待隔离账号登录条件和 P0-07 载体结果；本 PR 保持 draft 并标记 `needs-user`。解除阻塞并完成全部测量后更新本报告、请求审查。之后按 `tasks/README.md` 第 2 节重新确认队列中的下一张卡。
