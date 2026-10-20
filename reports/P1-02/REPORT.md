# P1-02 桌面窗口外壳与进程守护 — 报告

## 结果
完成。已按方案 A 彻底修复 Claude Code 第四轮复审意见（Comment 6284814478）。删除了 `click-mouse.ps1` 中所有 `PostMessage` 回退代码及相关 Win32 消息 API，移动与点击纯粹使用 Win32 `SendInput` 并对每次调用严格校验返回值；将底层计数窗口层级调整为紧邻舞台下方（`alwaysOnTop: 'pop-up-menu'`），避免被桌面其它窗口遮挡；在本地交互式桌面（`WinSta0\Default`）以最终头提交实跑并通过全部 50+50 次真实物理点击穿透与角色拦截测试，日志记录提交哈希、时间、会话、输入桌面及每次 `SendInputResult`；若处于无输入桌面权限环境则统一安全 `test.skip`；`voice-sidecar` 健康检查调整为 `none`（等待 P4 契约定义地址）；`shell.spec.ts` 增加 POSIX 保护。

## 审查与复审修复对照（PR #28）
1. **彻底消除 PostMessage 回退与全链路 SendInput 返回值校验（第四轮复审阻塞项 2）：**
   - 彻底删除 `click-mouse.ps1` 中所有 `PostMessage` 投递代码（包括 `WindowFromPoint`、`ScreenToClient`、`PostMessage(WM_LBUTTONDOWN/UP)` 等）。
   - 在 `Click` 方法中分别注入带绝对坐标的 `MOUSEEVENTF_LEFTDOWN`、等待 15ms 物理按压时长后再注入 `MOUSEEVENTF_LEFTUP`，对每一次光标移动、按下、抬起的 `SendInput` 返回值分别做严格校验（必须等于 1，否则立即返回 0 并退出）。
   - 在 `click-through.spec.ts` 中，`sendNativeClicks` 与 `sendNativeMouseMove` 一致对每次 PowerShell 执行捕获并解析 `SendInputResult`，若返回 0 则统一调用 `test.skip(true, '当前会话不是交互式桌面，无法注入真实输入 (SendInput returned 0)')`，任何其他异常一律抛出真实失败，绝不掩盖问题。
2. **底层计数窗口层级锁定与 CI 失败根因修复（第四轮复审阻塞项 1）：**
   - 根因分析：此前 `stage` 窗口设置了 `alwaysOnTop: 'screen-saver'`（Windows 最高置顶级别），而测试动态创建的 `counterWindow` 为普通窗口（`alwaysOnTop: false`）。在 Windows 真实交互桌面或宿主桌面运行有其它置顶/活跃窗口（如测试期间桌面上运行的 Google Chrome 窗口）时，`stage` 穿透的真实物理点击落在了中间遮挡的窗口上，导致 `counterWindow` 收到 0 次点击。
   - 修复措施：给 `counterWindow` 设置 `counterWindow.setAlwaysOnTop(true, 'pop-up-menu')` 并调用 `counterWindow.focus()`，确保其在 Windows Z-order 中紧邻舞台窗口下方，且高于所有桌面普通窗口；同时在 `click-mouse.ps1` 中输出当前光标下的目标窗口 HWND 与进程名以供溯源。
3. **交互式实跑证据规范化与完整溯源（第四轮复审阻塞项 3）：**
   - 在本地 Windows 真实交互式桌面（`WinSta0\Default`）使用最终头提交实跑并通过全部测试。
   - `reports/P1-02/interactive-click-through.log` 完整记录执行出处：包含 `git rev-parse HEAD` 完整哈希、ISO 精确时间戳、会话名称与用户（`Console` / `DESKTOP-8LBF84O\nanake`）、输入桌面名称（`WinSta0\Default`）以及每次光标移动与点击的 `SendInputResult: 1` 详细输出。
