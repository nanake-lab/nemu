重档：本 PR 涉及 executors/，按 AGENTS.md §6 保留完整验证与独立交叉审查要求。

# P5-07 IRL worker 与 codex-bridge 基础层 — 报告

## 当前状态

2026-11-02 增量修复：审查提出的三项 bridge 阻塞均已按先复现后修复处理，待独立复审；全工作区 TS 620 项、Python 28 项及静态检查/编译通过，详见 [R1 修复报告](review-r1-20261102/REPORT.md)。

**部分交付。** 2026-11-01 用户明确批准 #96 限定为可独立审查的 worker／bridge 基础层；本机基础层验收已通过，提交供交叉审查。没有生产接线，**P5-07 整卡未完成**。命名管道合同、正式模型测量与三语冻结人工标注验收仍须在本卡名下补齐，并增量复审，见 [BLOCKED.md](BLOCKED.md)。本次不合并、不取下一张卡。

原目录占用已随 #95/#108 合并解除。ADR 0031 继续要求原 wire schema、本地、异步、不挡回复；ADR 0033 已合入 main，本次未设置截止时间或保险丝阈值。最新 ADR 0031/0034 条件性载体决定继续由后继正式测量落实。

看板部分交付支持已由 #129 合入 main（8fed313b）；本报告「当前状态」和仍未完成的 BLOCKED 保持部分交付，不能据基础层通过宣称整卡完成。当前只请求复审，不合并。

## 交付物

- [IRL worker](../../workers/irl/src/nemu_irl/worker.py)：进程内异步候选库，读取生成类型的 UserText、当前事件、c_core 和输入绑定；保留实际 UTF-8 字节、原 wire 解析候选、生产者/版本及绑定副本。外部输入校验、来源与认识论升级拒绝、字节/深度/集合边界、失败/取消保留已收字节，不自动重试。
- 原 P0-08 wire schema 原样打包，按字节核对；每次模型请求使用 schema 要求的 `p0-08-0000` 别名，真实事件身份单独保留在绑定。它不是正式 IPC、Concept Registry 映射或 accepted z。没有把自由标签编造成 SemanticGraph。
- [bridge 基础库](../../executors/codex-bridge/src/base/README.md)：独占 home、锁定 CLI 版本、隔离启动参数、有限 JSON-RPC、线程/轮次/流式控制、零工具/零 MCP 守卫、按档位用量、断线不盲重发。45 份原样生成类型保留；未注册服务。
- [IPC 后继请求](../../tasks/requests/P5-07-ipc-contract.md)进入契约线排队；[模型与人工金标准请求](../../tasks/requests/P5-07-model-and-human-gold.md)单列。人工标注需要用户参与，执行者不代标、不降低要求。

实现只有一个异步函数和必要数据记录，没有后端注册表、队列、临时管道或为后继预留的策略层。`LocalInference` 只是离线测试用的函数签名；真实本地 m=0/受约束解码由 P5-13 提供。当前只做 UserText 预检，c_core 对象解析、语音等其他载荷与正式候选映射随合同补齐。

## 验收（2026-11-01 基础层；最新增量见 R1 报告）

