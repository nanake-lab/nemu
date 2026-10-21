# P2-03 Blender 无头资产管线 — 报告

## 结果
实现和本地样例验收已完成，PR #43 保持未合并，待 Claude 审查与最终 CI。最终代码两次完整复跑与全部本地检查已通过任务卡的样例判据。

## 同步与认领
- 开工执行 `git fetch origin`，从 `origin/main` 的 `ab1acb5` 新建 `task/P2-03-asset-pipeline`，没有依赖或检出本地 main。
- 完整读取 AGENTS、ARCHITECTURE、CONTRACTS、Echo Core README、P2-03 卡和 blender-headless 技能。
- 认领提交 `98dc77b`，草稿 PR：<https://github.com/nanake-lab/nemu/pull/43>，标题为 `feat(P2-03): Blender 无头资产管线`。
- 已查询在做的 PR，无与本卡资产管线目录重叠的卡。再次 fetch 后没有需要合并的 main 新提交；未 rebase、强推或合并。

## 交付物
- `tools/asset-pipeline/run.ps1` / `launch.mjs` / `pipeline.py`：完整命令和 `asset:step -- <step> in.blend out.blend`；独立进程、排他锁、资源检查、逐步骤日志、中间 blend、最后调用未修改的 asset-lint。
- 九个独立 Blender 脚本：导入、源校验、尾巴网格/UV/骨链/连续权重、弹簧与碰撞组、三种表情绑定与覆盖规则、按名称应用 MToon、贴图缩放/打包/清理、VRM 1.0 导出和 v2 profile。
- `blender-version.txt` 保留既有 `5.2.2`；相邻 `vrm-addon-version.txt` 新登记 `4.10.2`。每个 Blender 进程执行动作前都核对两者。
- `build.schema.json`、带注释 `build.example.yaml`、README 与 `pyproject.toml`。缺字段、重复 YAML 键、非有限数和不符合契约的 profile 都报错；profile 验证直接使用 `@nemu/contracts`，没有复制另一份跨进程类型。
- `tools/texture-fix/`：透明边缘 RGB 扩展、显式色板量化、邻域孤立色点清理；保留 alpha，只写新文件，拒绝覆盖和向只读内容目录写入。
- `docs/SETUP.md` 只追加「VRoid 导出设置」。根 package.json 只替换 asset:build 占位并新增 asset:step，其余条目不变（`scope-check.txt`）。
- `tests/fixtures/manifest.json` 登记现有官方样例的来源、许可和 SHA-256；不复制或提交新模型。
- `tests/test_logic.py`：21 项无 Blender pytest；`tests/verify_sample.py` 单独生成样例验收和 JSON diff 证据，正式构建没有忽略规则开关。

## 验收
| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| 实际版本/RNA/扩展属性检查 | Blender 5.2.2 LTS、扩展 4.10.2，与锁一致 | `api-initial.log`、`api-geometry.log`、`api-properties-complete.log`、`api-human-bones.log` |
| 错误版本的负向检查 | Blender 与扩展锁分别不匹配时均明确拒绝；探针 exit 0 | `probe_versions_negative.py`、`version-negative.log` |
| 连续完整构建（review-1、review-2） | 每次九个 Blender 步骤 exit 0；第十步 lint 原始 exit 1（3E/3W，详见下表） | `review-1.log`、`review-2.log` 和同名目录中的逐步日志 |
| 样例剩余错误与绑定/骨链检查 | 排除角色专属错误后 0；三种绑定存在，实际 8 节尾骨与 profile 对应 | `review-verification/sample-acceptance.json` |
| 幂等 glTF JSON diff | 完全相同，0 行 diff；没有剔除任何字段或时间戳 | `review-verification/gltf-json.diff` |
| 完整 VRM 的 SHA-256 | 两次均为 `7a6eeaf68832c6c4ee7adc100f0bd9dabcb17230f975481fc07336b14ace8784` | `review-verification/sample-acceptance.json` |
| `asset:step` springs、expressions | exit 0；输入 blend 与输出 blend 分离 | `asset-step-springs.log`、`asset-step-expressions.log` |
| pytest | 21 passed | `pytest-review.log` |
| mypy --strict | 18 个 Python 文件无错误 | `mypy-review.log` |
| ruff | 全部通过 | `ruff-review.log` |
| `pnpm lint` / `pnpm typecheck` | exit 0 | `pnpm-lint.log`、`pnpm-typecheck.log` |
| `pnpm test` | 29 个文件、443 项通过 | `pnpm-test.log` |
| 贴图 CLI | 扩展 RGB、保留 alpha、原件哈希不变；覆盖输入被拒绝；最终 Pillow 12.3.0 API 无弃用提示 | `texture-cli.log`、`pillow-api.log`、`texture-cli-final.log` |
| 只读输入保护 | 21/21 文件字节数和 SHA-256 相同，content 子模块与模板 git diff 为空 | `readonly-baseline-complete.csv`、`readonly-after.csv`、`readonly-final-diff.txt` |
| SETUP 与根脚本范围 | 原文是新增文档的精确前缀（规范化 CRLF）；其他 package 条目相同 | `scope-check.txt` |

