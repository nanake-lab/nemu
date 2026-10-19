# P2-04 资产契约校验器 — 报告

## 结果

完成资产契约 CLI、VRM 1.0 结构检查、profile 与 manifest 校验，以及 25 个自动化用例。已提供的 VRM 作为当前源导出进行只读检查；它仍有 6 条错误和 5 条警告，属于管线前模型状态，不代表 P2-04 测试失败。

## 交付物

- `tools/asset-lint/`：CLI、GLB 读取器、合同规则、测试夹具与测试。
- 根命令 `pnpm asset:lint <file.vrm> [--profile ...] [--manifest ...] [--json]`：`--json` 的标准输出可直接解析。
- `reports/P2-04/source-vrm-lint.json`：对原始 Nemu VRM 的 JSON 诊断和测量数据。
- `reports/P2-04/*-log`：测试与静态检查日志。

## 验收

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `pnpm install --frozen-lockfile` | 通过 | 锁文件解析和安装成功 |
| `pnpm --filter @nemu/asset-lint test` | 通过：25/25 | `asset-lint-tests.log`；包括合法模型、规则违规模型、语义骨名解析、无关弹簧链以及 CLI JSON 解析 |
| `pnpm lint` | 通过 | `pnpm-lint.log` |
| `pnpm typecheck` | 通过 | `pnpm-typecheck.log` |
| `pnpm test` | 通过：21 个测试文件、403 项 | `pnpm-test.log` |
| `pnpm asset:lint content/nemu/source/nemu-vroid-export.vrm --json` | JSON 可解析；6 错误、5 警告；命令按模型诊断返回退出码 1 | `source-vrm-lint.json` |
| 原文件 SHA-256 | 与初始检查一致：`5484C94F54F6C000767FEBACBC99FD7EA23AC074D9CA8EECEBAF65B967DE39AB` | 本机 `Get-FileHash`；源文件未改动 |

原始模型测量值：VRM 1.0 / glTF 2.0，VRoid Studio 2.14.0 导出；268 节点、3 网格、36,983 三角面、28 张纹理、18 个材质、215 个蒙皮关节、最大面部形态键数 57，静态 accessor bounds 高度约 1.584 m，纹理 RGBA8+mipmap 估算 205,871,445 字节。没有 profile，因此无法执行 profile 身高比较和按 profile 命名的耳朵/尾巴弹簧链核验。

当前源导出的错误为：`happy`、`relaxed`、`surprised` 的 `overrideBlink` 不符合合同；缺少 8 个 Nemu 自定义表情；缺少指定身体碰撞体；已识别的 `Hair` 弹簧链所引用碰撞体组未覆盖全部必需碰撞体。警告为纹理数 28（目标 ≤16）、材质数 18（目标 ≤12）、缺 profile 时无法识别耳朵与尾巴链，以及未找到模型的 manifest 条目。三角面、单图尺寸、贴图显存估算、required humanoid bones、MToon/unlit、VRM 元数据和脚底 y=0 检查未报错。

`validator.ts` 超过 400 行，因为它集中编排 CONTRACTS.md 1.1–1.6 的完整规则矩阵；GLB 解析、CLI、动态测试夹具和测试分别位于独立文件。后续若扩展规则，应把检查器按合同小节拆分。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| `@gltf-transform/core` | 4.5.1 | MIT | 解析规范化后的 glTF 2.0 结构和三角面数据；VRM 扩展由本工具独立读取 |
| `property-graph`（传递依赖） | 4.1.0 | MIT | `@gltf-transform/core` 的图数据结构依赖 |
| `@nemu/contracts` | `workspace:*` | 仓库内部包 | 使用现有 `NemuProfile` 与 manifest Zod schema |

## 待用户确认

- [ ] （不阻塞本卡）继续执行 P2-03 前，需提供可发现的 Blender 路径且版本符合项目要求；当前给出的 4.5.14 低于项目要求的 5.1 LTS 起点，且本机 `blender` 命令、`NEMU_BLENDER` 和卸载注册表均未发现该安装。
- [ ] （不阻塞本卡）P2-05/P2-06 需要 P2-02 的中期 VRM 导出；当前 `content/nemu/source/` 只有 VRM 文件，没有 `.vroid` 工程或进度截图。

## 后续建议（不在本任务范围内）

- 完成 P0-04 环境检查器及 P0-05 第二部分的内容仓库、子模块和 doctor 记录后，再开 P2-03。
- P2-03 的 Blender 版本锁定应遵从当前项目要求；不要用 4.5.14 生成正式管线结果。
- P2-05、P2-06 仍需 P2-02 中期导出、P2-03、用户的外观/手感选择和评审；P1-03 已合并。

## 下一步

等待本 PR 的 CI。满足仓库合并条件后合并本卡；P2-03 当前等待 P0-05 第二部分和兼容 Blender 环境。随后先处理 P0-04/P0-05 依赖，再按依赖和用户提供的 P2-02 资产继续。
