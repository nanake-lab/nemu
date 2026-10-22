# P0-08 复审修订进度

针对 Claude 评论 6332417953，当前保持草稿。

- max 是用户明确指定的对照，原始数据不替换；补测 low 的三语各50条和IRL 200条，其余远程配置、输入、顺序、Schema及计时口径不变。
- 已把 decision 第一行改为“规则输出，不可用作部署值”；部署草稿的操作字段已清空，机械规则结果另存，交 P0-09 作架构决定。
- 新增逐条远程时间分段和 token 汇总，不以边际百分位相减。
- 本地固定50输入做有/无约束配对计时，用户已批准远程完成后的30–45分钟GPU窗口；开始前须再次确认 runner/Blender/P4测量没有运行。
- Context7 额度不可用，已查阅 PyTorch 2.10 的 CUDA Event / forward hook 文档及 Transformers 4.57.3 的 streamer 文档和本地源码。
- 新增 CPU 计时测试3项和远程分段测试2项通过。初次 strict mypy 发现 CUDA Event 缺类型声明，显式 Protocol 后3文件通过，失败原日志保留。

完整修订和实测完成后，再 merge 最新 origin/main、重跑检查，按用户本次明确指令转为可审查并等待 Claude 复审。#29 的 unverified 不提前去掉。

## 本轮新发现：low IRL 严格无工具失败

三语各50个成功回复（另1次日文超时）及IRL200次已返回。IRL第158条 case-0158 观察到两次 list_mcp_resources，原始行保留，整组不能当作严格无工具合格组。默认比较器拒绝该组；显式 `--diagnostic` 仅输出带违规标记的诊断统计，low 后端/截止时间为 null，不放宽验收。诊断文件为 review-r1/remote-diagnostic-comparison.json。

候选修复为 app-server 启动及线程配置同时关闭 `orchestrator.mcp.enabled`。依据官方 config/src/config_toml.rs 的 OrchestratorToml.mcp/FeatureToggleToml，以及 core/src/config/mod.rs 的 orchestrator_mcp_enabled 解析。本地 MCP 服务列表为空不能覆盖这一路径。该解释待能力探针验证，不提前断言根因。隔离版本将单独存入 low-effort-isolated，旧组不覆盖。

本地配对测量于 2026-10-22T05:18:12Z 完成安静前置检查后开始；P4-01 idle，P4-02 notLoaded，Runner.Listener/Worker、Blender、竞争测量均无。GPU结束前不运行额外测试或远程探针。

## 隔离修复与完整重测

- MCP 服务会在首轮后从0变3；原启动目录的空列表不证明隔离。先后尝试仅关闭 orchestrator MCP、去继承环境，资源工具仍可调用；失败原证据保留。关闭本地 plugins、云技能/技能MCP安装后MCP为0，但仍暴露 list_agents。模型元数据 multi_agent_version=v2 可独立于线程 feature；新派生 catalog v3 将其置 disabled，并关闭 native collaboration tools。
- resource-surface-audit-v3.json：明确请求两个资源工具，2/2完成、0工具调用、轮后0 MCP；配置回读 plugins/cloud skills/orchestrator MCP均false。该结果是能力探针证据，不是一般安全证明。
- 新 low-effort-isolated 组完整重跑三语各50和IRL200。源码/辅助模块精确归档、计划与v3 catalog哈希见 low-isolated-plan.json。轮后MCP查询在完成计时外；语音和IRL均在工具调用、非零或未知MCP目录时保存原行并中止。
- 原max按用户指定保留；旧low整组保留为诊断。新组注入工具上下文减少，不能把新旧差异全归因于effort；原max虽350条零观测调用，也不能继续当作工具能力不存在的证明。
- 本地50对已完成，详见 review-r1/LOCAL-TIMING.md：无/有约束完成P50为9.905/15.389秒，生成速率21.18/13.55 token/s，回调占比中位31.15%。GPU窗口已结束并通知用户恢复runner。

## 补测完成

新隔离low语音150与IRL200全部完成并通过覆盖、零工具/逐条零MCP核验；IRL完成P50/P90=6524.99/9269.47ms，Schema200/200、来源147/200。最终结果以 REVIEW-R1-REPORT.md、decision.md 和 review-r1/remote-comparison.json 为准。以下进入最新main同步、检查与开放复审。
