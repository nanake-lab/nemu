> 复审修订提示（2026-10-22）：下文为原 max 对照与本地 200 条的历史结果。max 是用户明确指定的配置。当前补测与修订口径见 REVIEW-R1-STATUS.md、REVIEW-R1-METHODS.md；部署结论以 decision.md 为准，所有机械后端/P90 输出均不可部署。

# P0-08 Codex app-server 与 IRL 路径验证：第二部分 — 报告

## 结果

技术测量已完成；整卡仍有 P0-09 用户确认门槛。最终远程三语 150 条、远程 IRL 200 条、本地 m=0 IRL 200 条及控制/审批探针均已运行，原始记录保留。按双未达标回退规则选本地，截止时间候选 26168.09769999236 ms；不声称达到 400 ms 目标。

PR #50 继续部分完成后合并的 #29。#29 已加 unverified，第二部分经 Claude 审查后再去除。本次仍按用户明确要求提交 Claude 审查，即使最新 main 已取消一般中等难度卡的必需审查。未合并 PR #50。

## 交付物

- `spikes/codex-app-server/src/`：隔离 app-server 探针、能力审计、严格目录生成器、m=0 受约束解码、证据复算与 CPU 测试。
- `spikes/codex-app-server/drafts/`：目录已核对的 helpers 档位草稿、后端/截止时间草稿；日预算和模式 A/B/C 端到端延迟为 null。
- `benchmark.md`、`decision.md`：当前结果在前，第一部分原文保留为历史。
- `part2/strict-speech/`：最终 150+200 条原始 JSONL、候选与额度。
- `part2/local-irl-20261022-v2/`：最终 200 条及预热；`local-irl-20261022-v1/` 保留中止版本。
- `part2/protocol/`：环境、GPU/进程检查、源码 gzip/哈希、模型目录差异、版本绑定。
- `PART2-METHODS.md`：时间边界、失败、历史分组、来源检查和推论限制；`BLOCKED.md`：已解除项与剩余门槛。
- 根 package.json 只追加 probe:p0-08、probe:p0-08:local、probe:p0-08:analyze 三个命令。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 独立 CODEX_HOME 登录、状态、Git 状态 | 已完成；未读取认证文件/登录页面，未将凭据放入仓库 | 隔离 profile；初始授权后 Git clean；后续仅提交白名单目录 |
| CLI / 同版 generate-ts | 0.162.1；888 个文件，确定性清单哈希 4bbc53913e6df329442b62b503be5a1343c73d9f8c239328a0b34d87679ac409 | drafts/cli-protocol-candidate.json；protocol/cli-runtime.json、strict-irl-runtime.json |
| 模型目录 / 三档核对 | 7 个模型；Luna、Sol、Astra 三档存在 | part2/model-catalog.json、strict-speech/model-catalog.json |
| 最终只能说话线程，三语各 50 条 | 150/150；每条 raw 工具调用捕获为空，未改道 | strict-speech/raw/、analysis.json |
| 最终远程 IRL 200 条 | 200/200 完成与 Schema 合格；来源检查 195/200 | strict-speech/candidates/、raw/、analysis.json |
| 本地 m=0 IRL 200 条 | 200 条完成，Schema 192/200、来源检查 169/200；8 条解析失败原文保留 | local-irl-20261022-v2/ |
| interrupt / steer | 请求接受；interrupted / 标记结尾均观测到 | controls-untrusted.json |
| 命令与文件审批 | 两种实际请求均交到客户端、均 decline；未创建目标文件 | controls-untrusted.json |
| 100 次短回答 / 100 次 IRL 额度 | 已记录账号开始/100/整组快照 | 各 summary、quota-before/after100.json、analysis.json |
| 用户 GPU 窗口与进程检查 | 两次前置无 runner/Blender/P4 测量；逐条复查；结束已通知恢复 runner | protocol/gpu-window*.json、p4-window-state.json、local quiet-window.json |
| node analyze.mjs（主线合并后） | 退出 0；独立重校候选、来源、覆盖与源码哈希 | checks/post-merge-analysis.log |
| 最新 main 合并 | 已 merge bba68cc；P0-08 卡、必读架构/契约、载体配置没有变化 | checks/main-sync.log；merge b1900b6 |
| pnpm lint / typecheck / test（合并后） | 退出 0；29 文件、443 项通过 | checks/final-after-main/01–03 日志 |
| Node 单测 / 语法 / help（合并后） | 7 项通过，10 个 mjs 及 help 退出 0 | checks/final-after-main/04、08、09 日志 |
| Python CPU / Ruff / strict mypy（合并后） | 10 项通过；Ruff 通过，4 文件 mypy 无问题 | checks/final-python-native/ |
| CI | 以 PR #50 最新检查为准；草稿阶段 Windows 默认跳过 | GitHub PR checks |
| P0-09 用户确认 | 不阻塞，移交 P0-09；本卡测量与报告已完成 | decision.md、BLOCKED.md |