全部使用合成输入、黄金 DTO 与临时目录，不使用用户存储或推理额度。命令、环境、结果与被测源码摘要见 [本次核验索引](resume-20261101/acceptance.json)。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint`、`pnpm typecheck` | passed | [TS 结果](resume-20261101/final/ts-results.json) |
| `pnpm test` | passed：54 文件、598 项 | [日志](resume-20261101/final/test.log) |
| `pnpm exec tsc -p executors/codex-bridge/tsconfig.json --outDir D:/Dev/cache/P5-07/bridge-dist-20261101` | passed | [日志](resume-20261101/final/bridge-build.log) |
| worker：`uv run --offline ruff check .`、`uv run --offline mypy --strict src tests` | passed | [结果](resume-20261101/python-final/results.json) |
| worker：`uv run --offline pytest` | passed：28 项 | [日志](resume-20261101/python-final/pytest.log) |
| worker：`uv build --offline --out-dir D:/Dev/cache/P5-07/dist-complete-20261101` | passed | [日志](resume-20261101/python-complete/build.log) |
| wheel 安装后导入、源码/生成 DTO/资源完整性、原 wire schema 字节相等、bridge 生成类型 | passed，45/45 生成文件匹配 | [核验](resume-20261101/artifact-check.json)、[安装](resume-20261101/wheel-install.log) |
| 范围、Markdown 链接、JSON/TOML 解析、diff 空白检查 | passed | [范围核验](resume-20261101/scope-check.json) |
| SYS02 | 完整联测 not_run；只有局部正反用例通过 | [本次范围](SYS02/resume-20261101.json) |
| 正式管道、真实服务/双后端与 provider 录制 | not_run | [阻塞与选项](BLOCKED.md) |
| 三语各 100 条、schema ≥95%、冻结人工语义准确率、混淆矩阵/语言差、模型 P50/P90 | not_run | [本次指标状态](resume-20261101/metrics.json) |

**main 增量核对：** 旧 TS 验证源码 head 为 `6e8fb8c5547560673d1dc508b724b7f7be50f8e4`（[原记录](verification/final-bytes/test.json)），实际 main 基线 B=`2bc04bad4902657eef49d3225cafa7945e7cfae6`。本次 fetch 固定 M=`acc71dadc5ba8982392d0aded64b9bf6772b514a`，确认 B 是 M 祖先并检查新增提交/文件范围。IDL、Python 生成代码、黄金向量及任务/架构要求有相关变化，因此已 merge M（`a176c281df0bd0abc44b093ad701280afeaf0922`），并在新组合上重跑上述 Python/TS 检查。[影响记录](resume-20261101/main-impact.json)及 [源码 SHA256](resume-20261101/artifact-check.json)绑定当前未提交时实际被测字节；之后仅补报告和 README。本 PR 不改或链接 Rust 实现，不把其独立验收算成本 PR 验收。

**失败记录：** 本次首轮 mypy 因局部变量复用产生 1 项类型错误，改名后通过（[原日志](resume-20261101/attempt-01/mypy.log)）；补充深度测试有 1 项失败，原夹具深度恰好等于上限，改为真实超限的合法 Claim 并保留正向用例后通过，未改上限或放宽断言（[原日志](resume-20261101/python-complete/pytest.log)）。直接以 zipimport 载入 wheel 的额外探针失败：生成运行时要求安装后的文件路径；按正常 wheel 安装后导入及资源核对通过（[原输出](resume-20261101/wheel-zipimport-failed.log)）。历史失败仍保留：旧候选 pytest 误从根目录收集导致 6 项错误、mypy 8 项注解错误（[旧目录](verification/attempt-01/)）；离线安装曾缺 koffi 缓存，同锁版本安装后解决；旧 PowerShell 编译调用未 splat（[原日志](verification/final/bridge-build-post-main.log)）。封存旧 worker 的 23 项通过不计入本次验收。

## 规格对照

| 规格条款 | 实现位置 | 测试与限制 |
| --- | --- | --- |
| ADR 0031 §4 | `worker.py`、打包 wire schema | 字节一致；IRL 等待时独立回复可完成；不证明生产调度 |
| F §3.3/3.5、D 第一篇 §4.2/5.3 | `interpret`、`_check_sources` | 三语结构、Ref/Claim 原标签、来源/跨度、工具/系统标签升级与 inferred 升级拒绝；非语义准确率 |
| v2.0 §2.4、EV6、I23 | 实际字节与候选记录 | 完整/失败/溢出/取消均保留实际已收字节；m=0 需正式端点验证，未签发证明 |
| I14、输入绑定 | 生成 DTO 校验、payload/c_core 摘要核对 | 事件、事实及载荷错配不调用推理；输入绑定快照不随调用方修改 |
| ADR 0004 §1、P0-08 复审隔离要求 | `home.ts`、`process.ts`、`isolation.ts`、`bridge.ts` | 版本、独占锁、模型与零工具/零 MCP 守卫；合成协议帧，不是 provider 录制或 OS 隔离验收 |
| ADR 0004 §3–5、CONTRACTS §6、I16 | `rpc.ts`、`Bridge.run/usage` | 请求不自动批准；有限帧/截止时间、控制命令、用量、断线与额度耗尽失败，不盲重发 |
| I02、I18、I29 | 无权威存储接口，候选由调用方持有 | 临时存储与合成输入；用量内存记录限容且不含输出原文，无隐藏持久状态 |

本 PR 不修改 IDL、黄金向量、差异登记或 ADR；规格对照在报告内，以遵守用户目录范围。手写源码/测试均小于 400 行；生成文件原样保留。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| zod（仓库既有同版，新增根直接依赖） | 4.6.7 | MIT | bridge 输入校验 |
| jsonschema | 4.26.0 | MIT | 与 P0-08 同版的原 wire schema 校验 |
| attrs | 26.1.0 | MIT | jsonschema 传递依赖 |
| jsonschema-specifications | 2025.9.1 | MIT | 同上 |
| referencing | 0.37.0 | MIT | 同上 |
| rpds-py | 2026.9.1 | MIT | 同上 |
| typing-extensions（既有锁版本） | 4.16.0 | PSF-2.0 | referencing 的运行时传递依赖 |

许可证取已安装包元数据，见 [依赖核验](resume-20261101/runtime-dependencies.json)。工具为 Python 3.12.14、Node 24.21.0、pnpm 12.11.2、uv 0.12.15；未升级 Codex CLI。

## 待用户确认与下一步

- 本次基础层范围已获用户批准，无待补的范围批准；需用户安排独立交叉审查（GPT-6 Astra，最高推理强度）。执行者在 PR 写「请审查」，不替审查方下结论。
- 整卡仍阻塞于正式合同/接线和人工金标准/正式测量。人工标注需要用户参与；后继完成后在 P5-07 名下补验收、增量复审，不以本部分通过代替。
- P6-04/P6-06 复用时分别接自己的权限域；talk-only 策略不能替代助手行动闸门。