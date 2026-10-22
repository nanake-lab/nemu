# P0-08 复审修订报告（2026-10-22）

## 结果

技术修订与补测完成；整卡仍为部分完成，等待Claude复审及P0-09用户架构决定。按用户本次明确要求，在同步主线、检查完成后开放复审；不合并，不提前移除#29的unverified。见 BLOCKED.md。

## 对 Claude 评论 6332417953 的逐项回复

| 项目 | 修改与证据 |
| --- | --- |
| 1. 日常低推理组 | 原max为用户指定对照，保留350条；原配置low完成三语各50和IRL200，但第158条两次资源调用，整组仅诊断。修复后隔离low完整重测350条，全部零工具调用及逐条零MCP。三组分列，不能把隔离减少上下文的效果当作纯effort效应 |
| 2. 26秒不能是默认 | decision.md第一行及判定已改为机械规则输出、不可部署、交P0-09。新low机械输出remote/P90=9269.47ms，仍不可部署；历史max的local/26168.09769999236ms仅保留对照。操作backend/deadline为null；四个候选方向见 review-r1/OPTIONS.md，本卡未实施 |
| 3. 耗时拆分 | 远程逐条首增量前/后、完成耗时及5种token用量；本地固定50输入、交替两臂、两次排除预热，记录prefill wall/CUDA span、TTFT、逐token时刻、生成速率、回调及解析器构建耗时。见下表与两份专项报告 |

## 主要实测结果

| 项目 | P50 / P90 | 有效性与边界 |
| --- | --- | --- |
| 隔离low语音首增量（三语150条） | 4680.75 / 6896.31 ms | 三语各50；文本组成探针，不是可听见回复 |
| 隔离low IRL首增量前 | 5232.75 / 8136.44 ms | 混合网络、排队、prefill、推理，无法单独归因 |
| 隔离low IRL首增量后 | 1170.66 / 1899.81 ms | 每条完成-首增量后再取百分位，非边际百分位相减 |
| 隔离low IRL完成 | 6524.99 / 9269.47 ms | Schema200/200，来源147/200（73.5%）；来源检查不等于语义准确率 |
| 本地无约束完成（50条） | 9905.26 / 15723.42 ms | Schema49/50，来源43/50；仅诊断，不替换原200条 |
| 本地有约束完成（50条） | 15389.07 / 25592.72 ms | Schema49/50，来源43/50；输出哈希与原前50条50/50相同 |
| 本地无/有约束生成速率P50 | 21.18 / 13.55 token/s | 排除首token；逐对比值中位1.56，共同token前缀比较亦约1.56 |

约束回调wall占整次请求比例中位31.15%，包含可能的设备等待。无约束完成仍约9.9秒，故约束路径和模型/输出规模都有影响；单换解码器不足以达到400ms。两臂均有相同观测器与同步开销，不称为纯GPU或纯算法时间。原本地200条192/200 Schema、169/200来源及8条日文解析失败仍保留。

## 远程 token 总量

| 组/工作负载 | input | cached input | output | reasoning output | total |
| --- | ---: | ---: | ---: | ---: | ---: |
| 用户指定max 语音150 | 1465640 | 1094400 | 21455 | 18717 | 1487095 |
| 用户指定max IRL200 | 2083346 | 1658880 | 324184 | 271809 | 2407530 |
| 原配置low 语音150成功/151尝试 | 1444215 | 1084800 | 2500 | 0 | 1446715 |
| 原配置low IRL200（违规诊断） | 2083562 | 1669248 | 53904 | 13074 | 2137466 |
| 隔离low 语音150 | 130511 | 0 | 2518 | 0 | 133029 |
| 隔离low IRL200 | 301650 | 24320 | 43943 | 3299 | 345593 |

reasoning为output子集、cached input为input子集。原low超时行的用量未知，不补零；其余表列成功行用量覆盖完整。API计数不是账单，账号快照不是这些调用的独占额度。三组的逐语言分段、用量覆盖与前/100次/最终额度快照见 review-r1/REMOTE-COMPARISON.md。

## 新发现与修复：工具面实际未隔离

原配置初始化时MCP=0，但首轮后变3；仅禁用remote_plugin并不能关闭本地plugins。资源调用实测status=completed、无error，不是虚构工具名。关闭orchestrator或继承环境仍失败；关闭本地插件/技能后MCP为0，但模型目录multi_agent_version=v2仍暴露list_agents。

