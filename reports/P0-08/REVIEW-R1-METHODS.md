# P0-08 复审补测口径

## 远程

- 原 `part2/strict-speech/` 的 Luna/max 是用户指定对照；本次 `part2/low-effort/` 为卡片要求的 Luna/low。模型、严格无工具 catalog、协议 p、语料、Schema、顺序、计时起止沿用原组；源码和计划绑定见 `review-r1/plan.json`。不混合两组来决定百分位。
- 三语各 50 个成功回复，IRL 200 条。日文原批次第 4 条发生 300 秒 turn_timeout，原因未确认，原行与 partial summary 保留；续测 5..50，再单独补测第 4 条。故语音 151 次尝试/150 次完成；超时未赋有限延迟，其 token 与原始工具捕获缺失不能当作零。segment manifest 明列所有批次。
- 首增量前 = 原 firstDeltaMs；首增量后 = 每条 completedMs - firstDeltaMs，分别再取 nearest-rank P50/P90。前段含客户端、网络、排队、prefill 和推理，不能单独归因为 reasoning；后段是客户端观测的流尾，不是 GPU 独占解码。
- 报告 API 返回的 total/input/cached-input/output/reasoning-output token 计数、覆盖行数和总量；reasoning 是 output 的子集，cached-input 是 input 的子集，不重复相加。计数不是费用，不能换算独占账户配额。失败行未知用量不补零。
- 两组不同时间、缓存和服务端环境，不把差异全部归因于 effort。配额按原账号快照呈现，跨重置/其他使用不能简单相减。原 max 的控制/审批探针沿用，不用无工具测量线程替代审批证据。

## 本地固定 50 输入配对

- 原载体固定 Qwen3-4B-Instruct-2507 的 cdbee75f17c01a7cc42f958dc650907174af0554，NF4/bf16、eager、145 个 native RMS，m=0 无激活调制 hook。使用同一原始 200 语料的前 50 条（zh17/en17/ja16）。不替换原 200 条的选型总体。
- 每个输入有/无 LMFE v2 约束两臂，交替谁先运行；各有一次排除的预热。相同 greedy/seed731、max_new_tokens2048、新建每请求 cache 和相同 prompt（包括 Schema 文本）。原始 outputText、candidate、逐 token 到达时间与失败保留。
- 请求计时在构建约束解析器前开始，至 generate 和 CUDA synchronize 后结束。模型/Tokenizer 加载在请求外；parserBuildMs 单列。TTFT 为 streamer 首个生成 token 到达（跳过 pinned Transformers 的 prompt put）；结束 token 也计数。生成速率用 (N-1)/(末 token 时刻-首 token 时刻)，不混入首 token。另列总 token/总时间。
- 顶层 forward 的 observer pre/post hook 只观测首轮 prefill：wall 时间和 CUDA Event stream span（不是 kernel 时间之和）。两臂相同观测；它们有同步与 CPU streamer 转移开销，不能与未加观测器的原 200 条严格等同。observer hook 不修改激活，和 m=0 无调制 hook 并不冲突。
- constraintCallbackWallMs 包含 Python/词表/CPU-GPU 往返及可能的设备等待，不是纯 LMFE 算法时间。配对完整输出的 token/s 比较之外，另按两臂共同的前 min(Nu,Nc) 个 token 时间比较；长度受控但内容仍可不同，因此仅作诊断。
- 由原始 Schema AJV 和来源绑定再次核验输出；原 8 条日文 JSON 失败及169/200来源合格率仍保留。无约束臂不是新后端候选，不据此改验收或正式配置。
- 安静窗口需 Runner.Listener/Runner.Worker/Blender 均无、P4-01/P4-02 测量无；加载前和每臂前重复进程检查。失败关闭，不主动停用户进程。窗口内不运行检查或其他 GPU 工作。

## 决定边界

backend/P90 的机械规则输出不可部署，操作字段为 null。P0-09 决定架构与延迟目标，四个候选方向见 `review-r1/OPTIONS.md`。A/B/C 从用户说完到可听见回复的 P50/P90 继续 null；不相加边际百分位，待 P0-09 汇总 P4 数据。helpers/models lock/正式 profile 不改。无新增运行时依赖。

## API 核对

Context7 月度额度不可用后，核对官方文档和本机锁定版本源码：
- https://learn.chatgpt.com/docs/app-server
- https://docs.pytorch.org/docs/2.10/generated/torch.cuda.Event.html
- https://docs.pytorch.org/docs/2.10/generated/torch.nn.Module.html
- https://huggingface.co/docs/transformers/v4.57.3/en/internal/generation_utils

Transformers 4.57.3 本地 generation/utils.py 的 prompt streamer 与 next-token streamer 路径已核对。原源码精确 gzip 和新 local-source-archives 绑定用于重算。

## 严格工具面修复后的第三组

`part2/low-effort/` 原配置补测有1条IRL工具违规，不作为合格的严格无工具组。新 `part2/low-effort-isolated/` 独立完整重测，不删除或替换旧行。修复：子进程删除继承的 CODEX_* 应用上下文及 OPENAI_API_KEY/OPENAI_BASE_URL 覆盖，显式绑定同一隔离 CODEX_HOME；关闭 orchestrator.mcp、本地 plugins、cloud.skills、skill_mcp_dependency_install、native collaboration。

模型目录v3在v2基础上只将Luna multi_agent_version设为disabled，模型ID/推理档不换。`review-r1/catalog-v2-v3-diff.json` 核验只有这一Luna字段不同，其他模型目录相同。配置字段依据同版生成的MultiAgentVersion类型及官方Codex配置/工具路由源码；没有复制源码或引入依赖。

资源能力探针的失败历史表明，初始化时MCP=0并不可靠：原配置会在首轮后变为3。仅关闭orchestrator或继承上下文不足；本地插件另有开关，模型目录的协作版本也能独立启用工具。最终 `resource-surface-audit-v3.json` 2/2完成、零调用且轮后零MCP。它不能证明所有未来输入都安全，故正式新组每条都记录 appContextIsolated、mcpServerCountAfter、mcpHasNextPageAfter；计时外查询库存，有工具调用、非零或未知库存就保存原行和partial summary后中止。语音及IRL都使用同一关闭检查。

**比较限制：** 原max350条虽无观察到的工具调用，只能作为用户指定的行为对照，不能据此声称能力面完全不存在。原配置low与max的描述性比较仍保留；修复后的low减少了注入工具上下文，不能将其token/速度差异全部归因于effort。报告三组分列，绝不混合百分位。

精确新源码与辅助模块绑定在 `review-r1/low-isolated-plan.json`；原始max和旧low的源码快照仍各自保留。
