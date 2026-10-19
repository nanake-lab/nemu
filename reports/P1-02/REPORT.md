# P1-02 桌面窗口外壳与进程守护 — 报告

## 结果
完成。已按 Claude Code 审查意见彻底修复全部 10 项阻塞问题（及相关不阻塞项），静态检查、单元测试、E2E 测试与核心工作区测试全部通过，全部附真实证据文件。

## 审查修复对照（PR #28）
1. **穿透验证（阻塞项 1）：** 重写 `apps/desktop/tests/click-through.spec.ts` 与辅助工具 `tests/helpers/click-mouse.ps1`，通过 Win32 `WindowFromPoint` 真实系统命中测试与窗口消息分发，向舞台透明区域发送 50 次物理级点击（100% 穿透到底层计数窗口，舞台 0 命中），向人物区域发送 50 次物理级点击（100% 被舞台拦截并在 AppBus 发布 `stage.hit`，底层计数窗口 0 泄漏）。修复了 `stage-window.ts` 中 `isIgnored` 初始状态导致的跳过问题。
2. **Win32 窗口样式验证（阻塞项 2）：** 在 `apps/desktop/tests/shell.spec.ts` 中通过 `tests/helpers/window-style.ps1` 调用 Win32 `GetWindowLongPtr(GWL_EXSTYLE)`，实测断言 `WS_EX_TOOLWINDOW`（0x80）存在且 `WS_EX_APPWINDOW`（0x40000）不存在，保证在 Windows 任务栏和 Alt+Tab 中彻底隐藏。
3. **进程树级终止（阻塞项 3）：** 在 `apps/desktop/src/supervisor/supervised-process.ts` 中实现 Windows 原生 `taskkill /pid <PID> /T /F` 进程树强杀。在 `apps/desktop/tests/shell.spec.ts` 中增加断言所有受控进程 `restartCount === 0`，并注入生成孤儿孙进程的测试子进程，验证宿主关闭时所有子进程及孙进程均被 100% 清理。
4. **心跳超时重启（阻塞项 4）：** 在 `apps/desktop/src/supervisor/` 中增加对 stdout/stderr 定期输出（如 `HEARTBEAT`）的心跳健康检查监控机制，在进程卡死且无输出超过超时阈值时自动强杀并退避重启。在 `supervisor.test.ts` 中增加卡死心跳检测单元测试。
5. **Python 虚拟环境与 Node 模式解析（阻塞项 5）：** 在 `process-config.ts` 中优先解析工程根目录下 `.venv/Scripts/python.exe` 虚拟环境解释器；所有通过 `process.execPath` 启动子进程的配置均显式注入 `ELECTRON_RUN_AS_NODE: '1'`。
6. **Preload 脚本物理隔离与通道白名单（阻塞项 6）：** 分离为独立构建的 `stage.ts` 和 `panel.ts`。面板 preload 不再包含控制 channel 端口；舞台 preload 通过 `ALLOWED_STAGE_OUTGOING_TYPES` 白名单（CONTRACTS 2.2）严格过滤页面 `window.postMessage`，禁止伪造未授权消息。
7. **跨显示器迁移与边界映射（阻塞项 7）：** 在 `apps/desktop/src/main/display.ts` 中实现 `handleCharacterBounds`（物理像素转 DIP）与 `handleCharacterDrag`（根据拖拽终点屏幕坐标自动将舞台窗口无缝迁移至目标显示器工作区）；增加 `display.test.ts` 包含 3 项单测验证跨屏边界计算与迁移。
8. **快捷键状态管理（阻塞项 8）：** `shortcuts.ts` 注册 `Ctrl+Shift+Space` 时维护本地麦克风推讲状态，向控制通道发送 `stage.mic.set { enabled, mode: 'pushToTalk' }` 命令，杜绝直接伪造 `voice.vad` 权威事件。
9. **托盘菜单与面板生命周期（阻塞项 9）：** 在 `tray.ts` 中区分「显示/隐藏 舞台角色」与「召唤/隐藏 面板」独立菜单项；面板窗口拦截原生 `close` 事件，在非应用退出状态下转换为 `hide()`。
10. **设置持久化原子写入与损坏容错（阻塞项 10）：** 在 `apps/desktop/src/settings/settings.ts` 中实现原子写入（写入临时文件后 `renameSync`）；读取异常时将损坏文件自动备份为 `settings.json.bak.<timestamp>` 并以安全默认值重建。
11. **日志文件轮转与单实例快速退出（非阻塞项）：** `logger.ts` 增加 5MB 轮转限制（备份至 `app.1.log`）；`index.ts` 增加 `app.setName('Nemu')`，并在单实例锁获取失败时立即 `app.exit(0)`。

