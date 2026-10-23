# P2-05 尾巴、猫耳与弹簧骨调校

## 当前状态

**部分完成。** 用户已选 A 偏粗短尾形及 B 加重·留惯性弹簧，不再等待选型。B 已重新跑完完整候选管线，A1.4/A1.5 错误为 0；拖动检查仍出现局部关节旋转跳变，不能签发无高频抖动通过。正式入口挂接已获用户批准并完成，正式 VRM/profile 与候选逐字节一致。当前只剩抖动验收未闭合，见 [BLOCKED.md](BLOCKED.md)。PR #48 保持 draft + needs-user，不合并、不取下一卡。

任务工作树仍在 D:/Dev/nemu-p205；本轮新构建与浏览器临时缓存使用 D:/Dev/cache/。已合并 origin/main 的 2333c23（PR #66），按最新 AGENTS 执行；PR 最新提交、评论和审查状态已检查，当时无评论/审查。历史过程以 git 为准，原始失败证据保留。

## 交付物

- content/_candidates/P2-05/：三尾形、三初始弹簧预设、两组加重候选、来源清单与用户选择。尾形 A：55.3 cm，最宽约 19 cm；弹簧选 B。未移入正式目录。
- apps/stage/test/jitter/：独立评审夹具，默认 B；接入既有 P3-01 呼吸、重心、视线与眨眼，可加缓慢转身；生产评审台和动画运行时未修改。
- heavy-comparison/A-B-C_same-motion_24s_30fps.mp4：三列同动作原速、各部位参数全程显示；用户已选 B 并说明未明显看出区别，不推断为已感知明显改善。
- D:/Dev/cache/P2-05/selected-B-build/：最新 B 的实际构建产物及所有阶段日志。
- selected-B/：120 Hz、30 秒拖动＋5 秒回稳的原始有符号角速度、摘要、截图和频率诊断。
- FORMAL-TAIL-HOOK.md：已批准的最小正式入口挂接，单列验证及对 P2-03/P2-06 的边界。
- SOURCE-CONTROL.md、source-B-comparison.json：原始 VRoid 对照和一轮有边界的定位。

## 验收

| 检查 | 当前结果 | 证据 |
| --- | --- | --- |
| 原始源模型 asset-lint 基线 | 原始结果保留 | baseline-original.log |
| 尾形与弹簧选择 | A 尾形、B 弹簧已选 | 候选 user-selection.json；内容提交 2619df1a4211f00c00455fdffa493399e048f35f |
| B 完整正式管线 | 标准 asset:build 所有生成阶段完成，VRM/profile 与候选逐字节一致；全量 lint 因 A1.3 返回 1，未绕过 | formal-B-final.log、formal-hook-equivalence.json |
| A1.4/A1.5 | **错误 0**；源权限、表情 override 与自定义表情名称未改变 | selected-B-product-audit.json |
| 全量 asset-lint | 4 个 A1.3 错误留给 P2-06；1 个骨数警告（277） | 同上及构建 lint.json |
| 30 秒拖动＋5 秒回稳 | **未通过无高频抖动验收**：有限、可回稳，但局部旋转跳变未解释 | selected-B/jitter-summary.json、jitter-B.json.gz |
| 新诊断脚本 | 对真实重建模型核对 B 的 K/D/G；保留有符号角速度，非仅有限值判断 | apps/stage/test/jitter/check-selected-springs.mjs |
| 最新 main 同步及正式挂接后 TS 检查 | lint、typecheck、29 文件 443 项单测通过 | formal-hook-final-{lint,typecheck,tests}.log |
| asset-lint 负向回归 | **33/33 通过** | formal-hook-final-asset-lint-tests.log |
| Python 管线回归 | 本轮 31/31、ruff、mypy strict 24 文件通过 | formal-hook-final-{pytest,ruff,mypy}.log |
| 正式入口挂接 | 已批准并完成；标准 B 导出与候选逐字节一致，原 P2-03 样例验收保持通过 | FORMAL-TAIL-HOOK.md、formal-hook-sample.json |

## 契约相关修改（ADR 0015，单列）

