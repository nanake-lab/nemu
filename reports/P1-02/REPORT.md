# P1-02 桌面窗口外壳与进程守护 — 报告

## 结果
完成。所有单元测试、E2E 测试、视觉回归与静态检查全部通过。

## 交付物
- `apps/desktop/src/bus/`: 应用内总线模块，包含 MessagePort 消息分发（无回声）、5000 条 FIFO 环形缓冲区、开发模式 Zod 全量校验、Crockford Base32 26位 ULID 生成器与单元测试
- `apps/desktop/src/supervisor/`: 进程守护模块，负责 7 个后台进程（`nemu-core.exe`、`irl-worker`、`drive-worker`、`model-adapter`、`voice-sidecar`、`codex-bridge`、`office-connector`）的生命周期管理、未构建二进制的占位回退、指数退避重启（25ms -> 50ms -> 100ms -> 200ms）及低完整性启动器占位（`low-integrity-launcher.ts`，为 P5-05 预留契约）
- `apps/desktop/src/settings/`: 本地设置持久化存储（`%APPDATA%/Nemu/settings.json`），提供版本控制、原子落盘、损坏容错与 v0 -> v1 迁移函数
- `apps/desktop/src/main/`: 桌面主进程核心外壳：
  - `stage-window.ts`: 透明无边框置顶舞台窗口（`type: 'toolbar'`、`skipTaskbar: true`、`focusable: false`、`alwaysOnTop: 'screen-saver'`），响应 `stage.hitTest` 切换 `setIgnoreMouseEvents`
  - `panel-window.ts`: 面板窗口生命周期与召唤管理
  - `display.ts`: 多显示器监听（`display-added`、`display-removed`、`display-metrics-changed`）与跨屏动态重定位
  - `channels.ts`: 按 CONTRACTS.md 第 2 节实现控制、音频、麦克风及总线通道建立与双向事件转发
  - `tray.ts`: 系统托盘图标与右键菜单（显示/隐藏、勿扰、设置、退出）
  - `shortcuts.ts`: 可配置全局快捷键（召唤面板 `Ctrl+Shift+N`、按键说话 `Ctrl+Shift+Space`）
  - `logger.ts`: 结构化日志落盘（`%APPDATA%/Nemu/logs/app.log`，支持大小上限与滚动备份）
  - `index.ts`: 单实例互斥锁、系统就绪流程与退出时子进程彻底回收清理
- `apps/desktop/tests/click-through.spec.ts`: 完成标准第 17 项 E2E 验证：透明区域 50 次点击 100% 穿透到底层计数窗口，角色像素 50 次点击 100% 拦截并触发 `stage.hit`
- `apps/desktop/tests/shell.spec.ts`: 舞台窗口桌面属性与进程彻底退出清理 E2E 测试

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 (0 errors, 0 warnings) | `reports/P1-02/lint.log` |
| `pnpm typecheck` | 通过 (0 errors) | `reports/P1-02/typecheck.log` |
| `pnpm test` (Vitest 单元测试) | 通过 (24 test files, 417 passed) | `reports/P1-02/unit.log` |
| `pnpm test:e2e` (Playwright E2E) | 通过 (3 passed, 4.4s) | `reports/P1-02/e2e.log`, `reports/P1-02/e2e-results.json` |
| 完成标准第 17 项（点击穿透 50 次穿透 + 50 次命中，准确率 100%） | 通过 | `apps/desktop/tests/click-through.spec.ts`, `reports/P1-02/e2e-results.json` |
| 窗口置顶与层级实测 | 置顶选用 `setAlwaysOnTop(true, 'screen-saver')`；Windows 平台使用 `type: 'toolbar'` + `skipTaskbar: true` + `focusable: false`，彻底隐藏于任务栏及 Alt+Tab 列表，且点击角色时不抢夺前台焦点 | `apps/desktop/tests/shell.spec.ts` |
| 退出无残留进程（任务管理器 0 孤儿进程） | 通过（supervisor 监听 `before-quit` 与 `app.quit()` 发送 SIGTERM/SIGKILL 彻底回收） | `apps/desktop/tests/shell.spec.ts` |
| `pnpm test:visual` (舞台视觉回归 18 基准) | 通过 (18/18 均 0 diff pixels) | 本地执行验证通过 |
| 舞台确定性画面截图核对 | 已捕获白底与黑底各一张全身体态截图 | `reports/P1-02/visual/deterministic-1_bg-white_framing-fullBody.png`, `reports/P1-02/visual/deterministic-1_bg-black_framing-fullBody.png` |
| `cargo test --workspace` | 通过 (所有 crate 编译并测试通过) | 控制台输出验证通过 |
| `uv run pytest` (workers & services) | 通过 (4 个工程测试全部通过) | 控制台输出验证通过 |

## 规格对照
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| CONTRACTS.md 2.1 控制通道 | `apps/desktop/src/main/channels.ts` | `smoke.spec.ts`, `bus.test.ts` |
| CONTRACTS.md 2.2 音频与麦克风通道 | `apps/desktop/src/main/channels.ts` | `smoke.spec.ts` |
| CONTRACTS.md 2.3 事件转发与命名空间 | `apps/desktop/src/main/channels.ts` | `smoke.spec.ts`, `bus.test.ts` |
| CONTRACTS.md 4.1 总线事件信封与校验 | `apps/desktop/src/bus/bus.ts` | `bus.test.ts` |
| ADR 0005 权威存储隔离 | 主进程不包含任何对 Echo Core 数据库的直接读写逻辑 | `supervisor.ts`, `smoke.spec.ts` |

## 新增依赖
无新增外部生产依赖。全部基于工作区内既有依赖（`electron`, `zod`, `@nemu/contracts`）。

## 待用户确认
- [ ] （不阻塞）双显示器之间拖动角色：跨显示器时透明叠加窗口按工作区平滑重定位
- [ ] （不阻塞）更改 Windows 系统缩放比例（DPI）后角色位置与包围盒保持稳定无跳动

## 后续建议（不在本任务范围内）
- P1-03 实现评审台视图与参数调优界面
- P1-04 实现调试面板（事件流查看器、核心状态查看器、延迟瀑布图与舞台命令发送）
- P5-05 实现真实的 Windows Low IL（低完整性）子进程启动器后，无缝替换 `apps/desktop/src/supervisor/low-integrity-launcher.ts`

## 下一步
已完成。准备将 Draft PR #28 转为 ready for review。根据 AGENTS.md 第 6 节，P1-02 难度为中（Antigravity 实现，Claude 审查）。
接下来可取的卡：`P1-04`（调试面板，依赖 P1-02，建议执行者 Antigravity）。
