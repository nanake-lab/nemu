# P5-05 权限、身份与 worker 隔离（W22）— 报告

## 结果

部分完成，见 [BLOCKED.md](BLOCKED.md)。基于 origin/main
`a0c9ed865f8c46c0225b778a4fb1c7a2b3825177`，分支 `task/P5-05-permissions`，
[PR #100](https://github.com/nanake-lab/nemu/pull/100) 保持 draft。
认证、执行凭证、权限持久化和真实执行接线未完成；SYS09/I26 不报告通过。

## 交付物

- `core/crates/action-gate/src/permissions/`：一般 T0–T3 确认需求、审批选择约束、成长命令
  入口/当前用户/二次确认/生成契约检查。纯函数没有持久状态；返回值不是认证或执行能力。
- `tools/low-il-launcher/`：独立 .NET 启动器、Win32 边界和 OS 隔离测试。
  新建私有 Low 目录，降低本启动器的令牌；worker 挂起创建、验证 Low 后入作业再恢复，
  退出关闭作业并终止残留后代。无 shell、无句柄继承、环境只用白名单。
- `action-gate/src/lib.rs` 只追加模块登记一行（AGENTS §2 共享登记例外）。
- `tasks/requests/P5-05-permission-integration.md`：卡外契约、依赖、服务接线及 CI 请求。

## 验收

任务卡的「验收」为四组行为条件，没有提供具体 shell 命令。下面逐项映射，不能用局部测试替代。

| 命令或检查（含验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过，退出 0，无输出 | `W22/rust-20261026-1739/results.json` |
| `cargo test -p action-gate --locked --offline` | 6 项通过 | `W22/rust-20261026-1739/action-gate.log` |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 通过，退出 0 | `W22/rust-20261026-1739/clippy-workspace.log` |
| `cargo test --workspace --locked --offline --exclude core-store` | 42 项通过 | `W22/rust-20261026-1739/test-workspace.log` |
| `pwsh -NoProfile -File tools/low-il-launcher/test.ps1 -WorkRoot D:/Dev/cache/P5-05/run-20261026-1730` | 通过；两个项目各 0 警告/0 错误；14 次 Medium 文件/目录写操作被拒，Low 临时目录可写、后代清理/参数/环境检查通过 | `SYS09/os-20261026-1738/`（只代表 OS 局部） |
| SYS09：计划后撤权、实际发送前拒绝 | 阻塞，未运行；确认策略重算不等于实际执行路径撤权 | BLOCKED.md / 接口请求 |
| SYS09 / 施工合同第一篇 10.9：worker 和 UI 无直接写权威存储；OS+协议两层拒绝 | OS 合成文件检查通过；真实数据库/协议联合验收阻塞，整项未通过 | 启动器测试 / BLOCKED.md |
| 成长命令：worker/helper/network 拒绝，缺少复核引用拒绝 | 纯谓词测试通过；真实身份认证、审计和 H 提交未接线 | `permissions/growth/tests.rs`、action-gate.log |
| I26：批准后改变载荷，审批失效，与 P5-14 联测 | 阻塞，未运行 | BLOCKED.md / 接口请求 |
| Ubuntu CI（实现提交 `a17888d`） | 通过 | [run 40301867746](https://github.com/nanake-lab/nemu/actions/runs/40301867746)、`CI/a17888d-checks.json` |
| Windows CI（实现提交 `a17888d`） | self-hosted 作业 skipped；汇总 failure，明确原因为 draft guard；不是通过 | `CI/a17888d-windows-guard.json` |

测试只使用新建临时目录与合成文件，不打开排练/正式存储，不更改用户级或系统级设置。
脚本的 SDK 环境仅对当前子进程有效，禁用首次运行创建开发证书/修改全局工具 PATH。
全部构建/测试串行。首次本机尝试因 CI 占用而未启动，记录在 `resource-wait-20261026.txt`；
空闲后首次实际 .NET/Rust 检查均通过。Rust 使用锁定的 1.99.0；工作区测试采用仓库 ci.yml 的
非存储组命令，未重跑未改动的 core-store 专项崩溃/掉电套件；不把它计为本卡协议验收。
无 TS/Python 改动，未跑 pnpm/pytest/Electron/GPU 测量。默认 LocalLow 路径由 Known Folder API
取得；测试使用显式 D 盘新目录，未在用户 LocalLow 下跑默认路径的部署联测。
机械性证据核验由 `routine_reporter` 收集：6/42 项测试计数与五份 OS 日志的原件哈希逐项吻合，
主代理核对后归档；不是交叉审查。原始 OS 日志同时保存在 zip 内，避免文本换行规范化影响字节校验。

## 规格对照

| 规格条款 | 实现位置 | 测试/缺口 |
| --- | --- | --- |
| CONTRACTS §6 T0–T3、T3 不可信任 | permissions/tier.rs | 确认矩阵、DENY 和 T3 allowAlways 拒绝；不持久化信任 |
| v2.0 §5.4/§10/§11，ADR 0010 决策 5；CONTRACTS §4.3 | permissions/growth.rs | 非管理入口、身份事实、面板二次确认、CLI-only review、引用和枚举检查；不签发管理能力 |
| I18 | permissions/* | 无权限缓存或隐藏状态；认证/记忆等待正式接口 |
| 总方案 §21、施工合同第一篇 §8/10.9，Windows Low IL | tools/low-il-launcher/* | 合成文件写入边界；主进程调用与真实存储集成未完成 |
| I26 / SYS09 | 接口请求 | 未实现完整审批/执行凭证与实际发送前核验 |
| V2-89 / V2-101 | BLOCKED.md | 不固化旧社交规则，不把旧 GrowthCommand 形状检查当作新版报警复核服务 |

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| .NET SDK / Microsoft.NETCore.App | SDK 10.0.401 / 本机 runtime 10.0.12 | MIT（.NET）；Windows API 使用系统组件 | 启动器；仓库已有该工具链，新项目无 NuGet 包 |
| Rust crate / npm 包 | 无新增 | — | 仅使用已存在的 core-types 生成类型 |

所有新增源码文件均小于 400 行。没有资产产物或 manifest 变更。

## 待用户确认

- [ ] （阻塞）卡外依赖 manifest 的范围授权，以及契约/服务接线的归属与后续安排，见请求文件。
- [ ] （阻塞）完成整卡验收后，由独立会话按 AGENTS §6 做高难度交叉审查。

## 后续建议

完成所属契约卡与 P5-14/P5-15/P5-29/桌面守护接线后，重跑真正临时 core-store 的双层隔离、
撤权/过期/重放/载荷变化联测。Low IL 只提供写入完整性边界，不提供保密或网络隔离；
执行文件及祖先目录由可信安装/守护方保护，路径预检不能代替防并发替换的部署保障。

## 下一步

卡内局部验证和证据已完成，保持 PR #100 draft / needs-user；集成阻塞解除后完成剩余验收，再转为可审查并写「请审查」。
不合并，不取下一张卡。
