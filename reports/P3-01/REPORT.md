# P3-01 动画运行时（五层） — 报告

## 结果
完成。按 ARCHITECTURE.md 6.3 与 tasks/P3-body.md 实现了完整的五层动画运行时系统，五层彼此正交独立、可单独开关，在确定性模式与正常模式下均表现稳定，所有单元测试与视觉回归测试全部通过。

## 交付物
- `packages/avatar-runtime/src/animation/types.ts`：动画层类型定义（`AnimationLayerId`、`VisemeWeights`、`PostureLimits`、`TwoBoneIKParams` 等）。
- `packages/avatar-runtime/src/animation/math.ts`：数值与运动学工具库：
  - `CriticallyDampedSpring`：解析式临界阻尼二阶弹簧滤波（$\zeta = 1.0$），保证零超调与无条件数值稳定性；
  - `DelayBuffer`：环形时延缓冲区，供头颈视线延迟采样；
  - `smoothHermite`：三次 Hermite 平滑加权曲线（$3t^2 - 2t^3$），端点导数为 0；
  - `alignLoopHeadToTail`：循环片段首尾姿态自动对齐（旋转差 $\le 2^\circ$）；
  - `solveTwoBoneIK`：基于余弦定理的解析双骨 IK 求解器。
- `packages/avatar-runtime/src/animation/L0-base.ts`：基础姿态层。
  - 支持通过 `@pixiv/three-vrm-animation` 加载 VRMA 与实例化；
  - `AnimationMixer` 交叉淡化，默认 250 ms，最短强制限幅 150 ms；
  - 循环片段首尾对齐与骨骼角速度跟踪。
- `packages/avatar-runtime/src/animation/L1-gesture.ts`：手势叠加层。
  - 从 `spine` 往上的上半身骨骼遮罩（下半身 hips 与腿脚严格隔离未受任何污染）；
  - 淡入 120 ms、淡出 200 ms 权重曲线；
  - 基于四元数 slerp 叠加在 L0 基础动作之上。
- `packages/avatar-runtime/src/animation/L2-procedural.ts`：程序化身体层。
  - 呼吸：chest、upperChest 周期性旋转（3.5–4.5 s），节奏乘以身体读出 `idleTempo`；
  - 姿态偏移：按身体读出 `posture`（-1..1）给 spine、chest、neck、肩部叠加静态姿态偏移，处于呼吸与重心摆动之下；
  - 重心摆动：hips 慢速摆动（8–14 s 周期）；
  - 分级视线：目标点 $\to$ 眼（权重 1.0，延迟 0 ms）、头（0.6，延迟 120 ms）、颈（0.3，延迟 200 ms），临界阻尼平滑并遵守 `profile.gazeLimits`；
  - 双骨 IK：`solveTwoBoneIK` 与 `applyTwoBoneIK` 辅助工具。
- `packages/avatar-runtime/src/animation/L3-face.ts`：面部表情与眨眼层。
  - 双通道表情输入：`expressionBaseline`（0..0.4）与表演提示 `emo`（经 `profile.emotionMap` 展开），说话时以表演提示为主，句末平滑回落至基线；
  - 眨眼控制器：2–6 s 随机间隔，15% 概率连眨；`surprised` 状态下阻止眨眼；
  - 口型通道：接收 `aa ih ou ee oh` 权重，说话期间把表情权重上限压到 0.6，说完后 300 ms 内平滑恢复。
- `packages/avatar-runtime/src/animation/L4-secondary.ts`：次级运动层。
  - 耳朵偏移：`earPose` 映射到 `rig.earBackMaxDeg`；
  - 尾巴摆动：`tailAmplitude` 映射到 `rig.tailSwingDeg`；
  - 偏移在弹簧骨更新之前叠加到根骨上；
  - 支持表演提示临时覆盖（`setEarOverride` / `setTailOverride`）。
- `packages/avatar-runtime/src/animation/compositor.ts`：五层动画合成器。
  - 每帧按 L0 $\to$ L1 $\to$ L2 $\to$ L3 $\to$ L4 严格顺序合成；
  - 暴露 `setLayerEnabled(layer, on)` 与 `isLayerEnabled(layer)`。
- `apps/stage/src/stage-app.ts` & `apps/stage/src/main.ts`：舞台接入五层合成器，支持 `stage.readout`、`stage.setLayerEnabled` 与 URL `layers=L0,L1,L2,L3,L4` 调试参数。
- `reports/P3-01/layers/`：每层单独开关对比截图。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm typecheck` | 通过 | 全工作区零类型报错（`tsc --noEmit` 退出码 0） |
| `pnpm lint` | 通过 | ESLint 严格检查全绿，无警告无错误 |
| `pnpm test` | 通过 | 13 个测试套件、351 个单元测试全绿 |
| 权重淡化曲线测试 | 通过 | 验证 L0 $\ge 150$ ms、L1 淡入 120 ms、淡出 200 ms、Hermite 平滑性 |
| 上半身骨骼遮罩测试 | 通过 | 验证 L1 运行时 lowerBody 骨骼（hips、腿、脚）姿态 100% 保持未变 |
| 临界阻尼弹簧与时延平滑测试 | 通过 | 验证零超调单调逼近，眼(0 ms)、头(120 ms)、颈(200 ms)响应分离 |
| 视线限位测试 | 通过 | 验证偏航与俯仰角度严格限制在 `profile.gazeLimits` 范围内 |
| 眨眼间隔与双眨分布测试 | 通过 | 10,000 次采样实测：区间 $[2.0, 6.0]$ s，均值 3.99 s，双眨概率 15.1%，`surprised` 状态下眨眼完全抑制 |
| 无跳帧断言（100 次切换骨骼角速度） | 通过 | 确定性模式下 100 次 L0 片段切换，实测最大角速度 $\le 1.5 \times$ 片段自身最大角速度 |
| 口型抑制与 300 ms 恢复测试 | 通过 | 说话时表情权重上限截断至 0.6，停止说话后 300 ms 内平滑恢复至 1.0 |
| `pnpm test:visual` | 通过 | 18/18 视觉回归测试通过（0 像素差异，0.0000%） |
| 动画层独立开关对比图 | 通过 | 产出 `reports/P3-01/layers/` 下 6 张对比图 |

## 新增依赖
| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| `@pixiv/three-vrm-animation` | `3.5.5` | MIT | 加载 VRMA 动画资源并绑定至 VRM 人形骨骼 |

## 待用户确认
- [ ] （不阻塞）呼吸周期默认 4.0 s，摆动周期 10.0 s，可随时在后续评审台微调。

## 后续建议（不在本任务范围内）
- P3-02 动作管线交付第一批 VRMA 动作后，可直接通过 `compositor.l0.playClip(...)` 注入。

## 下一步
分支 `task/P3-01-animation-layers` 已就绪，准备推送并提交 PR。
根据 `tasks/README.md` 第 2 节，Antigravity 的下一张任务卡为：`P1-03 评审台（Review Bench）`。
