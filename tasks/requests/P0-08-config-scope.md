# P0-08 配置范围请求

## 原因

P0-08 的步骤要求记录 Codex CLI 版本到 `config/models.lock.json`、把三档模型写入 `config/helpers.json` 草稿，并按 IRL 测量结果写部署 profile 草稿；但卡头「目录」只授权 `spikes/codex-app-server/`。根据 AGENTS.md 第 2 节，本卡不直接修改这些卡外路径。`config/helpers.json` 还受 AGENTS.md 第 6、7 节的 ADR 与用户批准约束。

## 期望接口

本卡将在获准目录 `spikes/codex-app-server/drafts/` 生成候选记录，字段与正式接口对齐：

- Codex 锁定候选：CLI 版本、生成类型版本和生成文件摘要；
- 助手档位候选：provider、`everyday`/`work`/`hard` 模型 ID、每日预算；
- 部署候选：IRL 后端、实测 p90 截止时间和测量证据引用。

请求后续为正式配置更新安排明确范围，并在需要时先取得对应 ADR 与用户批准。候选仅供 P0-09 评审，不改变运行配置。
