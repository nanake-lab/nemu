# P1-04 调试面板 — 报告

## 结果
完成。在 `apps/panel/src/debug/` 中完整实现了事件流查看器、核心状态查看器、延迟瀑布图面板、帧统计、手动发送舞台命令（带实时 JSON 语法及 `@nemu/contracts` 契约校验）以及开发者模式显隐门控；Playwright e2e 验收测试与单元/集成测试全部通过。

## 交付物
- `apps/panel/src/debug/types.ts`：调试面板数据类型与状态模型定义（事件项、核心运行状态、延迟统计指标、帧统计）。
- `apps/panel/src/debug/ulid.ts`：浏览器端环境安全生成的 ULID 工具函数，严格符合 Crockford Base32 编码及 zod `z.ulid()` 校验规范。
- `apps/panel/src/debug/bus-client.ts`：面板总线适配客户端，监听 `nemu:channel:bus` 端口，提供订阅与分发能力，支持向总线/舞台派发命令并在事件流中即时留痕。
- `apps/panel/src/debug/schema-validator.ts`：基于 `@nemu/contracts` 中 `controlCommandPayloads` 的舞台命令校验引擎，提供各命令预设模板与实时错误解析。
- `apps/panel/src/debug/EventStreamViewer.tsx`：事件流查看器，支持按来源（main/stage/panel/voice）及类型筛选、流暂停与恢复、清空及单条事件 JSON 详情查看。
- `apps/panel/src/debug/CoreStatusViewer.tsx`：核心状态查看器，未收到核心事件时默认展示「未连接」（P6-02 合并前标准），收到事件后展示运行状态、谱系（排练/正式）、说话模式、严格标志、自主调度开关、最近回执（RECEIVED_DURABLE、CORE_COMMITTED）及身体读出。
- `apps/panel/src/debug/LatencyPanel.tsx`：延迟面板，按 `utteranceKey` 展示各处理阶段的瀑布图（VAD、ASR、核心提交、首块生成、TTS、音频启动），并统计最近 50 次请求的 p50 与 p90 延迟。
- `apps/panel/src/debug/FrameStatsViewer.tsx`：帧统计面板，展示渲染 fps、p95 帧时间及当前物理状态。
- `apps/panel/src/debug/CommandSender.tsx`：手动发送舞台命令组件，内置常用命令预设、JSON 编辑区、实时 schema 校验与一键发送。
- `apps/panel/src/debug/DebugPanel.tsx`：调试面板根组件，提供 Tab 导航切换（概览、事件流、核心状态、延迟瀑布、帧统计、发送命令）与开发者模式显隐门控（生产环境隐藏，支持开关切换）。
- `apps/panel/src/debug/debug.css`：调试面板样式。
- `apps/panel/src/debug/index.ts`：模块入口导出。
- `apps/panel/src/debug/debug-panel.e2e.spec.ts`：Playwright 端到端验收测试。
- `apps/panel/src/debug/playwright.config.ts`：端到端测试配置。
- `apps/panel/src/debug/*.test.ts(x)`：单元与静态渲染集成测试。
- `reports/P1-04/screenshots/`：验收测试运行截屏证据（包含总览、事件流与延迟瀑布）。

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| e2e 测试：发送一条 `stage.perform`，事件流中出现对应记录 | 通过 (1.6s) | `pnpm test:panel:e2e`；截图 `reports/P1-04/screenshots/debug-panel-events.png` |
| 仅在开发模式或设置里开启「开发者模式」后可见 | 通过 | e2e 测试验证未开启时只显示开启门控按钮，开启后展示调试面板根节点 |
| 核心状态在连接前显示「未连接」 | 通过 | `apps/panel/src/debug/DebugPanel.test.tsx`、e2e 测试断言初始状态为「未连接」 |
| 延迟面板计算最近 50 次 p50 与 p90 并在瀑布图中展示各阶段 | 通过 | `apps/panel/src/debug/latency.test.ts`、`DebugPanel.test.tsx` 及截图 `reports/P1-04/screenshots/debug-panel-latency.png` |
| 手动发送舞台命令通过 zod schema 校验 | 通过 | `apps/panel/src/debug/schema-validator.test.ts`（包含合法与非法 payload 校验） |
| `pnpm lint` | 通过 | 0 警告，0 错误 |
| `pnpm typecheck` | 通过 | 全工作区 TS 检查 0 错误 |
| `pnpm test` | 通过 | 29 个测试文件，443 项测试全部通过 |
| `pnpm test:e2e` | 通过 | 既有桌面端 e2e 测试（2 passed, 1 skipped，无回归） |
| `pnpm build` | 通过 | 14 个工作区项目全部构建通过 |

## 规格对照（Echo Core 相关的卡必填）
本任务卡不涉及 Echo Core 内核修改（非 P5/P6 卡），所使用的契约完全遵循 `docs/CONTRACTS.md` 第 2 节舞台协议与第 4 节应用内总线规范。

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| `@nemu/contracts` | workspace:* | 私有（项目内部包） | 提供舞台命令与总线事件的 zod schema 与 TypeScript 类型 |

*注：未引入任何新的第三方运行时或外部 npm 依赖。*

## 待用户确认
- [ ] （不阻塞）调试面板目前在非生产环境（或通过门控按钮/URL 参数）开启，默认展示包含全部子模块的「全部概览」视图，用户如有特定排版偏好后续可自由调整。

## 后续建议（不在本任务范围内）
- 待 P6-02（Echo Core 终端接入）合并后，真实核心的 `core.runState` 和 `receipt` 将直接流入总线并在面板中实时显示。
- 待 P4 语音管线接入后，真实的 `metrics.latency` 将由语音与主进程联动产出并在延迟瀑布图中展示真实端到端往返耗时。

## 下一步
P1-04 难度为「低」，按 AGENTS.md 第 6 节规则，CI 全绿且满足合并条件时由执行者合并。合并后将按 `tasks/README.md` 第 2 节继续取下一张卡。
