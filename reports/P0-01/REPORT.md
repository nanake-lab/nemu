# P0-01 单仓脚手架与 CI — 报告

## 结果

实现已提交准备验证；本机检查已通过一轮，远程 CI 尚待运行。最终验收状态追加于本报告后续记录。

## 交付物

- pnpm、Cargo、四个 uv 工程和 .NET 10 骨架；精确工具/依赖版本和锁文件。
- Electron 双窗口、安全 preload、独立 Vite 舞台、React 19 面板、7 个公共包和 codex-bridge。
- content-template、空 profile/schema、Codex 版本锁、W00 模板和规格对照表。
- Ubuntu/Windows CI、临时存储烟测、命名管道/Unix socket 字节测试、Low IL 环境测试、初始视觉基准。
- docs/SETUP.md、自托管 runner 安装说明、CI 分钟估算、PR 模板。
- 受限 legacy Composer 迁移；见 migration.md。

## 验收

| 命令或检查 | 当前结果 | 证据 |
| --- | --- | --- |
| pnpm i/lint/typecheck/test/build | 通过；13 个单元测试 | validation.json 与各 .log |
| cargo fmt/clippy/test --workspace | 通过；crate 为空骨架，0 个业务测试 | rust-*.log |
| 四个 Python 工程 sync/ruff/mypy/pytest/build | 通过；各 1 个示例测试 | workers-*.log、services-*.log |
| .NET Release build | 通过 | dotnet-build.log |
| Electron 冒烟 | 已通过；后续稳定性复核在进行 | electron.log |
| SwiftShader 白黑背景像素比较 | 通过 | visual/comparison.json |
| 命名管道往返、Low IL 写拒绝 | 通过；仅环境烟测 | transport.log、windows-isolation.log |
| Ubuntu 与 Windows CI | 待首次运行 | 后续记录追加运行链接 |

## 规格对照

见 ../../docs/traceability/P0-01.md。Echo Core 技能用于限定依赖方向、
规格证据和临时存储边界；视觉技能用于 SwiftShader 固定渲染与白黑双背景证据。
未实现业务消息、状态或算子；不签发 F/J/SYS/I 验收结论。无新增 ADR 或差异登记。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| Electron | 44.7.0 | MIT | 桌面运行时（开发依赖安装） |
| React / react-dom | 19.3.0 | MIT | 面板 |

其余均为构建/测试依赖，精确版本见根 package.json 和各 pyproject.toml。
Rust/.NET 骨架没有第三方运行时依赖。未引入 Nx/Turborepo 或 GPL/AGPL 应用依赖。
工具链：Node 24.21.0、pnpm 12.11.2、Rust 1.99.0、Python 3.12.14、
uv 0.12.15、.NET 10.0.401、Codex CLI 0.162.1。
Vite 7.3.7 / TypeScript 6.0.3 按 electron-vite / typescript-eslint 的兼容范围锁定。

## 待用户确认

- 无架构/契约变更。占位 UI 和初始基准仅供脚手架验收，不代表角色美术定稿。
- 长期 Windows runner 注册仍由 P0-05 第二部分管理；本次仅使用验证后自动注销的 ephemeral runner。

## 后续建议（不在本任务范围内）

- P0-02 实现应用契约；P0-06 填写 W00 模板；P5 逐步实现核心。
- 当前所有未实现业务命令明确返回 2；dev 只启动窗口，不伪装完整进程拓扑。
- 后续模型和运行时依赖在各自任务登记；不预装 GPU 模型。

## CI 分钟估算

默认 60 次/月 × Ubuntu 12 分钟 = 720 分钟/月。以 2000 免费分钟示例计为 36%；
实际套餐未知，不声称读取账单。预算脚本可配置，超过 80% 输出 workflow warning。

## 下一步

等待远程 CI 结果及最终验收记录。本轮只完成 P0-01；不提前启动依赖它的任务。

## 恢复执行记录（2026-10-14）

- 用户恢复执行；暂停检查点保留为历史记录。
- 修复 Electron 冒烟测试在 launch 失败或 close 抛错时遗漏临时目录清理的问题；未降低断言或修改视觉基准。
- 修复后本机 lint、typecheck、Electron 冒烟和 SwiftShader 白黑背景视觉回归再次通过。
- 全量本机验证的 34 项命令及退出码见 `final-validation/validation.json`；本次只复跑受影响的检查。
- 初次 CI 的 Windows SDK 安装权限失败已在 `23148c1` 修复；随后运行由用户暂停而取消，不计为通过。最终远程结果待下面追加。

## 最终验收记录（2026-10-14）

结果：P0-01 实现及验收完成，PR [#1](https://github.com/nanake-lab/nemu/pull/1) 已提交评审；此记录提交后再次验证最终 HEAD，再按 AGENTS.md 合并条件处理。

| 任务卡验收项 | 结果与证据 |
| --- | --- |
| pnpm i / lint / typecheck / test / build | 全部通过；`final-validation/validation.json`、对应日志及远程 Ubuntu job |
| Cargo workspace tests | 通过；`final-validation/` 与远程 Ubuntu job；空骨架不代表核心业务验收 |
| 四个 Python 工程 pytest | 每个 1 项通过；严格类型、lint、构建也通过；同上 |
| Electron 两窗口、透明舞台 | 通过；`e2e-results.json`、`visual/electron-*.png` 与远程 Windows job；原生截图 alpha 断言保持有效 |
| Ubuntu + Windows CI | [运行 38440863374](https://github.com/nanake-lab/nemu/actions/runs/38440863374) 两组 job 均 SUCCESS，验证实现提交 `f0e3442` |
| Windows runner 安装说明 | `docs/SETUP.md`；本次 ephemeral runner 实际完成命名管道、Low IL、Electron 与视觉检查 |
| 旧原型迁移 | `migration.md` 记录来源、哈希、取舍；仅 Composer UI，测试通过，无私人内容或旧状态逻辑 |
| 其他交付物 | 本报告交付物清单与 `docs/traceability/P0-01.md`；锁文件、模板、依赖方向和 schema 有结构测试 |

历史限制：任务卡“首次运行全绿”没有在第一次尝试达成；首次 Windows SDK 安装权限失败已修复，以上为修复后的完整绿灯运行，不隐去失败或取消记录。

无尚未解决的实现/迁移/凭据阻塞，无新增 ADR、阈值放宽或已有视觉基准更新。只验收脚手架，不声称实现聊天、核心状态、真实角色渲染或生产隔离。初始占位视觉不是角色美术定稿；长期 runner 管理由 P0-05 第二部分承担。技能约束保持了 Echo Core 权威边界和固定白黑背景视觉证据。

下一步：最终文档提交的 CI 全绿且 main 未变更时，按 AGENTS.md squash 合并 PR #1；本次不开始其他任务卡。