## 规格对照

| 条款 | 实现位置 | 验证 |
| --- | --- | --- |
| ADR 0003：仅说话、隔离能力 | probe.mjs、prepare-talk-catalog.mjs | 最终启动 0 MCP、对抗审计、350 条 raw 捕获；不夸大为通用安全证明 |
| ADR 0004：独立 profile、目录与版本 | probe.mjs、hash-generated.mjs、候选 drafts | 登录/目录/CLI 二进制绑定；未改正式配置 |
| ADR 0012：选定载体零参考 | local_irl.py、native_norm.py 只读复用 | 同载体 NF4/bf16/native RMS，无 hook，m=0；绑定模型修订、权重与源码 |
| 卡片阈值与回退规则 | analyze.mjs | 400 ms/95% 阈值未改；所有完成生成的 P50 比较，所选 P90 写草稿 |
| 架构完成标准第 8 项 | decision.md | 文本首增量不充当可听见回复；缺失 A/B/C 值为 null |

本卡不实现 Echo Core 算子、正式类型或存储，不以实验 Schema 替代正式契约。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| lm-format-enforcer | 0.11.3 | MIT | spike 受约束解码 |
| jsonschema | 4.26.0 | MIT | 原始候选 Schema 校验 |
| mypy | 1.20.2 | MIT | 开发静态检查 |
| ruff | 0.15.6 | MIT | 开发静态检查 |

传递依赖的名称/版本/许可在 protocol/python-dependencies.json、dev-license-evidence.json。pathspec 1.1.1（MPL-2.0）仅为开发检查传递依赖，不进入应用运行时。已有 AJV 8.20.0（MIT）和 P0-07 CUDA 环境只读复用，不改其他工作树。probe.mjs 超 400 行的原因及模型/tokenizer 的未解决限制见 PART2-METHODS.md。

## 待用户确认

- [ ] （不阻塞，移交 P0-09）确认后端、截止时间及最终延迟目标；A/B/C 完整链未测，保持 null。
- [ ] （本次交付审查门槛）Claude 审查第二部分及 #29 历史未核验内容；审查后决定 #29 标签去除。
- [ ] （正式配置后续门槛）按既有范围请求走 ADR；本 PR 不修改 helpers.json、models.lock.json 或正式部署 profile。

## 后续建议（不在本任务范围内）

- 完整 VAD/ASR→核心→说话→TTS/音频链联合测量；不要相加独立边际百分位当作实测。
- 进一步处理约束库/tokenizer 的 Unicode 交互，建立语义金标准评分；不把当前来源检查率称为语义准确率。
- 若需稳健比较小于 0.3 秒的中位数差，另做多批次/交错顺序实验；本卡不扩大测量范围。

## 下一步

合并后本地检查已通过；等待 PR CI、Claude 审查及 P0-09 用户确认。按用户指令不自行合并，不取另一张卡；#29 的 unverified 在审查通过前保持。

## 检查启动器诊断

final-after-main 的三项 Python 命令先在 CMD 解析正斜杠路径时失败，Python 未启动（含 Windows 编码乱码的 stderr 保留）。主代理诊断后指定原生 PowerShell 绝对路径调用，在 final-python-native/ 补跑三项并通过；没有改测试或实现，也没有覆盖初次失败。

## 最新草稿门槛

origin/main bba68cc 的 AGENTS.md 第 2、6 节新增“部分完成/等用户决定保持草稿”。P0-09 仍待决定，因此当前保持草稿；若用户确认本次按原第 7 步先转可审查供 Claude 审查，再执行该状态变更。Claude 可先审查草稿中的完整技术证据。