最终profile关闭本地plugins、cloud skills、技能MCP安装、orchestrator MCP、native collaboration，去继承CODEX应用上下文；v3目录只将Luna协作版本置disabled，其他Luna字段及其他模型均不变。两项明确资源请求探针零调用且轮后零MCP；正式350条逐条同样检查。语音与IRL在任何工具调用、未知或非零MCP时保存原行和partial summary后中止。该证据不等于一般安全证明。失败历史见 review-r1/audit-run-index.json，成功见 resource-surface-audit-v3.json。

## 交付物与复现

- review-r1/REMOTE-COMPARISON.md、remote-comparison.json：三组分段、token、额度、有效性、规则；`pnpm probe:p0-08:compare`重算。
- review-r1/LOCAL-TIMING.md、local-breakdown/：50对原始输出、逐token时间、绑定与summary；`pnpm probe:p0-08:timing-summary`重算。
- REVIEW-R1-METHODS.md：计时、观测器、混杂、失败和解释边界。
- review-r1/plan.json、low-recovery-plan.json、low-isolated-plan.json 与精确gzip源码：实测版本与恢复计划；原数据不覆盖。
- spikes/codex-app-server/：隔离、日志守卫、配对计时和分析器；根package.json只追加4个命令。
- decision.md及部署草稿：机械规则输出与实际操作字段分开。正式helpers/models lock/profile未改。

## 验收与检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 新low350条覆盖、Schema/来源重算、零工具/逐条零MCP、源码归档 | 通过 | remote-analysis-final.log、remote-comparison.json、low-isolated-plan.json |
| 本地100正式行/50对、2预热、来源/Schema/输出SHA、源码绑定 | 通过 | local-breakdown-analysis.log、local-breakdown/summary.json、local-source-archives.json |
| GPU窗口及P4错开 | 已确认；05:18:12Z前置，05:43:59Z完成，之后通知恢复runner | quiet-window-preflight.json、local-breakdown/quiet-window.json、原始运行日志；逐臂前重复检查 |
| 控制与审批行为 | 沿用原第二部分证据，未冒充在隔离无工具线程重测 | PART2-REPORT.md及part2控制日志 |
| 新代码最终静态/测试 | 通过：lint/typecheck；29文件443项仓库测试；Python13项；Node13项；Ruff及7文件strict mypy；18个JS语法检查 | review-r1/final-checks/（25份原始日志） |

新Node守卫/环境过滤/目录派生的7项预检通过；之前13项Python、9项Node及7文件ruff/mypy通过。初次mypy失败、旧CMD调用失败、协议违规和超时原记录保留，不用重跑覆盖原因。最终合并后检查以 final-checks 为准。

## 新增依赖

无。复用固定Torch2.10.0+cu128、Transformers4.57.3、bitsandbytes0.50.2、LMFE0.11.3、jsonschema4.26.0；未修改P0-07环境。probe.mjs沿用超过400行的单进程JSON-RPC/事件顺序组织，避免把计时边界拆散；新增环境、计时和分析模块独立。

## 待确认与下一步

- （阻塞）P0-09用户决定架构、IRL后端/截止时间、延迟目标与helpers正式ADR。当前规则均不达400ms，操作字段null。
- 等Claude复审本次三个问题及工具面新发现；#29的unverified在复审通过前保持。
- （不阻塞，本卡不扩展）A/B/C端到端P50/P90保持null，P0-09汇总P4数据；本地解析失败和来源质量如实记录。
- 按用户本次明确要求，最终检查完成后转为可审查供Claude复审，不表示P0-09已批准或允许合并。未取下一卡。

## 最终同步与执行证据

- 已merge origin/main bf69c3244f76435ae560c3ba8dff4a2817c5cfa9，检查基线HEAD为84ef0f0e2b13a04c0bde24f815645990c6b37b91。这次主线仅新增P0-09议程，AGENTS与本卡无变化；已全文阅读议程。用户已表态关键路径低推理、后台理解可高推理；“两层”架构仍待决定，本卡未实施。
- 机械检查由routine_reporter角色执行，固定Luna/max，主代理核对25份日志均退出0；本地TOML自动加载能力未验证。18个JS语法检查针对最终字节。审计脚本EOF空行在lint期间删除，无语义变化；另以lint-final-bytes.log记录最后一次lint。
- 远程和本地分析器在merge后重算通过，post-merge-remote-analysis.log、post-merge-local-analysis.log保留。范围和凭据模式扫描见final-scope-scan.json：无范围外文件、无凭据模式命中；此扫描不是一般秘密泄漏证明，未打开任何auth文件。原始日志不为消除空白警告而改写。
- CI状态与最终提交在PR页面跟踪；技术修订开放复审不等于P0-09批准。