4. **真实硬件鼠标输入与穿透验证（第三轮复审阻塞项 1，采用方案 A）：**
   - 彻底删除 `click-mouse.ps1` 中的 `ForwardMouseMove` 方法和 `-StageHwndHex` 参数，严禁向舞台窗口直接 `PostMessage(WM_MOUSEMOVE)` 绕过底层命中测试。
   - 改掉此前「完全还原 WH_MOUSE_LL 底层鼠标钩子行为」的说法，如实陈述实现机制：使用纯 Win32 `SendInput` 注入真实系统光标移动与点击，由操作系统天然路由经过 Electron 窗口底层的 `WH_MOUSE_LL` 钩子。
   - 完整实测通过：50 次透明区域物理点击 100% 穿透至底层计数窗口（舞台 0 命中），50 次人物区域物理点击 100% 被舞台拦截并在 AppBus 发布 `stage.hit`（底层计数窗口 0 泄漏）。
5. **voice-sidecar 健康探针调整（第三轮复审不阻塞项）：**
   - 在 `process-config.ts` 中将 `voice-sidecar` 的 `healthProbe` 调整为 `{ type: 'none' }`。因 P4 尚未正式定义其 HTTP 探针路由与端口，且原写死的 8765 端口与本地项目看板服务（`pnpm board`）冲突，改为暂不探测，待 P4 接入时统一从配置读取。
6. **Linux 平台孙进程树清理保护（第三轮复审不阻塞项）：**
   - 在 `shell.spec.ts` 中针对启动独立孙进程的测试增加 `process.platform === 'win32'` 判断。Windows 依赖 `taskkill /pid <PID> /T /F` 验证彻底杀灭孙进程；POSIX 环境下不生成 detached 孙进程，避免因未开启进程组强杀导致 Playwright 关闭超时卡住。
7. **健康检查重复重启与孤儿进程消除（第二轮复审阻塞项 1）：**
   - 梳理守护进程崩溃处理流程：健康检查超时强杀时仅执行 `taskkill /pid <PID> /T /F`，不再在其回调中调用 `handleProcessCrash()`，所有崩溃、退避计算与重启调度统由子进程唯一的 `exit` 事件负责。
   - 引入 `currentGeneration` 代际计数器，全生命周期内同一时刻至多存在一个活动子进程、至多一个处于挂起状态的重启定时器；`stop()` 执行时主动递增代际使在途事件失效，彻底终结进程树。
8. **Win32 窗口样式验证（首轮阻塞项 2）：**
   - 在 `shell.spec.ts` 中通过 `window-style.ps1` 调用 Win32 `GetWindowLongPtr(GWL_EXSTYLE)`，实测断言 `WS_EX_TOOLWINDOW`（0x80）存在且 `WS_EX_APPWINDOW`（0x40000）不存在，保证在 Windows 任务栏和 Alt+Tab 中彻底隐藏。
9. **Preload 脚本物理隔离与白名单（首轮阻塞项 6 & 非阻塞项）：**
   - 物理分离为独立构建的 `src/preload/stage.ts` 与 `src/preload/panel.ts`；彻底删除废弃的 `src/preload/index.ts` 及其构建条目；严格限制页面发送消息类型，杜绝越权伪造。
8. **多显示器混合 DPI 兼容（非阻塞项）：**
   - 在 `apps/desktop/src/main/display.ts` 中使用 `screen.screenToDipPoint` 计算物理像素至设备独立像素（DIP）的坐标转换，准确适配跨显示器不同缩放比例布局。
9. **设置迁移前向兼容（非阻塞项）：**
   - 在 `apps/desktop/src/settings/migrations.ts` 中，当检测到未来更高 schema 版本（`version > CURRENT_SETTINGS_VERSION`）时，合并默认基线配置与未知新字段，保证基线字段存在的同时保留新版本配置，杜绝破坏性降级。

