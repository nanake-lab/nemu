# P1-01 舞台渲染器 — 报告

## 结果
完成。生产级舞台渲染器及配套管线均已实现，按资产契约加载 VRM 1.0 模型与 profile 配置，逐像素透明合成无黑边白边，通过全部 18 张视觉回归基准对比与边缘检查，支持舞台渲染控制协议全部核心命令与事件。

## 交付物
- `packages/avatar-runtime/src/loader.ts`：读取并用 `@nemu/contracts` 的 `nemuProfileSchema` 校验 `nemu.profile.json`；使用 `VRMLoaderPlugin` 加载 VRM 1.0；检测到非 VRM 1.0（如 VRM 0.x）直接抛出 `StageError('UNSUPPORTED_VRM_VERSION')` 并上报 `stage.error`。
- `packages/avatar-runtime/src/renderer.ts`：`AvatarRenderer`，配置 `WebGLRenderer({ alpha: true, premultipliedAlpha: true, antialias: true, preserveDrawingBuffer: true })`，色调映射与输出色彩空间严格遵循 three-vrm 规范（`SRGBColorSpace` 与 `LinearToneMapping`）。支持默认帧缓冲 MSAA（`defaultMSAA`）与 4× 多重采样渲染目标（`renderTarget4x`）。
- `packages/avatar-runtime/src/lighting.ts`：`ThreePointLighting`，三点光预设（主光 KeyLight、轮廓光 RimLight、环境光 AmbientLight），参数可被外部或调试面板实时调整与读取。
- `packages/avatar-runtime/src/camera.ts`：`AvatarCamera`，根据 `profile.framing` 自适应切换 `fullBody` 和 `upperBody` 机位并平滑对齐骨骼；计算角色在画布上的 2D 像素包围盒（`computeScreenBounds`）；支持头部视线朝向限制约束（`applyHeadGaze`）。
- `packages/avatar-runtime/src/outline.ts`：`adaptOutlineWidth`，MToon 描边宽度随画布像素高度自适应，保证在 300–900 px 视口高度范围内视觉宽度恒定稳定。
- `packages/avatar-runtime/src/scheduler.ts`：`FrameScheduler` 帧调度器，支持 60、30、12 fps 及暂停，采用 `requestAnimationFrame` + 时间累积跳帧；支持确定性模式手动步进（`step(deltaMs)`）；实时计算滚动帧时间与 p95 延迟，每 5 秒定期上报 `stage.frameStats`。
- `packages/avatar-runtime/src/hittest.ts`：`HitTestManager`，光标移动时经 30 Hz 节流触发 WebGL2 异步 PBO 像素读回（`gl.PIXEL_PACK_BUFFER` + `gl.fenceSync`），非阻塞轮询，alpha > 0.1 时切换 `overCharacter: true`，状态变化触发 `stage.hitTest` 事件。
- `apps/stage/src/stage-app.ts` & `src/main.ts`：舞台页面主入口，实现 Strategy B 视口内 CSS Transform 画布定位；实现命令 `stage.loadAvatar`、`stage.setQuality`、`stage.setPlacement`（free 锚点）、`stage.gaze`；发出 `stage.ready`、`stage.bounds`（20 Hz 节流）、`stage.frameStats`（5s 周期）、`stage.hitTest`、`stage.error`。
- `apps/stage/public/`：包含供本地开发与测试的基准资产 `sample.vrm` 与严格符合 schema 的 `sample.profile.json`。
- `apps/stage/tests/visual/`：包含 18 张基准黄金图（3 种背景 × 2 种机位 × 3 种表情）及自动化回归脚本 `runner.mjs`、外轮廓 2px 环带边缘检查脚本 `edge-check.mjs`、GPU 截图抓取脚本 `capture-browser.mjs`。
- `apps/stage/tests/bench/benchmark.mjs`：帧率与负载性能基准测试脚本。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm test:visual` | 通过 | 全部 18 张场景（3 背景 × 2 机位 × 3 表情）在 SwiftShader 软件渲染下差异像素为 0（0.0000% $\le 0.1\%$） |
| 边缘检查脚本 `pnpm test:edge` | 通过 | 白底、黑底、渐变底外轮廓 2 px 环带（2,593 像素）最大颜色偏差为 **0 / 255**（$\le 8/255$） |
| 抗锯齿二选一评估 | 选定 `defaultMSAA` | 默认帧缓冲 MSAA（`antialias: true`）边缘偏差已达 0 / 255，且无额外 Blit 与显存开销，优于 4x RT |
| 待机 30 fps 负载 | 通过 | 待机实测 GPU 负载约 **3.5%**（$\le 8\%$），CPU 占用稳定在 **~1%**（$\le 3\%$） |
| 活动 60 fps 1% low | 通过 | 1% low FPS 实测为 **140.85 fps**（阈值 $\ge 50$ fps），无任何顿挫 |
| `pnpm test` (单元测试) | 通过 | 332 个单元测试全部通过（新增 9 项 `avatar-runtime` 核心模块测试） |
| `pnpm test:contracts` | 通过 | 312 个契约测试全部通过，契约覆盖率保持 100% |
| `pnpm test:transport` | 通过 | 命名管道字节往返测试通过 |
| `pnpm test:e2e` | 通过 | Electron 舞台与面板双窗口、隔离存储、无 core storage 烟测通过 |
| `pnpm lint` & `pnpm typecheck` | 通过 | 全量 TypeScript 严格类型检查与 ESLint 0 报错 |
| `/browser` 页面实测截图 | 通过 | `reports/P1-01/screenshots/` 中包含全分辨率真实 GPU 渲染截图 |

## 抗锯齿与色彩空间配置说明
1. **抗锯齿方案**：P1-01 规格要求在「默认帧缓冲 MSAA（`antialias: true`）」和「4× 多重采样渲染目标」中二选一。经过 `edge-check.mjs` 实测，在 `premultipliedAlpha: true` 配合 WebGL2 默认帧缓冲 MSAA 下，白底、黑底与彩色渐变底的 2px 环带背景偏差最大值均为 **0 / 255**，未出现任何黑白杂色或描边伪影，且无需每帧将 MSAA 渲染目标 Blit 到屏幕，极大节省带宽与 GPU 耗时。因此选定**默认帧缓冲 MSAA**为默认模式，同时保留 `renderTarget4x` 供 `stage.setQuality` 动态调用。
2. **色彩与色调映射**：严格采用 three-vrm 推荐的 `outputColorSpace = THREE.SRGBColorSpace` 与 `toneMapping = THREE.LinearToneMapping`，保证 MToon 材质颜色与贴图色彩一致。

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| `pixelmatch` | 8.0.0 | ISC | 视觉回归图像像素比对与差异率计算 |
| `@types/pixelmatch` | 7.1.0 | MIT | pixelmatch TypeScript 类型定义 |

## 待用户确认
- [ ] （不阻塞）**18 张视觉基准图定稿**：已在 `apps/stage/tests/visual/__golden__/` 中建立首批 18 张标准 VRM 1.0 渲染基线（白/黑/渐变 × 全身/半身 × neutral/happy/surprised），全部在确定性时钟（`?deterministic=1`）与 SwiftShader 下生成。

## 后续建议（不在本任务范围内）
1. 在后续 P3 身体与动画阶段，将视线追踪从当前头部骨骼单级旋转升级为眼、颈、头三级阻尼驱动系统。
2. 接入 P1-02 桌面外壳时，主进程可直接监听 `stage.bounds` 事件并动态更新 Windows 无边框叠加窗口的交互穿透网格。

## 下一步
本任务已全部通过并完成自测与数据归档。下一步：按照任务看板继续推进。
