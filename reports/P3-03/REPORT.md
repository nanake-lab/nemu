# P3-03 身体读出与待机生命 (Readout & Idle Life) — 报告

## 结果
完成。实现了身体读出应用器（`ReadoutApplicator`，支持严格数值限位、中性值回退、应用幂等性及 2 秒固定平滑过渡）与待机调度器（`IdleScheduler`，支持权重随机、冷却时间约束、连续 3 次防重复、长为 3 或以上的重复子序列判定与规避、`idleTempo` 节奏缩放及可注入随机源）。评审台已全面接入该应用器与调度器。全套单元测试、结构防隐藏状态扫描及 10 分钟仿真采样全部通过。

## 交付物
- `packages/body/src/readout/types.ts`：读出模块类型定义（连接状态、应用器配置、表情基线映射）。
- `packages/body/src/readout/constants.ts`：中性读出常量 `NEUTRAL_BODY_READOUT`（满足 CONTRACTS.md 第 7 节中性定义）及各项边界常数。
- `packages/body/src/readout/applicator.ts`：`ReadoutApplicator` 核心实现，负责读出输入清洗、边界截断、断开或隔离状态回退、幂等性判断以及 2.0s 平滑插值动画驱动。
- `packages/body/src/readout/readout.test.ts`：读出边界值校验、中性回退测试、幂等性测试、2 秒动画曲线测试及客户端防隐藏情绪/好感状态 AST/文件结构扫描测试（全部通过）。
- `packages/body/src/idle/types.ts`：待机动作项 `IdleSetItem` 与调度器配置定义。
- `packages/body/src/idle/scheduler.ts`：`IdleScheduler` 核心实现与 `hasConsecutiveSubsequenceRepeat` 重复子序列检测器。
- `packages/body/src/idle/idle.test.ts`：连续子序列检测测试、冷却约束测试、连续 3 次禁止相同测试、`idleTempo` 频率调制测试、确定性 PRNG 验证以及 10 分钟（600 秒 60 FPS）待机采样验证（全部通过）。
- `packages/body/src/index.ts`：`@nemu/body` 模块统一入口导出。
- `apps/stage/src/review/review-app.ts`：评审台接入 `ReadoutApplicator` 与 `IdleScheduler`。
- `apps/stage/tests/visual/p3-03-readout-idle-e2e.mjs`：端到端 10 分钟待机仿真数据采样与评审台两组（松弛 Casual 与敏锐 Alert）3 分钟待机观察快照生成套件。
- `reports/P3-03/10min_idle_sample.json`：600 秒 60 FPS 采样生成的 85 次待机动作完整时间戳及序列证据（验证无长 ≥ 3 的连续重复子序列，无连续 3 次相同动作）。
- `reports/P3-03/readout_A_relaxed/`：组 A（松弛 Casual 待机读出）3 分钟阶段采样截图（0s, 30s, 60s, 90s, 120s, 150s, 180s）。
- `reports/P3-03/readout_B_alert/`：组 B（敏锐 Alert 待机读出）3 分钟阶段采样截图（0s, 30s, 60s, 90s, 120s, 150s, 180s）。