## 交付物
- `apps/desktop/src/bus/`: 应用内总线模块，包含 MessagePort 消息分发（无回声）、5000 条 FIFO 环形缓冲区、开发模式 Zod 全量校验、Crockford Base32 26位 ULID 生成器与单元测试
- `apps/desktop/src/supervisor/`: 进程守护模块，支持代际计数（`currentGeneration`）、单退出路径崩溃处理、多协议健康探针（`stdout-heartbeat` / `http-get` / `none`）、Windows `taskkill /T /F` 进程树彻底清理、虚拟环境 Python 自动探测与指数退避重启
- `apps/desktop/src/settings/`: 本地设置持久化存储（`%APPDATA%/Nemu/settings.json`），提供版本控制、原子落盘、损坏容错、未知未来版本前向兼容与 v0 -> v1 迁移函数
- `apps/desktop/src/preload/`: 彻底物理分离的 `stage.ts`（控制端口桥接与严格类型白名单）与 `panel.ts`（总线端口隔离），构建输出独立 preload 产物
- `apps/desktop/src/main/`: 桌面主进程核心外壳：
  - `stage-window.ts`: 透明无边框置顶舞台窗口（`type: 'toolbar'`、`skipTaskbar: true`、`focusable: false`、`alwaysOnTop: 'screen-saver'`），响应 `stage.hitTest` 切换 `setIgnoreMouseEvents`
  - `panel-window.ts`: 面板窗口生命周期与召唤管理，拦截 `close` 转为 `hide`
  - `display.ts`: 多显示器监听与舞台窗口跨屏拖拽平滑迁移（支持 `screenToDipPoint` 混合 DPI）
  - `channels.ts`: 按 CONTRACTS.md 第 2 节实现控制、音频、麦克风及总线通道建立与双向事件转发
  - `tray.ts`: 系统托盘图标与独立角色/面板菜单项
  - `shortcuts.ts`: 快捷键注册与状态管理（`stage.mic.set` 控制命令）
  - `logger.ts`: 结构化日志落盘（`%APPDATA%/Nemu/logs/app.log`，带 5MB 自动轮转）
  - `index.ts`: 单实例互斥锁、系统就绪流程与退出时子进程彻底回收清理
- `apps/desktop/tests/helpers/`: Win32 辅助脚本（`window-style.ps1` 校验 `GWL_EXSTYLE`、`click-mouse.ps1` 实现基于 Win32 `SendInput` 的真实系统鼠标移动与点击）
- `apps/desktop/tests/click-through.spec.ts`: 完成标准第 17 项 E2E 验证：纯 Win32 `SendInput` 驱动硬件光标移动并触发真实命中检测，50 次透明穿透到底层窗口 + 50 次角色拦截捕获；非交互式桌面下安全 skip
- `apps/desktop/tests/shell.spec.ts`: 舞台窗口桌面属性、通道转发、面板召唤隐藏、进程树彻底清理与 0-restart 校验 E2E 测试