Python 校验使用已有 `workers/irl/.venv/Scripts/python.exe` 中 pytest 9.1.1、mypy 1.20.2、ruff 0.15.6，并以 `PYTHONPATH=tools/asset-pipeline/.venv/Lib/site-packages` 使用本卡独立配置依赖。没有修改 worker 环境或源码。纯测试会调用 Node 契约验证器，不加载 bpy。Blender 动态 RNA 边界没有本机类型桩，采用实机探针和集成运行验证；PyYAML 的两个定点类型忽略只覆盖外部无桩加载器。

### asset-lint 排除明细
原始诊断完整保留，没有改校验器、级别或阈值，正式 `asset:build` 遇到这些错误仍返回 1。以下排除仅用于任务卡允许的官方样例验收：

| 规则 | 原始数量/级别 | 原因与后续归属 |
| --- | --- | --- |
| `A1.3-custom-expressions` | 1 E | 样例没有 Nemu 的 8 种专属美术形态，不猜测或伪造。使用 `pipelineProbe` 验证 morph/material-color/texture-transform 三类真实绑定；具体 Nemu 表情由 P2-06 选择并验收 |
| `A1.4-cat-ear-spring-bones` 左耳 | 1 E | 官方样例没有猫耳骨链；真实角色由 P2-05 验收 |
| `A1.4-cat-ear-spring-bones` 右耳 | 1 E | 同上，右耳独立诊断 |

其余错误为 0。以下警告没有忽略或改级别：18 张材质纹理超过目标 16（`A1.1-texture-count`）；14 个材质超过目标 12（`A1.1-material-count`）；样例没有可识别的裙摆链（`A1.4-skirt-spring-bones`）。不为消除警告做审美合并。头发链满足识别要求，没有排除头发规则。尾巴≥8 节、全部必需身体碰撞体及相关链引用均未报错。

实测：37,506 三角面、175 个 skin joints、最大 57 个形态键，贴图预算估算 119,188,139 bytes；实测身高 1.615418584165127 m。profile 身高/头顶偏移来自导出 glTF 节点变换后的 POSITION accessor bounds，耳/尾名从场景骨骼解析，其余字段由 YAML 读取。

### 来源与许可
现有 `apps/stage/public/sample.vrm` 与 spikes 的 `VRM1_Constraint_Twist_Sample.vrm` 哈希相同：`12c2b97e95e700783a6a550dc0eee2d7880aeedccef9ae67bc4c5a2f0f2631a2`。内嵌 meta：作者 pixiv Inc.，版本 v1.0.1，版权 `(c) 2022 pixiv Inc.`，`licenseUrl=https://vrm.dev/licenses/1.0/`、`allowRedistribution=true`、`modification=allowModificationRedistribution`。来源登记依据仓库既有 P0-03 记录与内嵌元数据，本卡未联网重新下载或核对远端文件。生成 manifest 保留来源与许可；测试产物未作为 Nemu 定稿入 content。

