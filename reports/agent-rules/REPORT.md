# 子代理规则独立入库 — 报告

## 结果

规则及项目角色配置完成，等待本PR最终提交上的Ubuntu/Windows CI；满足AGENTS第6节条件后squash合并。用户于2026-10-17明确要求将此规则从P0-07分离并先合并。

## 交付物

- AGENTS.md：只追加第9节，保留main已有全文。
- .codex/agents/routine-reporter.toml：GPT-6 Luna / max，机械性工作只返回证据，禁止修复、改测试、提交、推送、合并及验收裁决。
- reports/agent-rules/verification.log：独立配置、追加边界和文件范围核查。

项目不限制累计子代理数量；平台槽位与服务限制仍有效，重负载全局串行，GPU与runner互斥。32GB内存不放宽资源或验收规则。规格解读、验收判断、选型和合并由主代理负责。

配置依据为OpenAI官方 https://learn.chatgpt.com/docs/agent-configuration/subagents 。客户端现已发现routine_reporter，主代理以agent_type=routine_reporter执行本PR实际只读核查；工具声明该角色固定模型gpt-6-luna、推理max。规则原先在P0-07分支试用，本PR不引入其模型脚本、权重或测量证据。

## 验收

| 检查 | 状态 | 证据 |
| --- | --- | --- |
| AGENTS main原文完整前缀，只有追加 | 等待核查记录 | verification.log |
| TOML字段、SHA-256、变更范围 | 等待核查记录 | verification.log |
| Ubuntu静态检查与单测 | 待本提交CI | GitHub PR checks |
| Windows完整作业 | 待用户runner执行 | GitHub PR checks |

只改规则和文档，无新增运行时依赖，无规格/契约扩展或阈值变化，无待用户批准的阻塞决定。CI尚未完成不视作已验收。Windows CI由runner执行，不同时本机跑Electron/Playwright/GPU。

## 下一步

等待最终提交CI全绿并确认仍包含最新main后合并。本独立小PR完成后回到已在做的P0-07 int4优化，不取新卡。

## 主代理核对与CI前状态

独立核验退出0：main AGENTS全文为完整前缀，字段符合GPT-6 Luna/max，配置SHA-256为fe68a73c2ed296d4844fc35248e0036179199d0dbba699603937b284ce0a824f，变更均在规则/配置/本报告目录。主代理核对该结果，前表两项本地核查现为通过；CI两项仍待最终提交。runner查询当时为offline，主代理将通知用户上线处理本PR和合并后main作业。
