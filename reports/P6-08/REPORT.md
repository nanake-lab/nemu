# P6-08 桌面感知 — 报告

## 当前状态
**实现及用户限定的本机验收完成，等待审查与 CI。** 固定合成工况 CPU 通过；Windows 10 专注识别按用户决定改为只读探针报告、非必过项；supervisor 单独 6/6、当前全量 564/564 通过，原失败保留。PR #75 保留 needs-user（ADR 0023）；按用户要求转为可审查、写「请审查」后停止，不合并、不取下一张卡。工作区 `D:\Dev\nemu-wt\P6-08`，分支 `task/P6-08-desktop-sense`，基线 `0b5412e`。实时 CI 以 PR 的当前提交检查为准，不把本机通过或旧提交 CI 当成当前 CI 全绿。

收尾已 fetch 并 merge 最新 `origin/main` 的 `7f962b2`（含 `9d236d6` CI 超时调整及 P5-33 研究报告）。规则、任务卡和必读文档无新增差异；未实现 P5-33 的缺席或预期事件。合并后的 `7e05796` 上串行重跑 lint、typecheck、全部 564 项测试及完整构建，退出码均为 0；随后只有报告/证据变更。

## 交付物

- `packages/desktop-sense/src/`：单模块 Win32 FFI，PMv2 查询与恢复、DWM 窗口/任务栏/屏幕平台、前景遮挡裁剪、稳定平台 ID、插入点、公开通知状态、会议特征、空闲边沿、节流与清理。
- `apps/desktop/src/sense/`：几何只发舞台，语义走既有 AppBus；锁屏停采样，故障 busy，舞台 ready 后重送当前几何，退出移除全部监听及定时器。
- `main/index.ts`：获用户授权的集中启动/停止/错误处理；启动失败记 error、发 busy，壳继续。退出保护覆盖 supervisor 尚未建立而感知已启动的情况。
- 用户追加授权的 scaffold 保护：原有 setLoginItemSettings 调用加 `if (!scratch)`；这是原入口行为的测试路径修复，**不是本卡引入登录项写入**，产品无 scratch 的行为不变。实际打包启动测试断言 scratch 调用次数为 0。
- `electron.vite.config.ts`：只在获准的两个数组追加桌面感知包与 koffi；前者编入主进程，后者作为原生依赖加载。
- [规格对照](../../docs/traceability/P6-08.md)、[FFI/许可/P7 后续](FFI-AND-LICENSE.md)、[测前提交的 CPU 方法](CPU-PROTOCOL.md)、`evidence/` 原始证据。

## 验收
| 命令或检查 | 结果 | 证据（evidence/ 下） |
| --- | --- | --- |
| `pnpm test:desktop-sense` | 6 文件、30 测试通过 | unit-focus-20261024.txt |
| `pnpm lint`、`pnpm typecheck` | 同步最新 main 后通过 | lint-synced-20261024.txt、typecheck-synced-20261024.txt |
| `pnpm build` | 同步最新 main 后完整构建通过；保留现有 Zod PURE 注释告警 | build-synced-20261024.txt |
| `pnpm sense:geometry …/native-geometry-current.json` | 8 个原生合成窗口，100/125/150/200% 物理尺寸、两个合成显示器范围，边误差均 0 px | native-geometry-current.json/.txt |
| 勿扰桩矩阵 | 全屏/PPT/会议/通知 busy 各 10 次断言通过；不等于真实专注开关验收 | semantics.test.ts、unit-28.txt |
| 隐私/边沿/清理 | 未授权不读取标题，撤回不附带标题、不冒充会议结束；几何不进总线；锁屏停采样；监听和定时器清零 | engine.test.ts、sense/index.test.ts |
| 打包后 Electron/Koffi | isPackaged=true；Electron 44.7.0、Node 24.21.0、modules 149、N-API 10；原生 x64 二进制从 app.asar.unpacked 找到并调用成功；合成 sample、hook 注册/卸载及销毁正常 | package-smoke-06.json/.txt |
| 接线前后退出 / scratch 设置保护 | 见下表；6 次退出码 0，2 秒后无测试子进程残留；当前 scratch 三次 setter 调用数全部 0，基线三次为 1（拦截不透传） | package-smoke-06.json/.txt |
| 首轮全量 `pnpm test` | 当时 559/559 通过 | test-01.txt |
| 原全量失败（保留） | **559/560，失败**：supervisor.test.ts:64，restartCount=0，期望 ≥1；未改该文件/阈值 | test-final.txt |
| supervisor 阻塞核验 / 当前全量 | 用户通知空闲且 CPU 测完后，原测试独立 6/6；当时全量 562/562；新增探针测试并同步最新 main 后当前 **564/564** | supervisor-isolated-20261024.txt、test-quiet-20261024.txt、test-synced-20261024.txt |
| CPU ≤1%，登记工况 | 原始单核三次 **0.468334%、0.128333%、0.494988%**，全部 ≤1%；空载均 0%，不扣除空载替代判据；原 2.339468% 失败并列保留 | cpu-quiet-20261024T081456Z/、CPU-RESULTS-20261024.md |
| Windows 10 专注只读探针（非必过） | build 19045，返回码0、4字节、值0，按映射 inactive；未切换设置验证。Windows 11 仅公开读分支，桩验证，不冒充实机 | focus-probe-19045-20261024.json/.txt、FOCUS-SOURCE.md、ADR 0023 |
| 登录项只读检查 | 名称/命令/快捷方式目标匹配 Electron/Nemu 的 Run 与 Startup 条目均 0；没有删除。只有键级时间，不能归因历史 CI/e2e | startup-audit-20261024.json、STARTUP-AUDIT-20261024.md |

