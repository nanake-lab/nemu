# P1 舞台与渲染（第 2–10 周）

出口条件：P1-01 至 P1-03 并入 N1，P1-04 并入 N2。舞台渲染器、窗口外壳与进程守护、评审台、调试面板全部可用，视觉回归基线建立。

---

## P1-01 舞台渲染器

- **负责：** Antigravity
- **依赖：** P0-02、P0-03
- **工期：** 5 天
- **分支：** `task/P1-01-stage-renderer`
- **目录：** `apps/stage`、`packages/avatar-runtime`

### 目标
做出生产级的舞台渲染器：按资产契约加载 VRM 1.0，透明合成正确，画面符合 ARCHITECTURE.md 6.1，实现舞台协议中与渲染相关的命令和事件。

### 交付物
- `packages/avatar-runtime/src/loader.ts`：读取 `nemu.profile.json` 并校验（用 `@nemu/contracts` 的 schema）；用 `VRMLoaderPlugin` 加载 VRM；VRM 0.x 直接报 `stage.error`。
- `packages/avatar-runtime/src/renderer.ts`：`WebGLRenderer({ alpha: true, premultipliedAlpha: true })`，抗锯齿在「默认帧缓冲 MSAA（`antialias: true`）」和「4× 多重采样渲染目标」中二选一，以边缘检查脚本的结果为准，并在报告里说明；色彩空间与色调映射按 three-vrm 推荐配置。
- `src/lighting.ts`：三点光预设（主光、轮廓光、环境光），参数可从调试面板调整。
- `src/camera.ts`：根据 `profile.framing` 生成全身和半身机位；画布大小随角色包围盒自适应。
- `src/outline.ts`：MToon 描边宽度随画布像素高度自适应，保证在 300–900 px 高度范围内视觉宽度稳定。
- `src/scheduler.ts`：帧调度器，支持 60、30、12 fps 和暂停；用 `requestAnimationFrame` 加时间累积跳帧，不靠 `setInterval`。
- `src/hittest.ts`：光标移动时（最多 30 Hz）用 WebGL2 异步读回（像素缓冲对象 + fence）取光标处 1 像素的 alpha，状态变化时发出 `stage.hitTest`。不要每帧同步调用 `readPixels`。
- `apps/stage`：实现 `stage.loadAvatar`、`stage.setQuality`、`stage.setPlacement`（本卡只需 `free` 锚点）、`stage.gaze`（只做头部朝向，完整视线在 P3）；发出 `stage.ready`、`stage.bounds`、`stage.frameStats`。
- 确定性模式：`?deterministic=1`。
- 视觉回归：`apps/stage/tests/visual/`，3 种背景（白、黑、彩色渐变）× 2 种机位 × 3 种表情（neutral、happy、surprised）= 18 张基准图。`pnpm test:visual` 固定使用软件渲染（Chromium 的 SwiftShader），基准图也必须在同样的环境下生成，这样本机和 CI 的结果一致。

### 验收
- `pnpm test:visual` 通过：pixelmatch 的 `threshold` 取 0.1，差异像素占比 ≤ 0.1%。
- 边缘检查脚本（完成标准第 3 项）：在白底、黑底、彩色渐变底各渲染一次，取角色轮廓外 2 px 的环带，计算背景色偏差的最大值，必须 ≤ 8/255。
- `pnpm bench`（在用户机器上运行）：待机 30 fps 时 CPU ≤ 3%、GPU ≤ 8%；活动 60 fps 时 1% low ≥ 50 fps。
- 用 `/browser` 打开 `pnpm dev:stage`，截图放入报告。

### 禁止
- 不要加 bloom、SSAO 这类会破坏 alpha 的后处理。
- 不要自己写 VRM 解析，统一用 three-vrm。

---

## P1-02 桌面窗口外壳与进程守护

- **负责：** Codex
- **依赖：** P0-02、P0-03
- **工期：** 5 天
- **分支：** `task/P1-02-desktop-shell`
- **目录：** `apps/desktop`

### 目标
把 P0-03 选定的窗口策略做成生产代码，实现应用内总线、托盘、快捷键、单实例等桌面外壳功能，并让主进程负责守护 Nemu 的全部后台进程。

