# P6-08 桌面感知 — 报告

## 当前状态
**部分完成，保持草稿 PR #75。** CPU 正式测量等待用户通知空闲时间，专注模式来源尚未确认，末轮全量测试有一个 supervisor 失败，见 [BLOCKED.md](BLOCKED.md)。尚不满足转为可审查的条件；不合并、不取下一张卡。工作区 `D:\Dev\nemu-wt\P6-08`，分支 `task/P6-08-desktop-sense`；起点及收尾 fetch 后的 origin/main 均为 `0b5412e`。

## 交付物

- `packages/desktop-sense/src/`：单模块 Win32 FFI，PMv2 查询与恢复、DWM 窗口/任务栏/屏幕平台、前景遮挡裁剪、稳定平台 ID、插入点、公开通知状态、会议特征、空闲边沿、节流与清理。
- `apps/desktop/src/sense/`：几何只发舞台，语义走既有 AppBus；锁屏停采样，故障 busy，舞台 ready 后重送当前几何，退出移除全部监听及定时器。
- `main/index.ts`：获用户授权的集中启动/停止/错误处理；启动失败记 error、发 busy，壳继续。退出保护覆盖 supervisor 尚未建立而感知已启动的情况。
- `electron.vite.config.ts`：只在获准的两个数组追加桌面感知包与 koffi；前者编入主进程，后者作为原生依赖加载。
- [规格对照](../../docs/traceability/P6-08.md)、[FFI/许可/P7 后续](FFI-AND-LICENSE.md)、[测前提交的 CPU 方法](CPU-PROTOCOL.md)、`evidence/` 原始证据。

## 验收
| 命令或检查 | 结果 | 证据（evidence/ 下） |
| --- | --- | --- |
| `pnpm test:desktop-sense` | 5 文件、28 测试通过 | unit-28.txt |
| `pnpm lint`、`pnpm typecheck` | 当前实现通过 | lint-current.txt、typecheck-current.txt |
| `pnpm build`；最终桌面 build | 通过；保留现有 Zod PURE 注释告警 | build-final.txt、desktop-build-final.txt |
| `pnpm sense:geometry …/native-geometry-current.json` | 8 个原生合成窗口，100/125/150/200% 物理尺寸、两个合成显示器范围，边误差均 0 px | native-geometry-current.json/.txt |
| 勿扰桩矩阵 | 全屏/PPT/会议/通知 busy 各 10 次断言通过；不等于真实专注开关验收 | semantics.test.ts、unit-28.txt |
| 隐私/边沿/清理 | 未授权不读取标题，撤回不附带标题、不冒充会议结束；几何不进总线；锁屏停采样；监听和定时器清零 | engine.test.ts、sense/index.test.ts |
| 打包后 Electron/Koffi | isPackaged=true；Electron 44.7.0、Node 24.21.0、modules 149、N-API 10；原生 x64 二进制从 app.asar.unpacked 找到并调用成功；合成 sample、hook 注册/卸载及销毁正常 | package-smoke-05.json/.txt |
| 接线前后退出 | 见下表；6 次退出码 0，2 秒后均无测试子进程残留 | package-smoke-05.json/.txt |
| 首轮全量 `pnpm test` | 当时 559/559 通过 | test-01.txt |
| 末轮全量 `pnpm test` | **559/560，失败**：supervisor.test.ts:64，restartCount=0，期望 ≥1；未改此文件，未重跑覆盖 | test-final.txt |
| CPU ≤1% | **未通过**：首测 2.339468%；新方法未运行，等待用户通知 | native-acceptance-01.json/.txt |

坐标证据使用真实 DWM 查询自行创建的窗口，显示器/DPI 为合成数据；未改变实际显示器或缩放，不外推物理双屏现场验收。白名单在读取窗口属性前过滤其他 HWND，没有读真实打开窗口内容。

### 退出对照

基线主入口来自 `0b5412e`，当前组来自最终桌面构建；均在 ASAR 分发目录运行。固定 3 对，顺序 baseline1/current1/current2/baseline2/baseline3/current3。口径为父进程收到退出请求标记至子进程 exit 的毫秒数。

| 组 | 1 | 2 | 3 | 中位 | 最大 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 接线前 | 382.8722 | 285.4000 | 136.2334 | 285.4000 | 382.8722 |
| 接线后 | 187.5449 | 169.7747 | 215.3089 | 187.5449 | 215.3089 |