坐标证据使用真实 DWM 查询自行创建的窗口，显示器/DPI 为合成数据；未改变实际显示器或缩放，不外推物理双屏现场验收。白名单在读取窗口属性前过滤其他 HWND，没有读真实打开窗口内容。

### 退出对照

基线主入口来自 `0b5412e`，当前组来自最终桌面构建；均在 ASAR 分发目录运行。固定 3 对，顺序 baseline1/current1/current2/baseline2/baseline3/current3。口径为父进程收到退出请求标记至子进程 exit 的毫秒数。

| 组 | 1 | 2 | 3 | 中位 | 最大 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 接线前 | 76.7206 | 57.6838 | 92.1242 | 76.7206 | 92.1242 |
| 接线后 | 58.6513 | 61.0997 | 60.5944 | 60.5944 | 61.0997 |

每轮退出后等待 2 秒，只检查测试壳报告的 renderer/utility 等 PID，全部无残留。最终分发包原生合成 source 的 hook/sample/销毁组合约 12.9181 ms；定时器与监听的精确清理由单元测试检查。不能把 3 对样本解释为性能改善。旧 05 组原始数据保留，不选择最好一轮。

边界：禁止展示窗口，测试始终拦截 setter；当前代码 scratch 不再调用它，基线调用不透传，因此不写用户/系统设置。supervisor 自动启动关闭，没有覆盖 shell.spec.ts:273 的完整监督进程树偶发失败，该问题未宣告修复。分发目录为 `D:\Dev\cache\P6-08\package-smoke-06\distribution`，不是签名安装器。完整 Windows 检查由转审后 CI 执行；未在本报告里预判其结果。

## 隐私、规格与集成边界

- 不截屏、不读键盘输入内容，GetLastInputInfo 只取时间戳。标题读取前/后及发送前查当前 T1；主入口默认拒绝，未接授权 UI，不从布尔设置伪造授权。
- 几何仅到 stage.desktop.update。观察按 allowlist 只含 kind/processName/title/dnd/idleMs；核心传输由终端所有者接线，本卡不复制 IDL、不打开权威存储。
- quietHours 用动态设置查询注入，主入口读取既有标记；设置尚无时段字段，时段编辑/计划解析留给 P6-11，不自造设置 schema。
- 公开通知状态不冒充专注开关。按用户批准，WNF 只作为独立可行性探针、Windows 10 非必过；未知不当 inactive、不报错/崩溃。Windows 11 探针仅走公开 FocusSessionManager，不回退 WNF。探针未接正式勿扰循环；未授权时 Teams/Meet 标题特征不可用，不宣称完整会议识别。
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
用户批准的新方法已在 `7b41d65` 测前提交推送，方法文件保持不改。用户 2026-10-24 通知空闲后完成 30 秒预热、60 秒采样、3 对固定顺序；全部原始感知占用 ≤1%，逐次列原始值、对照和差值，见 [CPU-RESULTS-20261024.md](CPU-RESULTS-20261024.md)。旧 2.34% 失败保留。

源提交 `1a3fddb`，产物 `D:\Dev\cache\P6-08\cpu-compiled-1a3fddb\measure.mjs`，SHA-256 `26b4ec57bf436f0504c5d1ce81ce69ec64031f2351528da709f8ab93313cbb20`，测前核对一致；构建记录见 cpu-build-1a3fddb.txt，运行未使用 tsx。随后新增诊断探针不在正式采样路径，新增探针后的发布 bundle 与原包字节相同，见 focus-production-bundle-comparison-20261024.json。之后仅修 scaffold 启动设置路径，不改变测量的感知循环。

## 待用户确认

- [x] 用户空闲授权后完成固定 CPU 测量；全部原始结果及旧失败保留。
- [x] 用户批准 Windows 10 只读探针、非必过验收边界，记入 ADR 0023；其余勿扰矩阵不放宽。
- [x] supervisor 原测试在空闲环境通过，当前全量 564/564；未改代码/阈值。资源争用只是可能解释，不把未复现当成原因已证明。
- [x] 用户批准 scaffold 登录项保护；实际包断言调用数为 0；只读检查发现 0 个匹配项，不声称历史从未写过。
- [ ] （合并前）用户安排交叉审查，当前提交的 CI 检查完成；新增 ADR 保留 needs-user，由用户合并，不自行批准或合并。

## 后续建议（不在本任务范围内）

- P6-11 接授权来源和时段设置；后合并者 fetch main 并解决集中主入口冲突。
- P7 处理安装器、签名、ASAR 完整性与精确 native 解包规则，并重跑实际分发加载/退出检查，见 FFI-AND-LICENSE。
- 完整 supervisor 进程树退出及 shell.spec.ts:273 偶发失败需专门复现，本卡不改监督模块/阈值。
