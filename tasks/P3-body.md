# P3 身体与动画（第 3–20 周）

出口条件：P3-01 至 P3-04 并入 N2（排练版需要完整的身体和第一批基础动作）；P3-05 动作库 v1 与 P3-06 身体读出 v2 并入 N3。五层动画运行时、身体读出、物理状态机与反射、身体动作执行、动作管线全部可用；动作库 v1 的 30 个片段由你选定。

身体只负责反射和表现，不保存任何会影响她行为的状态（AGENTS.md 黄金规则 4）。她做什么、什么时候主动，由 Echo Core 决定。

---

## P3-01 动画运行时（五层）

- **建议执行者：** Antigravity
- **难度：** 中
- **里程碑：** N1 认得出是她
- **依赖：** P1-01
- **工期：** 8 天（前 4 天先交付 L2 呼吸与视线、L3 眨眼，供 N1 使用）
- **分支：** `task/P3-01-animation-layers`
- **目录：** `packages/avatar-runtime/src/animation/`

### 目标
按 ARCHITECTURE.md 6.3 实现五层动画，每层独立、可单独开关，在确定性模式下结果可复现。

### 交付物
- `L0 base.ts`：基础姿态层。
  - 用 three-vrm-animation 加载 VRMA；
  - `AnimationMixer` 交叉淡化，默认 250 ms，最短 150 ms；
  - 循环片段的首尾自动对齐。
- `L1 gesture.ts`：手势层。上半身骨骼遮罩（从 spine 往上），按权重叠加在 L0 之上，淡入 120 ms、淡出 200 ms。
- `L2 procedural.ts`：
  - 呼吸：chest、upperChest 的小幅旋转，周期 3.5–4.5 s，节奏乘以身体读出的 `idleTempo`；
  - 姿态偏移：按身体读出的 `posture` 给 spine、chest、neck、肩部加静态偏移（负值收拢、正值舒展），叠加在呼吸和重心摆动之下；偏移幅度上限写在 `profile` 里，可在评审台调整；
  - 重心：hips 的慢速摆动，周期 8–14 s；
  - 视线：目标点 → 眼（权重 1.0，延迟 0）、头（0.6，延迟约 120 ms）、颈（0.3，延迟约 200 ms），用临界阻尼弹簧平滑；遵守 `profile.gazeLimits`；
  - IK：两骨 IK 工具函数（腿、手），供坐窗沿使用。
- `L3 face.ts`：
  - 表情输入有两路：身体读出的 `expressionBaseline`（P3-03 提供，本卡先用可手动设置的桩），以及表演提示 `emo`（经 `profile.emotionMap` 展开）。说话时以表演提示为主，句末按固定曲线回到基线；
  - 眨眼控制器：间隔 2–6 s 随机，15% 概率连眨；`surprised` 时阻止眨眼；
  - 口型通道：接收 `aa ih ou ee oh` 权重（P4-04 提供）。说话期间把表情权重上限压到 0.6（可在评审台调整），说完 300 ms 内恢复。原因见 CONTRACTS.md 1.3。
- `L4 secondary.ts`：three-vrm 弹簧骨更新；耳朵和尾巴的姿态偏移在弹簧骨更新之前叠加到根骨上。
  - 耳朵：`earPose` 映射到 `rig.earBackMaxDeg` 的范围；
  - 尾巴：摆幅按 `tailAmplitude` 映射到 `rig.tailSwingDeg`；
  - 表演提示可以短暂覆盖这两项。
- `compositor.ts`：按 L0 → L1 → L2 → L3 → L4 的顺序每帧合成；暴露 `setLayerEnabled(layer, on)` 供调试。

### 验收
- 单元测试：淡化权重曲线、遮罩、弹簧平滑、视线限位、眨眼间隔分布（10,000 次采样的均值和区间）。
- 无跳帧断言（完成标准第 7 项）：
  - 在确定性模式下自动切换 L0 片段 100 次，记录每帧所有骨骼的角速度；
  - 切换期间最大角速度不超过片段自身最大角速度的 1.5 倍。
- 评审台录屏：每层单独开关的对比。

> **2026-10-20 追加（v2.0）：** 身体读出改为三项（朝向你、放松或紧绷、精力，P5-22）。`idleTempo`、`posture`、`earPose`、`tailAmplitude`、`gazeSociability` 改由这三项经固定映射得出，L3 的表情基线去掉，说话之外的表情只来自表演提示。这些修改在 P3-06。

---

## P3-02 动作资产管线

- **建议执行者：** Codex
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P2-03
- **工期：** 5 天
- **分支：** `task/P3-02-motion-pipeline`
- **目录：** `tools/asset-pipeline/motion/`

### 目标
把任意来源的动作（BVH、FBX、GLB、VRMA）转换成 VRMA，并登记许可。

