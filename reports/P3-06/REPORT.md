# P3-06 身体读出 v2 — 报告

## 当前状态

实现、本地静态检查、551 项单测、构建、原有 18 组视觉回归及两段视频的实际解码复核已通过。PR #76 转为可审查、标 `needs-user`，等待用户另开的 Codex 会话交叉审查与用户合并。最终提交的 CI 状态以 PR Checks 为准；早期实现提交 `734b1db` 的 Ubuntu CI 已通过，不把它冒充最终提交的 CI。本卡含契约后继入口和 ADR 0022，执行者不合并、不领取下一卡。

起点为 fetch 后的 `origin/main` `0b5412e`；工作区 `D:\Dev\nemu-wt\P3-06`，分支 `task/P3-06-readout-v2`。P5-22 PR #57 已合并，未带 `unverified`。完成阶段再次 fetch/merge，main 无新增提交。

## 交付物

- `packages/body/src/readout/`：仅接收 `BodyReadoutV2`，严格校验、缺失/m = 0/失联回 `(0.5, 0, 0.5)`；约 2 秒固定 Hermite 动画平滑；重复应用不重启、不累积；回调与返回值防御性复制。`StageReadoutInput` 独立处理后继入口、旧入口中性回退和运行状态门控。
- `packages/avatar-runtime/src/animation/`：固定版本纯映射驱动 L2/L4，energy 同时改变呼吸、重心的节奏和幅度；L3 删除表情基线，仅保留 emo、口型与眨眼，句末清除自定义表情残留。逐帧移除上一帧程序偏移，不靠重复叠加姿态；复制 normalized 动画到 raw VRM 骨架。
- `packages/contracts/src/body-animation.ts`：profile 可选 `bodyReadoutMapping` 及其校验、候选初值。版本为 `body-animation-v2-candidate-1`，`calibration: pending`。没有把候选写入正式 profile、build.yaml 或 genesis；最终数值由用户或 P5-17 确定。
- `packages/contracts/src/stage.ts`：只新增 `stage.readout.v2`，载荷 `{ readout: BodyReadoutV2 }`。旧 `stage.readout`、旧 `BodyReadout` 的字段和解码均保留；旧入口在迁移后的客户端明确回中性，不读取旧字段或恢复表情基线。IDL、生成物与冻结字节无改动。
- 评审台：新 `readout-controls.ts` 提供 facing、tension、energy 三滑块、中性重置、内存中的候选映射 JSON 调整。只在共用文件里接入；选择待机动作不再覆写读出。`main.ts`、`stage-app.ts` 只做接线。
- **用户另行批准的卡外一行**：`apps/panel/src/debug/schema-validator.ts` 的穷举表仅追加 `stage.readout.v2` 中性样例，全部旧样例保留。既有穷举 schema 测试通过。授权与原因见 `tasks/requests/P3-06-panel-successor-preset.md`。
- `docs/adr/0022-body-readout-animation-v2.md`：契约兼容、候选依据、目录扩展、回退说明。没有占用 0020/0021，也没有新增差异登记编号。
- `reports/P3-06/preview-final/`、`videos-final/`：白/黑背景候选与两组待机视频；来源、失败历史、限制和复现见 `README.md`。视频采用捕获专用合成基础姿态，不是正式动作库；源 profile 的 `idleSet` 仍为空。其声明在 `fixture-manifest.json`。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm lint` | 通过 | `checks/lint.log` |
| `pnpm typecheck` | 通过 | `checks/typecheck-final.log` |
| `pnpm test` | 47 文件、551 项通过 | `checks/test.log` |
| 读出/动画/后继契约专项 | 41 项通过 | `checks/targeted.log` |
| `pnpm build` | 通过；保留依赖注释位置与 bundle 大小警告，没有放宽阈值 | `checks/build.log` |
| `pnpm test:p306:visual` | 原始 18 组全部 0 像素差；仅重定向报告目录，未更新基准 | `checks/visual-golden.log`、`visual/golden/comparison.json` |
| `pnpm test:p306:preview` | 三滑块/中性重置/白黑截图重复一致；最终夹具双手低于上臂断言通过 | `checks/preview-final.log`、`preview-final/manifest.json` |
| `pnpm record:p306` 与视频解码复核 | A 180.375 秒、B 180.452 秒；哈希一致，抽帧画面已人工检查 | `videos-final/manifest.json`、`videos-final/verification.json` |
| 舞台真实 window-message 接线 | 后继生效、旧入口中性、非法输入拒绝、m = 0 门控通过 | `stage-wire.json`、`checks/stage-wire-final.log` |
| `python core/contracts/check_compat.py --ref origin/main` | p5-01/p5-21/p5-32/current/main，137 消息/51 枚举，0 违规 | `checks/compat.log` |
| 旧 IDL、生成物与 content 指针 | diff 为空；无权威存储被打开 | Git diff 与各次捕获资产哈希 |
| CI | 见 PR #76 的对应提交检查；不把其他提交的 CI 当本提交结果 | GitHub Checks |

### 失败与修正

首次 typecheck 缺少调试面板穷举样例；首次单测 4 项失败：该样例、测试事件使用非 ULID、旧视线延迟测试受新读出偏置影响、把可动视线偏航误当成中性姿态偏移。分别补授权样例、修正测试事件、显式隔离视线偏置测试固定延迟（原阈值保留）、将中性姿态断言对准 pitch。原始日志 `checks/typecheck-first.log`、`checks/test-first.log` 保留，之后 551 项全通过。没有删除断言或降低既有阈值。

首次画面是源模型 T 姿势且脚部被工具栏遮挡（`preview/`）；合成基础姿态初次手臂旋转方向错误（`preview-idle/`）。两次都保留，最终在 `preview-final/` 修正且补几何断言。

首次两段录制墙钟 180 秒，但解码只有 179.602/179.539 秒，透明 WebGL 直接录制还产生边缘花块（`videos/`）。这些不计验收。初始辅助视频核验容差后来收紧为解码必须至少 180 秒；发现花块后中断的一次重录保留 `checks/videos-final.log`。改为不透明 2D 合成，并先完成约 4 秒短片验证（`video-probe/`，不是三分钟交付），再增加一秒录制缓冲。最终视频时长及抽帧由 `videos-final/verification.json` 给出。额外舞台接线脚本初次遇到端口 5187 已占用，未终止占用者；改为系统分配临时端口后通过，失败日志保留在 `checks/stage-wire.log`。

## 规格对照

| 条款 | 实现 | 测试 |
| --- | --- | --- |
| v2.0 §9.3、CONTRACTS §7：三项读出，无表情基线 | v2 应用器、纯映射、L3 | 边界八角点、中性、L3 结构扫描、emo 自定义绑定归零 |
| P3-06：只应用、不保存、不累积；m = 0/无读出中性 | 应用器与 StageReadoutInput | 重复应用、插值中重发、断开/未调制/恢复、非法输入、返回值隔离 |
| P3-06：固定版本映射，参数在 profile，可预览调整 | bodyReadoutMapping schema、readout-mapping、独立预览控件 | 旧 profile 原样解码、候选版本与范围、三滑块/重置 |
| ARCHITECTURE §6.3：L2/L4，L3 表演/眨眼/口型 | compositor、各动画层 | 10,800 帧无基片段漂移、外部姿态保留、禁用层清理、视线/energy 响应 |
| ADR 0007、用户授权：只增不改 | 新 stage.readout.v2；旧 schema 保留 | 新旧载荷各自校验、禁止混用；旧 IDL/生成物 diff 为空 |
| I18 / 黄金规则 4：终端无隐藏持久情绪 | 身体/舞台/运行时/terminal 源码 | TS AST 变量扫描、持久化调用扫描、L3 无基线输入扫描 |

## 新增依赖与范围

没有新增运行时或开发依赖，lockfile 不变。只使用已有 three/three-vrm、zod、Playwright、Vite、Vitest。依赖虚拟仓与 pnpm 存储在 D 盘，使用已有 D 盘 Chromium；未修改用户级或系统级设置。浏览器仅由验收脚本以无界面模式启动，未操作其他应用或联系审查方。

已有大文件继续原位最小修改：L2 从 398 到 460 行，纯映射与帧偏移缓存已放新文件；review-app 从 533 减至 505 行；stage-app 从 409 到 417 行仅接线；动画测试从 620 到 704 行、契约样例测试从 466 到 468 行，保留既有共享夹具与用例，不顺带重构共用文件。

机械性证据整理使用 `routine_reporter` 角色（按角色名加载，本次另读配置，gpt-6-luna/max）：只收集文件、报告、哈希与一次 CI 步骤状态；未委派规格判断、实现或审查，没有子代理启动重负载。主代理核对后采用。

## 待用户确认

- [ ] （阻塞）ADR 0022 与契约后继入口，经用户安排的另一个 Codex 会话交叉审查后，由用户合并批准；`needs-user` 保留。
- [ ] （不阻塞）两组候选读出视频的观感；映射初值待用户或 P5-17 校准，不视为定稿。

## 后续建议（不在本任务范围内）

正式动作库接入后，使用相同两组读出再观察待机小动作；本次合成基础姿态视频不证明正式动作库、小动作资源或整日自然程度。P2-06 与本卡后合并者须 fetch main 并解决评审台冲突。
