# P2-06 自定义表情与材质精修 — 报告

## 结果
部分完成：候选与截图准备完成，等待用户选择。PR #59 保持草稿；未定稿、未运行最终 Blender 构建、未转为可审查、未合并。

## 交付物
- `content/_candidates/P2-06/expressions.candidates.yaml`：8 个自定义表情，每个 A/B/C = 0.55/0.80/1.00；12 种 emo，每种 A/B 两个组合；程序化面部部件参数。
- `content/_candidates/P2-06/materials.candidates.yaml`：soft / standard / contrast 三组 MToon 候选，阴影色、边界、轮廓光、描边均显式配置。
- 同目录 `expressions.build-fragment.yaml`、`materials.*.yaml`：后续选定参数的管线格式片段，尚未写入正式 build.yaml。
- 同目录 `models/`、`manifest.json`、`LICENSE.md`：三个未定稿预览模型、逐模型 profile、来源与许可；不授予分发许可。
- `review.html`：可选 A/B/C 与生成选择文本的本地评审页，仅显示选择，不自动保存或批准。12 种 emo 各与设定集对应格并排。
- `visual/`：62 张单图与 URL、确定性证据；`sheets/`：8 张自定义表情对照、12 张 emo 对照、3 张背景材质对照，共 23 页。
- `tools/asset-pipeline/expressions/`：候选组装、有限数/数据保全核验、截图及对照页生成工具；不是正式管线的替代品。

## 范围与保护
从 origin/main `91b31ed` 新建独立任务工作树与分支，草稿 PR 先于实现创建。主工作区的未提交内容和源 VRM 不动。内容仓使用独立 P2-06 分支，以 P2-05 已推送的 `03742ab` 为候选底版。

只读复用 `_candidates/P2-05/models/short-thick-standard/`；模型 SHA256 为 `25aea72bbae2d08048f591c5fc091f2c5c7ac693e67ee4264b5446823ad5dbf4`。没有修改 P2-05 的 tail、springs、候选文件、选择或工具实现。原始模型二进制前缀、蒙皮、弹簧骨、元数据与使用权限逐项保持；面部基础顶点不变，仅追加一个猫嘴 morph；其余原有网格不变。源文件哈希不变。没有正式内容定稿。

当前八个表情：nemuSmug、nemuCatMouth、nemuSparkle、nemuBlush、nemuTeary、nemuSleepy、nemuPout、nemuAngryMark。猫嘴采用面部形态键；星星、脸红、泪光、井字采用本卡脚本生成并绑定到头部的几何。脸红使用顶点透明度柔化，其余符号采用 opaque unlit，避免脸部透明材质排序盖住符号。所有外观数值来自候选 YAML。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| asset-lint：soft、standard、contrast，包含对应 profile 与 manifest | 每个 0 错误、2 警告 | `lint-*-final.json` |
| ADR 0015 许可字段与权限保全 | VRM 公共 licenseUrl、otherLicenseUrl=LICENSE.md、原权限均保持 | `structural-checks.json` |
| 原模型字节、原蒙皮、弹簧骨、形态键计数、追加 accessor 有界且有限 | 3 模型通过 | `structural-checks.json` |
| 连续两次候选组装 | 所有候选文件 SHA256 相同 | `reproducibility.json` |
| 静态预览双截图逐字节一致 | 62/62，通过；白、黑、彩色渐变背景 | `visual/screenshots.json` |
| Python pytest，既有资产管线测试 | 21 passed | `pytest.log` |
| mypy --strict / ruff check，新 Python 工具 | 通过 | `mypy-final.log`、`ruff-final.log` |
| pnpm lint / pnpm typecheck / pnpm test | 通过，29 文件、443 测试 | `pnpm-*.log` |
| 最终正式 Blender 全管线与阶段验收 | 未运行，待用户选择及 P2-05 集成 | `BLOCKED.md` |
| 用户对 12 种最终组合逐项签字 | 待完成 | 本表不代签 |

两个 lint 警告均原样保留：13 个材质超过 12 的建议值（本卡增加 4 个符号材质），源模型 277 个蒙皮关节超过 250 的建议值。E 级规则均为零；没有降低阈值或排除错误。

本机依赖以已有安装的工作区目录链接复用。pnpm 的脚本前自动安装因链接目录而产生警告，随后的 eslint、tsc 和 Vitest 实际执行并通过。Python 资产运行环境缺少 mypy/ruff 的原始失败日志已保留；通过已有 uv 离线缓存准备固定版本检查环境后重跑通过。没有联网下载或新增应用依赖。

## 视觉 QA 与边界
保留首轮被拒绝的猫嘴浮贴、实心脸红示例于 `qa/`。已改为真实嘴部 morph 和柔边脸红；修正星星眼的透明排序，并把 excited 的两组候选改为睁眼组合，避免被 happy 的眯眼遮住。原始尝试日志保留。

截图使用独立的 three-vrm 候选对照夹具、固定相机与光照、SwiftShader 软件渲染、零时间推进。它不是正式舞台动态或真实 GPU 效果验收；没有更新任何视觉基准。嘴型、眨眼和新增部件随面部动作的贴合、正式 Blender 往返保真仍待最终集成检查。当前 emo 组合使用完整 custom 定义（C）；用户若选择单项 A/B，须重建组合并重新签字。

## 新增依赖
无新增运行时依赖。复用项目 Python/PyYAML/jsonschema、three 0.174.0、three-vrm 3.5.5、Playwright 1.64.0 与固定检查工具；许可证分别为 Python PSF、MIT、MIT、MIT、MIT、Apache-2.0。过程生成的符号没有外部素材来源，内容许可仍为私有待 P7 决定。

## 待用户确认
- [ ] （阻塞）8 个自定义表情各选 A/B/C 或提出修改。
- [ ] （阻塞）12 种 emo 各选 A/B 或提出修改；最终组合重建后逐项签字。
- [ ] （阻塞）选择 soft / standard / contrast 材质或提出修改。
- [ ] （阻塞）选定后，批准将所选参数写入正式 build.yaml；本次仅提供候选。

## 后续建议（不在本任务范围内）
源模型骨骼建议值警告保留，由后续资产优化任务评估；不在本卡删骨骼。

## 下一步
候选可与 P2-05 一起挑选。等待选择期间保持草稿；用户选定后仅替换 expressions、materials 与本卡 emotionMap 参数，结合 P2-05 的最终 tail/springs 运行正式管线。全部验收完成后 fetch/merge 最新 origin/main、重跑检查、等待 CI 全绿，转为可审查并由用户合并。中难度按本次用户指示不另请审查。