VRMA 定义在 VRM 标准人形骨骼上，运行时由 three-vrm-animation 适配到任何 VRM 角色，所以这条管线不需要等 Nemu 的模型完成。穿模、脚底滑动等与体型有关的问题，等 Nemu 模型出来后在评审台里检查。

### 交付物
- `pnpm motion:import <源文件> --id <动作id> --source <说明> --license <SPDX或custom> [--loop] [--mask upper]`，依次执行：
  1. 在 Blender 中导入源动作；
  2. 按骨骼映射表（`motion/bone-maps/*.yaml`，至少包括 Mixamo、Quaternius、SMPL、VRM）重定向到一个参考 VRM 的标准人形骨骼；
  3. 清理：提取根运动、可选脚部锁定、去抖（低通滤波）；循环片段首尾对齐；
  4. 导出 VRMA 到 `content/_candidates/motions/`；
  5. 写入 `motions/manifest.json` 草稿条目（`redistributable` 必填）。
- `pnpm motion:preview <id>`：生成转台预览 GIF，放进报告。
- 第一批：从 Quaternius Universal Animation Library（CC0）挑 10 个基础动作（待机、走、坐、起身、挥手、点头、摇头、欢呼、思考、伸懒腰）转换。
- CI 规则：manifest 中缺少 `license` 或 `redistributable` 的条目使 `asset-lint` 报错。

### 验收
- 10 个动作在评审台播放正常，没有明显的脚底滑动（站立类动作中，脚骨水平位移 ≤ 2 cm）。
- 循环片段首尾帧的骨骼旋转差 ≤ 2°。

### 禁止
- 不要把 Mixamo、BOOTH 或其他禁止再分发的原始文件提交到公开仓库；只能放在私有的内容仓库里，并标记 `redistributable: false`。

---

## P3-03 身体读出与待机生命

- **建议执行者：** Antigravity
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P3-01、P0-02
- **工期：** 4 天
- **分支：** `task/P3-03-readout-idle`
- **目录：** `packages/body/src/readout/`、`packages/body/src/idle/`

### 交付物
- **读出应用器：** 接收 `stage.readout`，把 `BodyReadout`（CONTRACTS.md 第 7 节）映射到 L2、L3、L4 和待机调度的参数。
  - 只应用，不保存、不累积：同一个读出应用两次，结果与应用一次相同；
  - 新旧读出之间的过渡是动画层面的固定平滑（约 2 s），不是状态；
  - 没有读出时（核心未连接、`CORE_UNMODULATED`、`CORE_QUARANTINED`）使用中性值。
- **待机调度：**
  - 按 `profile.idleSet` 的权重随机抽取小动作，带冷却时间；
  - 同一动作两次出现的间隔 ≥ 冷却时间；连续 3 次抽取不能相同；
  - 抽取节奏乘以 `idleTempo`；
  - 随机数用可注入的随机源，确定性模式下固定种子。
- 评审台的「身体读出预览」接入本卡的应用器。

### 验收
- 单元测试：
  - 读出映射的边界值；
  - 中性读出得到中性姿态；
  - 应用的幂等性；
  - 结构检查：终端代码里没有以情绪、心情、好感命名的持久变量。
- 10 分钟待机采样（完成标准第 6 项的自动部分）：记录动作序列，不能出现长度 ≥ 3 的重复子序列连续出现两次。
- 评审台录两段 3 分钟待机视频（两组不同的读出），交给你看。

> **2026-10-20 追加（v2.0）：** 读出应用器改读 BodyReadout v2，由 P3-06 完成。「没有读出时使用中性值」不变；`CORE_QUARANTINED` 在 v2.0 中不再出现。

---

## P3-04 物理状态机、反射与身体动作

- **建议执行者：** Antigravity
- **难度：** 中
- **里程碑：** N2 她在了（排练版）
- **依赖：** P3-03
- **工期：** 5 天
- **分支：** `task/P3-04-physical-states`
- **目录：** `packages/body/src/fsm/`、`packages/body/src/reflex/`、`packages/body/src/actions/`

### 交付物
- **物理状态机：** 用 XState v5 实现 CONTRACTS.md 2.1 的 10 个物理状态和它们之间的转换。
  - 输入：应用内总线事件（经 `stage.event` 转发，范围见 CONTRACTS.md 2.3）、舞台自己的交互事件（命中、拖拽）、音频通道上的 `stage.speech.*`、身体动作请求；
  - 输出：对动画运行时的调用（切换 L0 片段、触发 L1 手势、设置视线目标、设置帧率档位）；
  - 各状态使用的片段一律按 CONTRACTS.md 1.8 的槽位 id 取。
