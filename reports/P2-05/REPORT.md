# P2-05 尾巴、猫耳与弹簧骨调校

## 当前状态

**部分完成，PR #48 保持 draft + needs-user。** 尾形 A 偏粗短已选；用户于 2026-10-23 明确撤回 B 作为最终弹簧参数，B 仅保留为历史对照。按最新要求在「沉稳」基础上新增 C、D 两组，每组连同沉稳、B 各跑一次相同快速拖动诊断，并录好四列同动作视频。暂无可以确认稳定通过的组；等待用户评审，运行时稳定性另交请求，不在本卡修复，不再追加参数试验。

工作树 D:/Dev/nemu-p205；新缓存 D:/Dev/cache/P2-05。已合并最新 origin/main 2333c23；继续工作前 fetch 无新提交，PR 无评论/审查。源 VRM 保持只读。全部验收之前不转为可审查、不合并、不取下一卡。

## 交付物

- content/_candidates/P2-05/：尾形 A（55.3 cm，最宽约 19 cm）、原有预设、新候选 springs/calm-heavy-{1,2}.yaml、four-review-parameters.json、来源清单与更新的选择记录。没有移入正式目录。
- apps/stage/test/jitter/：独立评审夹具；四组页 heavy-comparison.html?four=1，单组页默认沉稳参照，不再显示 B 已定稿。复用 P3-01 呼吸、重心、视线、眨眼；生产评审台和动画运行时未修改。
- four-review/video/calm-B-C-D_same-motion_24s_30fps.mp4：2560×1080、24 秒、30 fps、四列同动作原速，K/D/G 全程显示；白黑背景关键帧同目录。
- four-review/comparison.json：四组峰值、关节、参数、模型及逐帧 gzip 哈希索引；逐帧原始数据留在对应 D 盘报告路径，不删改、不重复上传大文件。
- tasks/requests/P2-05-spring-runtime-stability.md：交 P3-01 L4 / P3-04 的复现条件、参数、速度、关节、角度及保护建议。
- FORMAL-TAIL-HOOK.md：此前已获批准并完成的最小正式入口挂接，单列验证及对 P2-03/P2-06 的边界。

## 四组诊断（每组只跑一次）

| 组 | 头发链最大单步角度 | 关节 | 时间 | 观察 |
| --- | --- | --- | --- | --- |
| 沉稳 | 174.59949° | J_Sec_Hair2_06 | 22.21667 s | 接近 180° |
| B 历史对照 | 179.85087° | J_Sec_Hair1_07 | 26.06667 s | 接近 180°，与既有 B 一致 |
| 新 C 适度加重 | 122.95363° | J_Sec_Hair2_06 | 22.20000 s | 未出现同样近 180° 峰值，但仍有大幅单步变化，不能称稳定通过 |
| 新 D 加重快稳 | 176.75161° | J_Sec_Hair2_06 | 18.42500 s | 接近 180° |

新 C 的头发/猫耳/尾巴/裙摆 K/D/G 分别为 1.08/.80/.024、2.295/.82/.002、1.60/.82/.050、1.30/.82/.055；新 D 为 1.05/.84/.028、2.25/.87/.003、1.50/.87/.065、1.25/.87/.070。两组以沉稳为基础，略增重力与阻尼，刚度接近原值；无质量或人工延迟参数。B 文件哈希未变。

四组加载同一重建 VRM，只在评审内存中应用配置 K/D/G 和向下重力，保留拓扑、碰撞体与关节半径。新配置已通过 schema，但本轮未分别重导出两份新 VRM；用户选型后仍需重跑正式管线。完整参数及一次运行原始记录见 comparison.json 和各组 jitter-summary.json。

诊断为 **120 Hz，30 秒驱动＋5 秒回稳**：hips x=.24 sin(2π·1.3t) m、y=.12 sin(2π·2.1t) m；head roll=.18 sin(2π·1.7t)、spine roll=.10 sin(2π·1.3t) rad。相邻局部四元数最短旋转，处理 q/-q 等价。不是 OS 拖拽全路径，也不能直接等同可见屏幕位移，未新增角度门限。

