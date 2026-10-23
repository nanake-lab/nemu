# P6-08 FFI、许可与发布核验

## 固定依赖与原文出处

- Koffi：锁文件固定 `3.3.3`，MIT；`@koromix/koffi-win32-x64` 同为 `3.3.3`、MIT，是其原生可选依赖。
- 原始分发包：[koffi-3.3.3.tgz](https://registry.npmjs.org/koffi/-/koffi-3.3.3.tgz)，其中 `package/LICENSE.txt`。
- 锁文件 integrity：`sha512-XWNI4eyxJJzR2MakPZzR3QeZ06JOklLbxglin0dHhx/qIQCby8lqHs9Acq4Q7CpCKnOrBSj+6b9J5y69wfr2ZA==`。
- LICENSE 全文原样复制到 [evidence/koffi-3.3.3-LICENSE.txt](evidence/koffi-3.3.3-LICENSE.txt)，SHA-256：`22BCBBBACD739B026A8DC484F62A31484CE4C4E31F3408B60B29B4E41D62B290`。
- 验证工具 `@electron/asar 4.3.1`（MIT）仅装在 D 盘任务缓存，未加入运行时依赖；构建使用现有 esbuild/Vite。

## 单一 FFI 边界

所有 Koffi 加载、声明、回调注册/卸载及原生调用均在 `packages/desktop-sense/src/win32.ts`。原生模块只在显式创建 Win32 source 时加载，缺失二进制/系统调用初始化失败由主入口捕获并记录错误，壳继续运行、发 busy 门控。

该文件超过 400 行的理由：用户要求单一 FFI 模块。原生 ABI 声明、坐标上下文恢复、source、资源清理和原生合成窗口夹具集中在此；几何、语义、循环、桌面接线均已拆出。原生合成夹具与 ABI probe 不在公共包入口导出，也不会进入主进程生产 bundle。

查询接口清单：`EnumWindows`、`IsWindowVisible`、`IsIconic`、`DwmGetWindowAttribute`（EXTENDED_FRAME_BOUNDS/CLOAKED）、`GetWindowThreadProcessId`、`QueryFullProcessImageNameW`、`GetClassNameW`、`GetWindowTextW`（仅经 T1）、`GetForegroundWindow`、`EnumDisplayMonitors`、`GetMonitorInfoW`、`GetDpiForMonitor`、`SHAppBarMessage`（只用 ABM_GETTASKBARPOS）、`SHQueryUserNotificationState`、`GetLastInputInfo`、`GetTickCount`、`GetGUIThreadInfo`、`ClientToScreen`、`LogicalToPhysicalPointForPerMonitorDPI`、`GetWindowDpiAwarenessContext`。

受控资源接口：`OpenProcess` 只请求 PROCESS_QUERY_LIMITED_INFORMATION，随即 `CloseHandle`；`SetWinEventHook`/`UnhookWinEvent`；`SetThreadDpiAwarenessContext` 仅在同步查询期间暂设 PMv2 并恢复，不改用户/系统设置。仅合成夹具使用 `CreateWindowExW`/`DestroyWindow`，只操作自行创建的非激活离屏窗口。不绑定截屏、键盘内容读取、输入注入、注册表写入或系统设置修改接口。

## P7 发布后续

- 保留 Koffi 及原生可选包的 MIT 通知；Windows x64 产物必须包含匹配版本的原生包。
- 安装包制作时显式解包 `.node` 及 Koffi 运行资源；本卡 ASAR smoke 包将 node_modules 全部放在 app.asar.unpacked，未修改其他构建配置。P7 可收窄解包规则，但必须重跑实际打包加载和退出检查。
- P7 负责最终安装器、签名、ASAR 完整性/fuses、更新与发布；本卡的临时分发目录不是签名安装包。不得把“开发环境能加载”替代发布目录的运行证据。
- CI/本机测试的 scaffold userData 一律选择合成 source；不依赖用户打开的窗口或实际会议状态。