- **转换规则（至少）：**
  - `voice.vad speechStart` → listening；第一句开始播放（`stage.speech.started`）→ speaking；最后一句播完 → idle；
  - 拖拽开始 → dragged（`dangle`）；松手 → falling（`fall_loop`，落地播放 `land`）或 sitting（`sit_edge_loop`），由 P6-09 的落点判定决定；
  - `sleep`、`wake`、`peek`、`goTo` 身体动作 → sleeping、idle（经 `wake_up`）、peeking、walking；
  - `desktop.dnd` 为 fullscreen 或 presentation → hidden（暂停渲染）；`meeting`、`busy` 不隐藏。
  - 主动性不在这里：状态机自己不决定小睡或探头，只执行核心的身体动作请求。
- **反射（固定参数，带版本）：**
  - `voice.vad speechEnd` 后 300 ms 内看向用户并轻点头；
  - 被拍（`pat`）时耳朵抖一下；被快速甩出时播放 `react_surprised`；
  - 视线在空闲时跟随光标，说话时看向用户。
- **身体动作执行器：** 执行 `stage.action`，校验动作类型与槽位的「可请求」标记，完成或失败后发出 `stage.actionResult`。
- **表演提示调度：** 收到 `stage.speech.begin` 时缓存这句的 `cues`，在 `stage.speech.started` 那一刻触发；`act` 只接受「可表演」的槽位；`look` 设定视线目标并保持到下一个 `look` 或该句结束。
- 用 XState 可视化导出状态图，放进报告。

### 验收
- 基于模型的测试：遍历所有状态对，不能出现未定义的转换；非法事件被忽略并记日志。
- 集成测试：用录好的事件序列回放一次完整对话和一次「睡着—被叫醒」，状态序列与预期一致。
- 非语言回应延迟：从 `voice.vad speechEnd` 到头部开始转向 ≤ 300 ms（完成标准第 8 项的一部分）。
- 身体动作：不可请求的槽位被拒绝并返回 `failed`。

---

## P3-05 动作库 v1 选定

- **建议执行者：** Codex 收集候选，你挑选
- **难度：** 低
- **里程碑：** N3 诞生
- **依赖：** P3-02、P1-03
- **工期：** 5 天（你约 3 小时）
- **分支：** `task/P3-05-motion-library`
- **目录：** `tools/motion-library/`、`content/_candidates/motions/`

### 步骤
1. Codex 按 CONTRACTS.md 1.8 的 30 个槽位，每个槽位准备 2–3 个候选，来源按优先级：
   1. Quaternius（CC0）；
   2. 你购买的 BOOTH 动作包（只进私有内容仓库）；
   3. HY-Motion 1.0 文生动作——**许可确认前只能用于原型**，manifest 中标记 `license: "custom:HY-Motion-unverified"`、`redistributable: false`；
   4. 你自己表演 + 视频动捕（可选，工具费用由你决定；确认工具对输出动作的许可）。
2. 全部转换为 VRMA，在评审台中按槽位分组展示（用 Nemu 的中期或最终模型播放）。
3. 你为每个槽位选 1 个，或标记「都不行」，由 Codex 再找一轮。

### 验收
- 30 个槽位全部有选定的动作，`manifest.json` 许可字段完整。
- 在 Nemu 模型上，30 个动作在 3 个机位下没有明显穿模（完成标准第 4 项），截图放进报告。
- [ ] 你签字确认动作库 v1。

---

## P3-06 身体读出 v2（E5 修订）

- **建议执行者：** Antigravity
- **难度：** 中
- **里程碑：** N3 诞生
- **依赖：** P5-22
- **工期：** 3 天
- **分支：** `task/P3-06-readout-v2`
- **目录：** `packages/body/src/readout/`、`packages/avatar-runtime/src/animation/`、`apps/stage/src/review/`
- **必读：** v2.0 第 9.3 节；CONTRACTS.md 第 7 节（按 P5-22 修订）；ARCHITECTURE.md 6.3；P3-01、P3-03 的报告

### 交付物
- 读出应用器改读 BodyReadout v2。三项经固定、带版本的映射（参数在 `profile` 里，可在评审台调整）驱动动画参数：
  - `facing` → 视线与身体朝向你的倾向（原 `gazeSociability` 的用处）；
  - `tension` → 姿态偏移、肩部、耳朵与尾巴（原 `posture`、`earPose`、`tailAmplitude` 的用处）；
  - `energy` → 呼吸、重心摆动与待机小动作的节奏和幅度（原 `idleTempo` 的用处）。
- 去掉表情基线：L3 只接收表演提示 `emo` 与口型；说话之外是中性表情加眨眼。
- 只应用，不保存、不累积；m = 0 或没有读出时用中性值（facing 0.5、tension 0、energy 0.5）。
- 删除对 v1 字段的读取。
- 评审台的身体读出预览改为三个滑块（P5-17 用它看读出参数的效果）。

### 验收
- 单元测试：映射的边界值；中性读出得到中性姿态；应用的幂等性。
- 结构检查：终端代码里没有以情绪、心情、好感命名的持久变量；L3 没有表情基线的输入。
- 评审台录两段 3 分钟待机视频（两组不同的读出），交给你看（不阻塞）。