- export_vrm.py：验证 licenseUrl 等于 https://vrm.dev/licenses/1.0/，otherLicenseUrl 等于 contentLicenseUrl 且非空，保留源权限。
- asset-lint A1.5：拒绝错误的公共许可 URL，以及缺失、空白或非字符串的内容许可字段。
- 相关夹具、build.example.yaml 和 Python/TS 负向测试已补齐；本轮 33 项 asset-lint 测试包含这些反例。
- 最新 B 的公共许可和内容许可分别为 VRM 1.0 公共 URL 与 LICENSE.md；源使用权限无差异。完整管线中的 expressions/materials 等其他卡小节仍为候选临时最小配置；A1.3 留给 P2-06，没有吞掉原始退出码。

## 当前抖动证据及边界

固定输入为 hips x=0.24·sin(2π1.3t)、y=0.12·sin(2π2.1t)，头 roll=0.18·sin(2π1.7t)，spine roll=0.10·sin(2π1.3t)；30 秒归零，再回稳 5 秒。原弹簧骨中心、碰撞体和 B 参数均保留，未伪装为 OS 拖拽全路径。

头发峰值 **376.679 rad/s**，约 179.85°/帧，位置 J_Sec_Hair1_07、t=26.0667 秒；尾巴峰值 **94.403 rad/s**，NemuTail10、t=27.3833 秒。猫耳/裙摆峰值分别约 21.905/36.170 rad/s。停止后回稳，但不能以有限或回稳替代无高频抖动要求。

频率诊断取各区域峰值前后约 2 秒，有符号局部角速度三分量逐项去均值、Hann 加窗、DFT 后合计功率；>15 Hz 比例分别约 89.21%（头发）、41.63%（尾巴）、0.061%（耳）、0.074%（裙摆）。这不是新验收门限，孤立跳变也会带来宽带能量，局部旋转不能直接等同屏幕位移；根因待定位。本轮未继续调参绕过。按用户条件另做原始源模型对照，再进行两项有边界诊断；结果见 SOURCE-CONTROL.md。原始模型没有近 180° 跳变，但只套用 B K/D/G 即出现约 170.66°；B 关闭碰撞仍为约 179.85°。未发现零长骨，轴向/重力接近平行且峰值时处于反折。问题已收窄到 B 参数组合与既有求解响应，尚不能确认具体单参数或求解步骤；按用户第 4 条停止本轮，未修改 B、源模型、运行时或验收要求。

源 VRM SHA-256：d3f4799c37ecfdd1806ef93264b06f5d979b697a592d4422884046b594e1cb2b，仍只读；B 产物 SHA-256：42ee5201dd8af34b6fbfbb864d14763db681dd64fcca58c035a0140976361c79。压缩前原始诊断 SHA-256 在摘要中；原始失败数据不删改。

## 资源与依赖

P4-02 最新会话确认测量已结束，P4-01 空闲；本轮前未见 Runner.Listener、Runner.Worker、Blender 或活动测量进程。构建、浏览器诊断与回归串行，使用既有 Blender 5.2.2、VRM Add-on 4.10.2、three-vrm、Playwright 与 Python 环境，无新增运行时依赖。未操作其他应用请求审查或联系执行者。

## 正式入口挂接（单列）

用户已明确批准：build_tail.py 只增加一个 import 和一次调用，候选已有材质处理提取到 tail/material.py；兼容入口复用同一函数。没有修改 pipeline.py、springs.py、expressions.py、materials.py 或 schema。正式 B 的 VRM/profile 与原候选逐字节一致；原 P2-03 样例原验收器 inspect 通过，仍只有已知 3 个角色特有错误。具体文件、兼容配置处理和证据见 FORMAL-TAIL-HOOK.md。此材质改动没有用于修复或掩盖弹簧跳变。

## 待用户决定

- [x] A 尾形、B 弹簧选择，无需再选。
- [x] 最小正式入口挂接已批准并完成。
- [ ] （阻塞）本轮定位后仍无法确认和修复反折时的旋转连续性；保持 B，待用户选择运行时专项复核或暂保留草稿。
- [ ] （阻塞）拖动验收完成后，再同步最新 main、重跑检查；全部验收完成才转为可审查。