## 验收
| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 (0 errors, 0 warnings) | [lint.log](lint.log) |
| `pnpm typecheck` | 通过 (0 errors) | [typecheck.log](typecheck.log) |
| `pnpm test` (Vitest 单元测试) | 通过 (25 test files, 422 passed, 覆盖代际重启与 0 孤儿断言) | [unit.log](unit.log) |
| `pnpm test:e2e` (Playwright 全套 E2E) | 通过 (2 passed, 1 skipped (click-through 自动识别非交互式桌面环境跳过), 7.6s) | [e2e.log](e2e.log) |
| 完成标准第 17 项（交互式桌面 `WinSta0\Default` 实跑） | **通过 (1 passed, 6.6s)**：纯 Win32 `SendInput` 移动光标驱动真实 `stage.hitTest`，50 次穿透到底层计数窗口，50 次被舞台拦截并在 AppBus 捕获发布 `stage.hit`，0 假阳性 | [`click-through.spec.ts`](../../apps/desktop/tests/click-through.spec.ts), [`click-mouse.ps1`](../../apps/desktop/tests/helpers/click-mouse.ps1), [interactive-click-through.log](interactive-click-through.log) |
| 完成标准第 17 项（非交互式会话 / CI 环境） | **安全跳过 (1 skipped)**：检测到 `SendInput` 返回 0（`ERROR_ACCESS_DENIED`，无真实输入桌面注入权限），通过 `test.skip` 跳过并明确输出跳过原因，绝不回退至伪造窗口消息 | [`click-through.spec.ts`](../../apps/desktop/tests/click-through.spec.ts), [e2e.log](e2e.log) |
| 窗口置顶与层级实测 (`WS_EX_TOOLWINDOW` 且无 `WS_EX_APPWINDOW`) | 通过（Win32 `GWL_EXSTYLE` 实测断言 `WS_EX_TOOLWINDOW` 存在且 `WS_EX_APPWINDOW` 不存在，Alt+Tab 与任务栏彻底隐藏） | [`shell.spec.ts`](../../apps/desktop/tests/shell.spec.ts), [`window-style.ps1`](../../apps/desktop/tests/helpers/window-style.ps1), [e2e.log](e2e.log) |
| 进程树彻底退出清理与 0 孤儿验证 | 通过（断言所有受控进程代际稳定；模拟健康检查超时多次重启，实测同时存活子进程 <= 1，停止后历史子进程全部退出无孤儿） | [`supervisor.test.ts`](../../apps/desktop/src/supervisor/supervisor.test.ts), [unit.log](unit.log), [e2e.log](e2e.log) |
| 进程卡死心跳检测与健康探针 | 通过（`stdout-heartbeat` / `none` 按进程配置，探针失败后仅触发强杀，通过单退出事件重启，单元测试覆盖） | [`supervised-process.ts`](../../apps/desktop/src/supervisor/supervised-process.ts), [unit.log](unit.log) |
| Preload 隔离与通道白名单校验 | 通过（独立 stage.ts 与 panel.ts preload，删除废弃 index.ts，严格限制页面发送消息类型，杜绝越权伪造） | [`stage.ts`](../../apps/desktop/src/preload/stage.ts), [`panel.ts`](../../apps/desktop/src/preload/panel.ts), [e2e.log](e2e.log) |
| 跨显示器平滑拖拽与边界迁移 | 通过（坐标从舞台局部转换到屏幕全局物理坐标，支持 Windows `screenToDipPoint` 处理混合 DPI 边界迁移） | [`display.ts`](../../apps/desktop/src/main/display.ts), [`display.test.ts`](../../apps/desktop/src/main/display.test.ts), [unit.log](unit.log) |
| 舞台确定性画面截图核对 | 已捕获白底与黑底各一张全身体态截图 | [white-bg](visual/deterministic-1_bg-white_framing-fullBody.png), [black-bg](visual/deterministic-1_bg-black_framing-fullBody.png) |
| `cargo test --workspace` | 通过 (所有 crate 编译并测试通过，17 passed in core_kernel, 11 passed in core_policy) | [cargo-test.log](cargo-test.log) |
| `uv run pytest` (workers) | 通过 (Python 3.12 虚拟环境测试通过) | [pytest.log](pytest.log) |

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
- 真实子进程健康探针补全：当前除占位进程外，真实组件探针均配置为 none（由 exit 事件与进程树强杀兜底）；后续接入各真实进程的任务卡（P4-voice、P5-echo-core、P6-presence 等）需依据各组件契约规范补充专用探针（如 HTTP GET /health 或命名管道握手）
- P1-03 实现评审台视图与参数调优界面
- P1-04 实现调试面板（事件流查看器、核心状态查看器、延迟瀑布图与舞台命令发送）
- P5-05 实现真实的 Windows Low IL（低完整性）子进程启动器后，无缝替换 `apps/desktop/src/supervisor/low-integrity-launcher.ts`

## 下一步
已完成。推送到 GitHub 分支 `task/P1-02-desktop-shell`，并在 PR #28 中回复审查意见，等待 Claude 复审。
复审通过后按 AGENTS.md 第 6 节合并，接下来可取的卡：`P1-04`（调试面板，依赖 P1-02，建议执行者 Antigravity）。
