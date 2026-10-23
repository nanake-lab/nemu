# P6-08 主入口接线与设置接口请求

日期：2026-10-24。依据：AGENTS.md 第 2 节的目录范围。

本卡允许 `packages/desktop-sense/` 与 `apps/desktop/src/sense/`。已在范围内提供 `startDesktopSense` 注入接口，接收总线、舞台发送函数、会话锁屏信号、实时 T1 授权与安静时段查询，不读写设置。

要在当前桌面主进程中实际启动，需要在卡外 `apps/desktop/src/main/index.ts` 接入启动与退出清理。当前 `packages/terminal/` 仍是骨架；语义事件交给现有总线，核心传输由终端接线实现，本卡不复制 IDL。

期望接口：主进程在控制通道就绪后启动感知；退出前先停止；授权来源提供可撤销的标题读取决策；设置提供安静时段查询。未经授权不改卡外实现。

另：Microsoft 对 SHQueryUserNotificationState 的公开定义只说明通知阻断、全屏、演示和系统首次登录 quiet time，没有将其定义为 Windows 专注模式的可靠状态来源。不能把 QUNS_BUSY 桩测试当成真实专注模式已验证。

## 2026-10-24 用户授权与构建接线

用户已授权修改 `apps/desktop/src/main/index.ts`，仅限集中放置启动、退出清理与失败处理，并要求退出时间对照。

首次完整构建成功，但检查产物发现主进程把 `@nemu/desktop-sense` 留为外部模块，其 exports 指向 TypeScript 源文件。需要在卡外 `apps/desktop/electron.vite.config.ts` 按已有 contracts/voice 做法追加 `@nemu/desktop-sense` 到 exclude，同时追加 koffi 到 external，让已编译的感知代码进入发布包、原生 Koffi 作为原生依赖加载。期望只改这两处数组，不重构配置。

用户已授权两个数组追加，条件为先核对固定版本许可、提供实际打包运行证据、单一 FFI 模块、P7 登记发布事项。已落实在 reports/P6-08/FFI-AND-LICENSE.md 与 REPORT.md；未改其他构建配置。