视频为不同用途的 **24 秒日常动作比较**：4 秒呼吸、8 秒慢转身、3 秒拖动、9 秒回稳，物理仍 120 Hz，输出 30 fps。与旧版同一动作，各列一致；视频中诊断数字指上表 120 Hz 单步，而不是 30 fps 编码帧。不能因视频日常动作看不出差异或翻转而覆盖 30 秒诊断结果。

## 验收与检查

| 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 原始源模型 asset-lint 基线 | 原始结果保留 | baseline-original.log |
| A1.4/A1.5 | 既有正式 B 产物错误 0；未修改本轮模型或许可 | selected-B-product-audit.json |
| 全量 asset-lint | 4 个 A1.3 错误留给 P2-06；骨数警告 277，原始非零退出保留 | formal-B-final.log |
| 最小正式尾巴入口 | 已批准完成；B VRM/profile 与候选逐字节一致，P2-03 样例验收保持通过 | FORMAL-TAIL-HOOK.md、formal-hook-equivalence.json、formal-hook-sample.json |
| 四组快速拖动 | 每组一次，全部有限；存在上表大幅旋转变化，未签发稳定通过 | four-review/comparison.json、各组 jitter-summary.json |
| 四组视频 | 720 帧全部解码、四组所有关节 K/D/G 核对、诊断峰值显示、MP4 服务与浏览器播放通过 | four-review/verification.json、video/capture.json |
| TS 检查 | 本轮 lint、typecheck、29 文件 443 项单测通过 | four-review/{lint,typecheck,tests}.log |
| Python / 资产契约回归 | 此前挂接后 31 项 Python、ruff、mypy strict 24 文件、33 项资产校验通过；本轮未改这些实现 | formal-hook-final-*.log |

## 契约相关修改（ADR 0015，单列）

- export_vrm.py：验证 licenseUrl=https://vrm.dev/licenses/1.0/，otherLicenseUrl 等于非空 contentLicenseUrl，保留源权限。
- asset-lint A1.5：拒绝错误公共许可 URL、缺失/空白/非字符串内容许可；夹具、build.example.yaml、Python/TS 负向测试已补齐。
- expressions/materials 等其他卡小节仍为候选临时最小配置。A1.3 属 P2-06，不吞原始退出码。

## 正式入口挂接（单列）

用户此前明确批准：build_tail.py 只增加 import 与调用，原候选处理提取到 tail/material.py，兼容入口复用。只处理新尾巴材质，未修改 pipeline.py、springs.py、expressions.py、materials.py、schema 或运行时。三份 B VRM/profile 逐字节一致；原 P2-03 样例验收器 inspect 通过。此变更未用来修复或掩盖弹簧跳变。

## 证据边界与资源

原始 VRoid 未复现同样近 180° 跳变；原始模型只套 B K/D/G 约 170.66°，B 关闭碰撞仍约 179.85°，无零长头发骨。新尾巴/碰撞体不是必要条件，具体运行时根因未确认，见 SOURCE-CONTROL.md。旧失败原始数据和标签纠正说明均保留。

源 VRM SHA-256：d3f4799c37ecfdd1806ef93264b06f5d979b697a592d4422884046b594e1cb2b；共用重建 VRM：42ee5201dd8af34b6fbfbb864d14763db681dd64fcca58c035a0140976361c79；B 配置：db829f9231fb7b61d3d9a16645b7bc692271f5eb8c47d254f84dda0e49c16099。

本轮运行前读取 P4-01/P4-02 最新状态均空闲且测量已结束，未见 Runner.Listener、Runner.Worker、Blender、Electron 或活动测量进程；诊断、录制、浏览器核验串行。视频为真实 RTX 3080 渲染。GPU 工作已结束；未新增依赖、下载模型、联系其他执行者或修改动画运行时。

## 待用户确认

- [x] A 偏粗短尾形和最小正式入口挂接已批准。
- [ ] （阻塞）四类弹簧最终手感待选；B 仅历史对照。
- [ ] （阻塞）本轮四组均不能确认无跳变通过；运行时问题交 P3-01 L4 / P3-04 按请求处理，P2-05 不自行修复。
- [ ] 全部验收完成后再同步最新 main、重跑检查，才能转为可审查。