### 交付物
- 舞台窗口：透明、无边框、不在任务栏显示、置顶（`setAlwaysOnTop(true, 'screen-saver')` 或按实测选择层级，在报告中说明）、`focusable: false`（点击角色时不抢焦点）。如果 Alt+Tab 列表里仍然出现舞台窗口，在 Windows 上改用 `type: 'toolbar'`。
- 点击穿透：响应 `stage.hitTest` 切换 `setIgnoreMouseEvents`。
- 显示器管理：监听 `screen` 的 display-added、display-removed、display-metrics-changed；角色所在显示器变化时迁移叠加窗口。
- 应用内总线实现（`apps/desktop/src/bus/`）：MessagePort 路由、环形日志（5000 条，只用于调试）、开发模式 zod 全量校验。它只承载界面层事件（CONTRACTS.md 4.1），不保存任何会影响她行为的状态。
- 进程守护（`apps/desktop/src/supervisor/`）：按配置启动、健康检查、崩溃后指数退避重启、退出时清理以下进程：`nemu-core.exe`、`irl-worker`、`drive-worker`、`model-adapter`、`voice-sidecar`、`codex-bridge`、`office-connector`。worker 一律通过 P5-05 的低完整性启动器启动（在 P5-05 合并前先用可替换的占位启动器，并在代码里标注）。主进程从不读写 Echo Core 的存储。各进程未就绪时用可配置的占位进程，保证应用能启动。
- 按 CONTRACTS.md 第 2 节创建三条通道：控制通道（主进程 ↔ 舞台）、音频通道（语音进程 ↔ 舞台）、麦克风通道（舞台 → 语音进程）。
- 按 CONTRACTS.md 2.3 把指定的应用内总线事件包进 `stage.event` 转发给舞台。
- 托盘菜单：显示/隐藏、勿扰、设置、退出。
- 全局快捷键（可配置）：召唤面板 `Ctrl+Shift+N`、按键说话 `Ctrl+Shift+Space`。
- 单实例锁；开机自启（设置项）；日志落盘到 `%APPDATA%/Nemu/logs/`。
- 设置存储：`%APPDATA%/Nemu/settings.json`，带版本号和迁移函数。

### 验收
- e2e 测试（完成标准第 17 项）：启动一个「点击计数」测试窗口放在舞台下面；点击透明区域，计数窗口收到点击；点击角色像素，计数窗口收不到、舞台收到 `stage.hit`。各 50 次，正确率 100%。
- e2e 测试：舞台窗口不出现在任务栏，Alt+Tab 列表里不出现。
- 单元测试：总线路由、环形日志、设置迁移、守护进程的退避重启（用会崩溃的测试进程）。
- 关闭应用后，任务管理器里不残留任何 Nemu 进程（e2e 检查）。
- 手动测试清单（写进报告，由用户勾选）：双显示器之间拖动角色；更改缩放比例后角色位置不跳。

---

## P1-03 评审台

- **负责：** Antigravity
- **依赖：** P1-01
- **工期：** 3 天
- **分支：** `task/P1-03-review-console`
- **目录：** `apps/stage/src/review/`

### 目标
给用户一个在浏览器里评审角色的页面，P2、P3、P4 的审美评审都在这里进行。

### 交付物（`pnpm dev:review`）
- 转台：自动旋转 / 手动拖动 / 预设角度（正、侧、背、3/4）。
- 光照预设与背景预设（纯色、壁纸样例、白、黑）。
- 表情网格：两种视图——单个表情（全部必需表情和 Nemu 自定义表情），以及 12 种表演提示 `emo` 按 `profile.emotionMap` 组合后的效果；缩略图点击放大。
- 身体读出预览：用滑块手动设置 `BodyReadout` 的各项（CONTRACTS.md 第 7 节），实时看姿态、耳朵、尾巴、待机节奏的变化（P3-03、P5-17 使用）。
- 动作播放器：列出 `motions/manifest.json` 的全部动作，可播放、循环、慢放。
- 对照模式：左边是设定集里的图（从 `content/nemu/bible/` 读取），右边是相同角度的渲染。
- 弹簧骨参数面板：实时调整刚度、阻力、重力，并能导出为 `build.yaml` 片段（P2-05 使用）。
- 评审记录：每个视图可以打标记「通过 / 有问题」并写备注，通过开发服务器的本地接口写入 `reports/review-<日期>.md`。
- 截图导出：PNG，带视图参数文件名。

### 验收
- 用 `/browser` 走一遍全部功能，录屏或截图放进报告。
- 用示例 VRM 导出一份评审记录样例。

---

## P1-04 调试面板

- **负责：** Antigravity
- **依赖：** P1-02
- **工期：** 2 天
- **分支：** `task/P1-04-debug-panel`
- **目录：** `apps/panel/src/debug/`

### 交付物
- 事件流查看器：实时显示应用内总线事件，可按类型、来源筛选，可暂停。
- 核心状态查看器：显示 Echo Core 的运行状态、谱系（排练或正式）、说话模式、最近的回执（RECEIVED_DURABLE、CORE_COMMITTED）和最近的身体读出；在 P6-02 合并前显示「未连接」。
- 延迟面板：按 `utteranceKey` 展示 `LatencyMarks` 的瀑布图（含接收、提交、首块），以及最近 50 次的 p50、p90。
- 帧统计：fps、p95 帧时间、当前物理状态。
- 手动发送舞台命令（JSON 编辑器 + schema 校验）。
- 仅在开发模式或设置里开启「开发者模式」后可见。

### 验收
- e2e 测试：发送一条 `stage.perform`，事件流中出现对应记录。
