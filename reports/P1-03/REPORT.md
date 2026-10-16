# P1-03 评审台 (Review Bench) — 报告

## 结果
完成。在浏览器中成功建立完整二次元 3D 角色资产审美与动态评审台，支持 `pnpm dev:review` 启动，全功能端到端验证通过，并已成功导出评审记录与完整测试截图证据。

## 交付物
- `apps/stage/review.html`：评审台独立 UI 页面，具备 3D 视口、对照模式分屏容器、浮动工具条与侧边控制栏。
- `apps/stage/src/review/style.css`：二次元深色工作室视觉设计系统。
- `apps/stage/src/review/turntable.ts`：转台控制器，支持自转、手动拖拽轨道球、平移与滚轮缩放、正/3/4/侧/背预设视角。
- `apps/stage/src/review/lighting-bg.ts`：5 种光照预设（三点、棚拍、戏剧、微光、平光）与 5 种背景预设（白、黑、渐变、棋盘格、壁纸）。
- `apps/stage/src/review/spring-bones.ts`：VRM 弹簧骨实时调参器，支持刚度、阻力、重力调节与 `build.yaml` 导出。
- `apps/stage/src/review/review-app.ts`：评审台核心控制协调器，统一桥接 Three.js、AvatarRenderer、AnimationCompositor、表情网格（单表情与 12 种 emo）、身体读出参数滑动预览、动作播放控制、截图导出与本地评审报告 API。
- `apps/stage/src/review/main.ts`：UI 事件绑定与应用入口。
- `apps/stage/src/review/review.test.ts`：转台、光照背景、弹簧骨调参器单元测试（6/6 通过）。
- `apps/stage/tests/visual/review-bench-e2e.mjs`：端到端全功能自动化验证与截图录制套件。
- `apps/stage/public/sample-bible-ref.svg`：设定集参考图矢量图样本（供对照模式左右分屏比对）。
- `apps/stage/public/motions/manifest.json`：动作清单条目。
- `reports/review-2026-10-16.md`：导出的评审记录真实样例。
- `reports/P1-03/screenshots/`：16 张全功能功能演示截图。

## 验收
| 命令或检查（含任务卡验收要求） | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 (0 warnings, 0 errors) | CI & 本地全绿 |
| `pnpm typecheck` | 通过 (0 errors) | TypeScript 5.8 严格检查无报错 |
| `pnpm test` (包含 `review.test.ts`) | 通过 (14 suites, 357 tests) | 6/6 review bench 单元测试通过，全工作区测试无破坏 |
| `pnpm test:visual` | 通过 (18/18 视觉用例 0 diff) | 视觉回归对比全部 0 像素偏差 |
| `node apps/stage/tests/visual/review-bench-e2e.mjs` | 通过 (全交互路径自动遍历) | 产出 16 张高分辨率功能截图并生成报告 |
| 转台功能（自动旋转、拖拽轨道、正/3/4/侧/背预设视角） | 通过 | `01_review_bench_overview.png`, `02_turntable_threeQuarter_45deg.png`, `03_turntable_side_90deg.png`, `04_turntable_back_180deg.png` |
| 光照与背景预设切换 | 通过 | `05_lighting_studio_white_bg.png`, `06_lighting_dramatic_black_bg.png`, `07_checker_bg.png`, `08_wallpaper_bg.png` |
| 设定集对照模式（左参考图，右 3D 渲染实时联动） | 通过 | `09_comparison_mode_split_view.png` |
| 表情网格（单个基础与 Nemu 特殊表情、12 种 emo 组合表演提示） | 通过 | `10_expression_happy.png`, `11_expression_surprised.png`, `12_expression_emo_smug.png` |
| 身体读出预览（实时滑块驱动 posture、earPose、tailAmplitude 等） | 通过 | `13_body_readout_preview.png` |
| 动作播放器（播放、暂停、循环切换、0.25x/0.5x/1.0x 变速） | 通过 | `14_motion_player_wave.png` |
| 弹簧骨参数面板（实时调参与 `build.yaml` 代码导出） | 通过 | `15_spring_bone_inspector.png` |
| 评审记录导出（通过本地 Vite 开发接口写入 `reports/review-<YYYY-MM-DD>.md`） | 通过 | `16_review_notes_saved.png` 与 `reports/review-2026-10-16.md` |
| 截图导出功能（带视图参数命名的 PNG 导出） | 通过 | `ReviewApp.captureScreenshot()` 实现命名规则 `nemu_review_deg-{angle}_expr-{expr}_bg-{bg}.png` |

## 新增依赖
无新增外部生产运行时依赖（基于项目中已有的 `@pixiv/three-vrm`、`three`、`vite`、`zod` 实现）。

## 待用户确认
- [ ] （不阻塞）评审台默认机位距离与聚焦高度（`heightM * 0.55`，距离 `heightM * 2.3`）可根据后续实际资产立绘比例按需微调。
- [ ] （不阻塞）对照模式后续可支持动态上传用户自己的本地参考图文件。

## 后续建议（不在本任务范围内）
1. 在 P2-05 中使用本评审台导出的 spring bones 配置写入正式资产 `content/nemu/build.yaml`。
2. 在 P3-03 身体读出联调时，直接借助身体读出控制滑块对比不同生理与心理状态下的呼吸与姿态变化。

## 下一步
分支 `task/P1-03-review-console` 已完全就绪，无阻塞项，准备创建 PR 并合并至 `main`。
后续依照任务路线图取卡：`P3-03 身体读出与待机生命 (Readout & Idle Life)`。