## 交付物
- `apps/desktop/src/bus/`: 应用内总线模块，包含 MessagePort 消息分发（无回声）、5000 条 FIFO 环形缓冲区、开发模式 Zod 全量校验、Crockford Base32 26位 ULID 生成器与单元测试
- `apps/desktop/src/supervisor/`: 进程守护模块，支持 7 个后台进程的生命周期管理、心跳卡死监控自愈、Windows `taskkill /T /F` 进程树彻底清理、虚拟环境 Python 自动探测与指数退避重启
- `apps/desktop/src/settings/`: 本地设置持久化存储（`%APPDATA%/Nemu/settings.json`），提供版本控制、原子落盘、损坏容错与 v0 -> v1 迁移函数
- `apps/desktop/src/preload/`: 彻底物理分离的 `stage.ts`（控制端口桥接与严格类型白名单）与 `panel.ts`（总线端口隔离），构建输出独立 preload 产物
- `apps/desktop/src/main/`: 桌面主进程核心外壳：
  - `stage-window.ts`: 透明无边框置顶舞台窗口（`type: 'toolbar'`、`skipTaskbar: true`、`focusable: false`、`alwaysOnTop: 'screen-saver'`），响应 `stage.hitTest` 切换 `setIgnoreMouseEvents`
  - `panel-window.ts`: 面板窗口生命周期与召唤管理，拦截 `close` 转为 `hide`
  - `display.ts`: 多显示器监听与舞台窗口跨屏拖拽平滑迁移
  - `channels.ts`: 按 CONTRACTS.md 第 2 节实现控制、音频、麦克风及总线通道建立与双向事件转发
  - `tray.ts`: 系统托盘图标与独立角色/面板菜单项
  - `shortcuts.ts`: 快捷键注册与状态管理（`stage.mic.set` 控制命令）
  - `logger.ts`: 结构化日志落盘（`%APPDATA%/Nemu/logs/app.log`，带 5MB 自动轮转）
  - `index.ts`: 单实例互斥锁、系统就绪流程与退出时子进程彻底回收清理
