# P2-05 正式尾巴入口挂接（用户批准，单列）

2026-10-23 用户明确批准最小 build_tail.py 挂接，要求不改变 P2-03 其他行为、不影响 P2-06。本变更只提取已验证的候选材质处理，没有用于消除或掩盖弹簧跳变。

- build_tail.py 仅增加一个 helper import 和一次调用。
- tail/material.py 复用原候选处理：只对新生成的尾巴材质去掉不匹配的源图集、关闭依赖该图集的白色发光，并保留平滑法线。源材质不修改。
- 显式配置 shade_color_factor 时沿用该值；未配置尾巴材质覆盖时保留复制材质原有 shade 值。后者是原 P2-03 样例允许的配置，不能因提取候选逻辑引入 KeyError。
- tail/build_candidate.py 改为兼容入口，调用同一个正式实现；tail/__init__.py 登记 Python 包。
- 未修改 pipeline.py、springs.py、expressions.py、materials.py、schema、导出器或 P2-06 配置，未新增参数/依赖。B 的 K/D/G 和碰撞配置保持不变。

## 验证

1. 原候选构建、正式入口初次构建、最终兼容处理后的正式构建，三份 VRM 和 profile 均逐字节一致，见 formal-hook-equivalence.json。
2. VRM SHA-256 均为 42ee5201dd8af34b6fbfbb864d14763db681dd64fcca58c035a0140976361c79；profile 均为 59421f54644d91c536662b9c19313fb90e3619463e6e7d325b39c13e5f0f0ac9。
3. 标准 pnpm asset:build 的 B 产物 A1.4/A1.5 错误为 0；全量 asset-lint 仍如实因 4 个 A1.3 错误退出 1，留给 P2-06，没有跳过规则。
4. 原 build.example.yaml 也从正式入口完整运行，原样例验收 inspect 通过；仍只有原先 3 个角色特有错误（自定义表情、左右猫耳），其余错误 0。见 formal-hook-sample.json，未修改样例验收器。
5. Python ruff、mypy strict 和 31 项管线测试通过；最终检查日志见 formal-hook-final-*。

正式产物仍在 D:/Dev/cache/P2-05/，没有把未通过抖动验收的 B 移入正式内容目录。原提案补丁已应用，提案文本可在 912c165 的历史中查阅；当前以实际源代码差异为准。