每轮退出后等待 2 秒，只检查测试壳报告的 renderer/utility 等 PID，全部无残留。原生合成 source 的 hook/sample/销毁组合约 17.0852 ms；定时器与监听的精确清理由单元测试检查。不能把 3 对样本解释为性能改善。

边界：禁止展示窗口，截获原有 setLoginItemSettings 调用，不写用户/系统设置；supervisor 自动启动关闭，没有覆盖 shell.spec.ts:273 的完整监督进程树偶发失败，该问题未宣告修复。既有 Electron/Playwright 全套未运行，以免执行其中现有的设置写入路径。分发目录为 `D:\Dev\cache\P6-08\package-smoke-05\distribution`，不是签名安装器。

## 隐私、规格与集成边界

- 不截屏、不读键盘输入内容，GetLastInputInfo 只取时间戳。标题读取前/后及发送前查当前 T1；主入口默认拒绝，未接授权 UI，不从布尔设置伪造授权。
- 几何仅到 stage.desktop.update。观察按 allowlist 只含 kind/processName/title/dnd/idleMs；核心传输由终端所有者接线，本卡不复制 IDL、不打开权威存储。
- quietHours 用动态设置查询注入，主入口读取既有标记；设置尚无时段字段，时段编辑/计划解析留给 P6-11，不自造设置 schema。
- 公开通知状态不是已核实的专注开关；未授权时 Teams/Meet 标题特征不可用，不宣称完整会议识别。
- 不实现缺席、预期未发生、被冷落事件，不赋予情绪/证据意义；P5-33 决定仍交给用户。
- I18：只有瞬态采样/去重/门控，无新增持久行为信息。I29：合成测试与临时 userData，无排练/正式用户数据。无 IDL、黄金向量、规律、规格或契约变更。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| koffi | 3.3.3 | MIT | Win32 FFI |
| @koromix/koffi-win32-x64 | 3.3.3，Koffi 可选依赖 | MIT | Windows x64 预编译二进制 |

contracts/zod 为既有工作区依赖，只增加本包登记。LICENSE 全文和固定版本出处见 FFI-AND-LICENSE。所有 FFI 在单一 win32.ts；超过 400 行是为满足单边界条件，其他职责已拆分。

## 失败保留

- 首次 CPU 实际失败；首版 JSON 在断言前误写 passed，以数值、退出码 1 和断言为准，原件未改，后续生成器已修正。
- lint-01 的绑定变量/异常恢复写法已修正；当前静态检查通过。
- package-smoke-01 是依赖定位脚本错误；02 继承 ELECTRON_RUN_AS_NODE 导致无法启动；03 漏复制原生可选包；04 能加载 ASAR 但 isPackaged=false；全部保留。05 使用命名分发程序和完整本机原生依赖后成功。未把早期失败当通过。

## CPU 方法预登记
用户批准的新方法已在 `7b41d65` 提交推送：30 秒预热、60 秒采样、3 对固定顺序，3 次单核原始占用均须 ≤1%，逐次列原始值、对照和差值。尚未通知空闲，不运行测量，不以退出时间代替 CPU 结果。

只完成测量入口编译，没有启动测量进程：源提交 `1a3fddb`，产物 `D:\Dev\cache\P6-08\cpu-compiled-1a3fddb\measure.mjs`，SHA-256 `26b4ec57bf436f0504c5d1ce81ce69ec64031f2351528da709f8ab93313cbb20`；命令与构建 manifest 见 `evidence/cpu-build-1a3fddb.txt`。正式运行前仍须核对用户空闲通知与代码/产物哈希，不使用 tsx 测量。

## 待用户确认

- [ ] （阻塞）用户通知 CPU 空闲窗口，按已登记方法执行并保留全部结果。
- [ ] （阻塞）专注模式可靠来源及未授权会议特征缺失如何处理尚未决定，不放宽矩阵验收。
- [ ] （阻塞）末轮全量 supervisor 失败待核验，不宣称全量通过。

## 后续建议（不在本任务范围内）

- P6-11 接授权来源和时段设置；后合并者 fetch main 并解决集中主入口冲突。
- P7 处理安装器、签名、ASAR 完整性与精确 native 解包规则，并重跑实际分发加载/退出检查，见 FFI-AND-LICENSE。
- 完整 supervisor 进程树退出及 shell.spec.ts:273 偶发失败需专门复现，本卡不改监督模块/阈值。