### 失败与调度证据
- 保留了联调时脚本导入路径、扩展版版本读取、骨骼枚举键、Curve/Mesh 类型以及 collider 枚举大小写错误的原始日志。修复依据本机 RNA 和已安装扩展源码；最后完整集成运行通过上述步骤。
- `api-properties*.log` 含早期失败；最终所需属性已在后续探针确认。不存在把探针失败写成通过的情况。
- 子代理只做只读哈希/元数据与依赖路径查询；按工具提供的 `routine_reporter` 角色执行，读取项目职责配置，没有运行测试、安装依赖、改代码、提交或操作 PR。本记录不声称客户端会自动加载任意项目 TOML。主代理交叉检查时发现其初始清单遗漏 `content-template/nemu/lipsync/.gitkeep`，用 origin/main 的原始 git blob 补齐后重比，21/21 一致；初始清单差异保留在 `readonly-diff.txt`。
- Blender、根测试和检查由主代理串行安排。runner 起初离线，随后其他卡的 Windows CI 开始；`accepted-1.log`、`accepted-2.log` 记录调度守卫检测 `Runner.Worker` 后拒绝启动 Blender，属于资源拒绝，不是管线回归。
- 所有 blend 和 VRM 测试产物保留在工具 `.cache/`，不提交；逐步 stdout/stderr 与 CLI 结构化结果由外部编排器保存。本卡不涉及 GPU 测量或舞台视觉验收。

## 新增依赖
仅开发工具依赖，应用运行时依赖不变；安装仅使用已有本机缓存和已安装环境。

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| PyYAML | 6.0.3 | MIT | 解析 YAML |
| jsonschema | 4.26.0 | MIT | 配置 JSON Schema 检查 |
| Pillow | 12.3.0 | MIT-CMU | 独立贴图 CLI，使用桌面工具已有 Python 包 |
| pytest / mypy / ruff | 9.1.1 / 1.20.2 / 0.15.6 | MIT | 已有环境中的开发校验工具；本卡 pyproject 登记固定版本 |
| Blender | 5.2.2 LTS | GPL-3.0-or-later | 已安装的外部开发工具，未引入应用代码或打包 |
| VRM Add-on | 4.10.2 | MIT OR GPL-3.0-or-later | 已安装扩展，版本及许可来自本机 blender_manifest.toml |

本卡独立环境实际从缓存安装的间接包：attrs 26.1.0（MIT）、jsonschema-specifications 2025.9.1（MIT）、referencing 0.37.0（MIT）、rpds-py 2026.9.1（MIT）、typing-extensions 4.16.0（PSF-2.0）。离线尝试通用 uv lock 时缺少其他平台的 rpds-py 分发缓存，未下载或制造 lock；本机依赖安装成功，确切环境由 `pyproject.toml` 和报告版本登记。没有改 CI 四项目循环，本卡无 Blender pytest 在本机执行。

## 待用户确认
- [ ] （不阻塞）真实 Nemu 的尾巴、猫耳、表情、MToon 与物理参数仍分别由 P2-05/P2-06 选择；样例参数仅用于机制测试，不作为审美定稿。

## 后续建议（不在本任务范围内）
- `content-template/nemu/build.yaml` 当前是 `{}`，缺少新 schema 的 version、source、provenance、sourceChecks、meta、tail、springs、expressions、materials、textures、profile 全部必需小节。按用户要求未改它，也未改 content 的 build.yaml。
- CONTRACTS 1.5 写「licenseUrl 指向内容包许可文件」，但已安装的 VRM 1.0 扩展固定导出公共许可 URL，附加许可使用 otherLicenseUrl。本卡没有改契约或静默放宽：导出后若 licenseUrl 与 YAML 不同会报出期望/实际 URL。`license-negative.log` 证明不同 URL 被拒绝；本卡样例的 URL 一致。正式内容若使用其他许可地址，需后续单独澄清字段语义后才能导出，不能当作已经支持。
- 当前导入入口明确校验 VRM 1.0；VRM 0.x 的显式迁移需单独补充，不会静默改写来源许可或源文件。
- JSON Schema 735 行、带注释样例 412 行是声明数据，保留展开的字段/参数以便字段级错误定位；Python/JS/PS 实现文件均小于 400 行。
- 可在后续环境具备通用依赖缓存时，给独立 Python 工具生成跨平台 uv.lock 并在 ci.yml 追加单独检查步骤；本卡未动原有四项目循环。

## 下一步
等待 Claude 审查 PR #43 及最终 CI；不合并，不取下一张卡。若最终 Windows 检查排队，等待现有 runner 空闲；不自行停止 runner 或其他卡作业。
