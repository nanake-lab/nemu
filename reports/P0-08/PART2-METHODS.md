# P0-08 第二部分测量口径与限制

## 分组与可追溯性

- 用户指定远程 `gpt-6-luna` / `max`，覆盖卡片原来的低推理强度；不混入其他模型或强度。
- `part2/strict-speech/` 是最终远程组，包含三语各 50 条短回答和 200 条 IRL。该目录名称沿用启动时的名称；IRL 也在其中。
- `part2/raw/`、`part2/candidates/` 的初始远程组保留为历史对照。初始能力配置没有彻底移除 code-mode 工具，不能作为“只能说话”的最终证据。
- `part2/local-irl-20261022-v1/` 是因解码器缺陷中止的实现版本：10 条正式返回（4 合格、6 失败）及 1 条预热。没有把它们改成成功，也不混进 v2 的 200 条。
- `part2/local-irl-20261022-v2/` 是最终本地对照，1 条预热排除，200 条正式返回。失败仍计入合格率分母；没有生成后修补 JSON。
- `protocol/source-archives.json` 登记源码；`.gz` 保留当时捕获的原始字节，可读副本规范化换行和尾空白。不要从可读副本覆盖原始压缩包。日文旧组补测 28–36 未单独捕获启动源码，这个缺口已在旧组 manifest 明示。

## 远程能力配置

仅关闭 shell/apps 等功能标志还不够：对抗性时钟/文件请求在旧配置下出现 raw `exec`，而 `item/started` 没有工具项。因此不能用旧组的工具项计数 0 证明没有调用工具。

修正版通过 Codex CLI 0.162.1 支持的 `model_catalog_json`，仅把 Luna 的 `tool_mode` 改为 `direct`，移除 apply-patch、实验工具和搜索能力声明。模型 ID、推理强度和其余模型元数据不变；整份供应商目录保留在 Git 外。启动时关闭 apps/plugin/hooks，线程中关闭余下内置工具并传空 environment/capability roots。工作目录为新临时目录、只读、无网络、无动态工具、`personality: none`、样例协议 p。

修正版对抗探针没有产生 raw 工具调用或审批请求，没有创建文件；最终启动目录查询显示 0 个 MCP server。每条正式返回额外捕获 raw response-item 工具调用名称。最终分析要求这项证据存在且为空。这些观测适用于这份测量配置，不是对所有模型行为的安全证明。CLI 版本、生成类型、目录差异及哈希均在证据中；不读取登录凭据。

配置依据是该版本官方源码与文档：

- [Codex app-server](https://learn.chatgpt.com/docs/app-server)
- [0.162.1 配置 schema](https://github.com/openai/codex/blob/rust-v0.162.1/codex-rs/core/config.schema.json)
- [0.162.1 工具声明](https://github.com/openai/codex/blob/rust-v0.162.1/codex-rs/core/src/tools/spec_plan.rs)
- [Structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs)

## Schema 和来源校验

原始候选 Schema 的 `oneOf` 被远程严格接口拒绝。wire schema 展开 `allOf`、将互斥节点种类改成 `anyOf`，并用接口支持的数组形式表达 span。两种后端使用相同 wire schema，返回后仍校验原始 schema（含 tuple 和唯一性），没有修改正式契约或阈值。解析器兼容去除 Markdown JSON 围栏；输出结构合格不代表语义正确。

来源检查另计：输入 sourceRef、节点 ID 唯一性、sourceRefs 和 Unicode 码点 span 边界。它不是金标准语义评分，不能声称事实正确、消歧正确或没有幻觉。

## 本地配置与失败

载体为 P0-07/ADR 0012 已选定的 `Qwen/Qwen3-4B-Instruct-2507`，修订 `cdbee75f17c01a7cc42f958dc650907174af0554`；NF4、bf16 compute、无 double quant、eager attention，Torch 2.10.0+cu128 / Transformers 4.57.3 / bitsandbytes 0.50.2，145 个 native RMS（`aten::_fused_rms_norm`）。m=0 基准按 ADR 0012 使用同一配置不装 hook；不是跨内核比较。权重分片哈希已核对，原环境只读复用。

受约束解码使用 lm-format-enforcer 0.11.3。v1 中 literal TAB 等控制字符可以经过 tokenizer alphabet/free-string cache；CPU 回归复现后，v2 在内存中过滤这些字符和 token，不修改第三方安装文件。v2 仍有 8 条日文 JSON 解析失败：30、51、90、96、111、150、171、186。它们没有达到 2048-token 上限；不能断言约束库与该 tokenizer 的所有 Unicode/token 交互已修复。原始 malformed 输出与哈希保留；合格率为 192/200，来源检查 169/200。

GPU 窗口获用户授权；开始及 v2 重启前确认 Runner.Listener、Runner.Worker、Blender 均不存在，P4-01/P4-02 没有测量，模型逐条重复进程检查。RTX 3080 10 GB，驱动 610.88。窗口内不运行其他测试/GPU 工作；结束后已通知用户恢复 runner。桌面 DWM 等基础负载并非零，初始显存约 1045 MiB/利用率 9%，记录在案。

## 时间与分母

- 首增量：发送 `turn/start` 前计时，到第一个 `item/agentMessage/delta`；不是首个可播放音频。
- 远程 IRL：同一开始点到 `turn/completed`，临时线程创建不计入。包含远端推理、网络与客户端接收开销。
- 本地 IRL：模型和 tokenizer 已加载后，计约束 parser 构造与同步 generation，排除 tokenization、结果 decode/校验；这与远程路径边界并非完全相同，是本次卡片的请求级对照，不是整套应用性能保证。
- 百分位：nearest rank（排序后 `ceil(p*n)-1`）。后端判定使用预登记的所有已完成生成的完成 P50，包括 schema 失败；另列 schema 合格子集，不偷换样本。传输超时保留且视为删失，不填一个有限完成延迟；失败仍在合格率分母。
- 两者都不满足 400 ms/95% 时按完成 P50 选较快者，截止时间采用该组完成 P90。样本不足以证明两种后端总体分布有显著差异。
- 额度使用账号显示快照；共享账号可能有其他活动、百分比取整或窗口重置，不能把变化全部归因给本卡或当成计费单价。
- A/B/C 的端到端说话延迟还缺 VAD/ASR、A/M 与核心事务、TTS/缓冲及 C 起草改写计时。保持 null；不把边际 P50/P90 相加成端到端百分位。

## 资源、范围与依赖

本卡使用临时目录与合成输入，不打开正式/排练存储；所有审批请求均拒绝。原始模型目录完整文本不入库；IRL 合成候选单列，不记录用户完整对话。

新增 spike 运行依赖：lm-format-enforcer 0.11.3（MIT）、jsonschema 4.26.0（MIT）；开发检查 mypy 1.20.2（MIT）、ruff 0.15.6（MIT）。mypy 的传递开发依赖 pathspec 1.1.1 为 MPL-2.0，仅用于检查，不进入应用运行时。详见 `part2/protocol/python-dependencies.json` 和 `dev-license-evidence.json`。AJV 8.20.0（MIT）是已有工作区依赖；P0-07 CUDA 依赖沿用已有环境。

正式 helpers.json、models.lock.json、部署 profile 和契约不在本卡修改范围。候选结果只写入 spike 草稿；正式配置按 `tasks/requests/P0-08-config-scope.md` 走 ADR。`probe.mjs` 超过 400 行，保留集中式 JSON-RPC 生命周期/事件相关/计时/审批处理，已捕获每版测量源码；本卡不在测量中途重构改变时序。