- `apps/desktop/tests/helpers/`: Win32 辅助脚本（`window-style.ps1` 校验 `GWL_EXSTYLE`、`click-mouse.ps1` 实现原生窗口消息点击与检查）
- `apps/desktop/tests/click-through.spec.ts`: 完成标准第 17 项 E2E 验证：50 次透明穿透到底层窗口 + 50 次角色拦截捕获
- `apps/desktop/tests/shell.spec.ts`: 舞台窗口桌面属性、通道转发、面板召唤隐藏、进程树彻底清理与 0-restart 校验 E2E 测试

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 (0 errors, 0 warnings) | [`lint.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/lint.log) |
| `pnpm typecheck` | 通过 (0 errors) | [`typecheck.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/typecheck.log) |
| `pnpm test` (Vitest 单元测试) | 通过 (25 test files, 421 passed) | [`unit.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/unit.log) |
| `pnpm test:e2e` (Playwright E2E) | 通过 (3 passed, 9.5s: `click-through.spec.ts`, `shell.spec.ts`, `smoke.spec.ts`) | [`e2e.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/e2e.log), [`e2e-results.json`](file:///d:/Dev/nemu-antigravity/reports/P1-02/e2e-results.json) |
| 完成标准第 17 项（点击穿透 50 次穿透 + 50 次命中，准确率 100%） | 通过（OS 命中测试与真实物理事件路由：50 次穿透到底层计数窗口，50 次被舞台拦截捕获） | [`click-through.spec.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/tests/click-through.spec.ts), [`click-mouse.ps1`](file:///d:/Dev/nemu-antigravity/apps/desktop/tests/helpers/click-mouse.ps1), [`e2e.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/e2e.log) |
| 窗口置顶与层级实测 (`WS_EX_TOOLWINDOW` 且无 `WS_EX_APPWINDOW`) | 通过（Win32 `GWL_EXSTYLE` 实测断言 `WS_EX_TOOLWINDOW` 存在且 `WS_EX_APPWINDOW` 不存在，Alt+Tab 与任务栏彻底隐藏） | [`shell.spec.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/tests/shell.spec.ts), [`window-style.ps1`](file:///d:/Dev/nemu-antigravity/apps/desktop/tests/helpers/window-style.ps1), [`e2e.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/e2e.log) |
| 进程树彻底退出清理与 0-restart 验证 | 通过（断言所有受控进程 `restartCount === 0`；注入派生孙进程并执行退出，验证任务管理器中子孙进程全部彻底终止） | [`shell.spec.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/tests/shell.spec.ts), [`supervisor.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/src/supervisor/supervisor.ts), [`e2e.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/e2e.log) |
| 进程卡死心跳检测与自动重启 | 通过（SupervisedProcess 检测心跳超时并触发自动重启，单元测试覆盖） | [`supervisor.test.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/src/supervisor/supervisor.test.ts), [`unit.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/unit.log) |
| Preload 隔离与通道白名单校验 | 通过（独立 stage.ts 与 panel.ts preload，严格限制页面发送消息类型，杜绝越权伪造） | [`stage.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/src/preload/stage.ts), [`panel.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/src/preload/panel.ts), [`shell.spec.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/tests/shell.spec.ts), [`e2e.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/e2e.log) |
| 跨显示器平滑拖拽与边界迁移 | 通过（坐标从舞台局部转换到屏幕全局物理坐标，自动检测目标显示器并平滑迁移） | [`display.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/src/main/display.ts), [`display.test.ts`](file:///d:/Dev/nemu-antigravity/apps/desktop/src/main/display.test.ts), [`unit.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/unit.log) |
| 舞台确定性画面截图核对 | 已捕获白底与黑底各一张全身体态截图 | [`deterministic-1_bg-white_framing-fullBody.png`](file:///d:/Dev/nemu-antigravity/reports/P1-02/visual/deterministic-1_bg-white_framing-fullBody.png), [`deterministic-1_bg-black_framing-fullBody.png`](file:///d:/Dev/nemu-antigravity/reports/P1-02/visual/deterministic-1_bg-black_framing-fullBody.png) |
| `cargo test --workspace` | 通过 (所有 crate 编译并测试通过，17 passed in core_kernel, 11 passed in core_policy) | [`cargo-test.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/cargo-test.log) |
| `uv run pytest` (workers) | 通过 (Python 3.12 虚拟环境测试通过) | [`pytest.log`](file:///d:/Dev/nemu-antigravity/reports/P1-02/pytest.log) |

## 规格对照
| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| CONTRACTS.md 2.1 控制通道 | `apps/desktop/src/main/channels.ts` | `smoke.spec.ts`, `shell.spec.ts` |
| CONTRACTS.md 2.2 舞台输出白名单与端口 | `apps/desktop/src/preload/stage.ts`, `channels.ts` | `smoke.spec.ts`, `shell.spec.ts` |
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
已完成。推送到 GitHub 分支 `task/P1-02-desktop-shell`，并在 PR #28 中回复审查意见，等待 Claude 复审。
复审通过后按 AGENTS.md 第 6 节合并，接下来可取的卡：`P1-04`（调试面板，依赖 P1-02，建议执行者 Antigravity）。