## 验收
| 命令或检查（含任务卡验收要求） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 (0 warnings, 0 errors) | CI & 本地全绿 |
| `pnpm typecheck` | 通过 (0 errors) | TypeScript 5.8 严格检查无报错 |
| `pnpm test` (包含 `readout.test.ts` 与 `idle.test.ts`) | 通过 (16 suites, 374 tests) | 17/17 P3-03 单元测试通过，全工作区测试 374/374 通过 |
| 读出映射的边界值测试 | 通过 | 超出范围值被严格截断：`posture` ∈ [-1, 1], `idleTempo` ∈ [0.5, 1.5], `earPose` ∈ [-1, 1], `tailAmplitude` ∈ [0, 1], `gazeSociability` ∈ [0, 1], `expressionBaseline` ∈ [0, 0.4] |
| 中性读出得到中性姿态测试 | 通过 | `NEUTRAL_BODY_READOUT` 及断开/隔离状态输出中性：`posture: 0`, `idleTempo: 1.0`, `earPose: 0`, `tailAmplitude: 0.5`, `gazeSociability: 0.5`, `baseline: {}` |
| 应用的幂等性测试 | 通过 | 连续调用相同读出不会重新触发过渡、不产生累加或状态漂移（返回 `false` 且插值平滑继续） |
| 结构检查（无情绪/好感持久变量） | 通过 | 扫描 `packages/body`, `packages/avatar-runtime`, `apps/stage` 源码，未发现任何持久化存储好感度、心情、亲密度的违规变量（满足黄金规则 4 与不变量 I18） |
| 10 分钟待机采样（长度 ≥ 3 重复子序列检查） | 通过 | 600s 仿真触发 85 次小动作，`hasConsecutiveSubsequenceRepeat(history, 3) === false`，连续 3 次无相同动作，所有动作间隔均 ≥ 对应冷却时间 |
| 评审台两组 3 分钟待机视觉采样 | 通过 | 产出 `reports/P3-03/readout_A_relaxed/` 与 `reports/P3-03/readout_B_alert/` 共 14 张时间序列截图证据 |

## 规格对照
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| CONTRACTS.md 7: `BodyReadout` 结构与取值范围 | `packages/body/src/readout/constants.ts` | `packages/body/src/readout/readout.test.ts` |
| CONTRACTS.md 7: 读出平滑（约 2 秒动画插值） | `packages/body/src/readout/applicator.ts` (`smoothingDurationSec = 2.0`) | `packages/body/src/readout/readout.test.ts` (`smoothing transitions over 2s`) |
| CONTRACTS.md 7: 核心未连接/隔离使用中性值 | `packages/body/src/readout/applicator.ts` (`applyReadout(..., status)`) | `packages/body/src/readout/readout.test.ts` (`resets to neutral on quarantine/disconnect`) |
| AGENTS.md 黄金规则 4: 终端不得保存情绪好感等隐藏持久状态 | `packages/body/src/readout/applicator.ts`（只应用、不持久化） | `packages/body/src/readout/readout.test.ts` (`structural check`) |
| P3-03: 待机随机抽取、冷却与防连续 3 次相同 | `packages/body/src/idle/scheduler.ts` | `packages/body/src/idle/idle.test.ts` |
| P3-03 / 完成标准 6: 连续重复子序列长度 ≥ 3 判定与规避 | `packages/body/src/idle/scheduler.ts` (`hasConsecutiveSubsequenceRepeat`) | `packages/body/src/idle/idle.test.ts` & `10min_idle_sample.json` |
| P3-03: `idleTempo` 调制动作频率 | `packages/body/src/idle/scheduler.ts` (`interval / idleTempo`) | `packages/body/src/idle/idle.test.ts` (`modulates trigger frequency`) |

## 新增依赖
无新增外部生产运行时依赖（基于项目中已有的工作区包和工具链实现）。

## 待用户确认
- [ ] （不阻塞）两组 3 分钟待机视觉序列（松弛与敏锐）中呼吸与耳尾摆动幅度是否符合预期审美，后续可在 `profile.json` 中微调。

## 后续建议（不在本任务范围内）
1. 在 P3-04 中将 `IdleScheduler` 与 XState 物理状态机的 `idle` 状态转换深度绑定（在进入 `idle` 时激活调度，在 `dragged`、`listening`、`speaking` 时暂停待机随机小动作调度）。
2. 在 P5-17 核心身体读出算子就绪后，直接通过应用内总线打通 `stage.readout` 真实传输链条。

## 下一步
本任务已全部通过，无阻塞项。创建 PR 并合并至 `main`。
后续依照任务路线图取卡：`P4-04 口型生成与面部通道 (Lip Sync & Face Channel)`。
