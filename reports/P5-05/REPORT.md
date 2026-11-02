# P5-05 权限谓词、原样载荷绑定与 worker 隔离（W22）— 报告

## 结果

完成用户批准的缩小范围实现及独立验收，等待交叉审查及最终提交 CI；未合并。
2026-10-26 用户批准 sha2 单文件依赖授权及拆卡。系统接线移交 **P5-40**，任务定义在
[PR #102](https://github.com/nanake-lab/nemu/pull/102)（只改 tasks/，needs-user，未合并）。
原阻塞的解除/移交记录见 [BLOCKED.md](BLOCKED.md)。
[PR #100](https://github.com/nanake-lab/nemu/pull/100) 的分支为 `task/P5-05-permissions`。
最新合入的 main 为 `86779b1`；合并提交 `79b295b0f2a960461f2e3ebd9b56120064b2301d`。
完整 SYS09、真实数据库/协议双层隔离和实际发送路径 I26 未验收，均由 P5-40 承接。

## 交付物

- `core/crates/action-gate/src/permissions/`：无持久状态的 T0–T3 权限谓词、成长命令
  入口/结构检查；原样 SHA256 绑定、确定性审批元数据、验证后自持字节及按值消费。
  使用已有生成类型；纯谓词、调用者事实及载荷快照均不是身份认证或执行凭证。
- `tools/low-il-launcher/`：独立 Windows 启动器和 OS 隔离测试。新建私有 Low 目录，
  降低独立启动器令牌，worker 挂起创建、核验 Low、加入 kill-on-close 作业后运行。
  无 shell、无继承句柄、环境白名单；关闭作业清理残留后代。
- 用户单次授权：`action-gate/Cargo.toml` 增加 `sha2 = "=0.11.1"`；Cargo 离线解析
  更新锁文件，只增加 action-gate 到已有 sha2 的依赖边，无新包或版本变更。
  `action-gate/src/lib.rs` 只追加模块登记一行，适用 AGENTS §2 共享例外。
- `tasks/requests/P5-05-permission-integration.md`：保留原请求并追加批准及移交结果。
- 本目录的 W22、I26、SYS09 证据、源文件哈希与原件 ZIP；旧证据原样保留。

## 验收

缩小后的命令和边界已写入 PR #102 的 P5-05 追加；未删除或放宽原系统条件。

| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 合并 main 后退出 0，无输出 | W22/rust-20261026-1926/results.json、fmt.log |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | 合并 main 后退出 0 | W22/rust-20261026-1926/clippy.log |
| `cargo test --workspace --locked --offline --exclude core-store` | 合并 main 后退出 0；106 passed（unit 88、integration 17、doc 1）、0 failed、1 ignored（见下文） | W22/rust-20261026-1926/workspace.log |
| W22 权限与成长结构 | 6 项测试通过：确认矩阵、T3 禁记忆、非法入口/身份事实/二次确认/缺引用/CLI-only review | permissions/tier.rs、growth/tests.rs；上述 workspace.log |
| I26 原样字节绑定局部 | 9 项测试及 1 项 compile-fail doctest 通过 | [I26/README.md](I26/README.md)、上述 workspace.log |
| `pwsh -NoProfile -File tools/low-il-launcher/test.ps1 -WorkRoot D:/Dev/cache/P5-05/os-split-20261026-1901` | 退出 0；两个项目均 0 警告/0 错误，14 次 Medium 写操作被拒；正对照、哈希/标签不变、环境/参数、后代清理通过 | [SYS09/os-20261026-1901/](SYS09/os-20261026-1901/) |
| 完整 SYS09 / 施工合同第一篇 10.9 / I26 实际执行路径 | 未运行，移交新卡 P5-40；OS 合成文件不替代真实数据库与协议验证 | BLOCKED.md、PR #102 |
| V2-89 / V2-101 / 管理审计及 H / P5-14、P5-29、P5-15 联测 | 移交新卡 P5-40 | BLOCKED.md、PR #102 |
| Ubuntu CI（合并提交 79b295b） | 通过；最后报告提交的状态另看 PR 检查，不能将旧提交结果冒充最终提交 | [run 40313253929](https://github.com/nanake-lab/nemu/actions/runs/40313253929)、CI/79b295b-checks.json |
| Windows CI（草稿阶段） | self-hosted skipped、汇总 draft guard failure；不是通过；转审后需执行最新提交 | CI/111443e-windows-draft-guard.json |
| 后继卡定义的 Docs guard | 提交 6e77cd8 通过 | [run 40309886052](https://github.com/nanake-lab/nemu/actions/runs/40309886052) |

原样绑定覆盖已知散列答案、CRLF/Unicode/NUL 不归一化、载荷/摘要/ID/说明改动、
到期边界、T3 allowAlways、源数组后续修改、回调失败和单个快照重复消费的编译拒绝。
摘要不是签名；tier/expiry 等元数据须由未来认证服务建立。重新验证同一 wire 请求仍可
创建另一快照，不能据此声称全局/跨重启防重放。成长结构检查不提交管理事件、不写 H。

工具链为 Rust 1.99.0、.NET SDK 10.0.401；全部本机重负载串行。只使用新建 D 盘临时目录
和合成文件，不打开排练/正式存储，不修改用户级或系统级设置。SDK 环境仅作用于子进程，
禁止首次运行生成开发证书或修改全局工具 PATH。Low IL 默认 LocalLow 部署路径未联测。
本次未改 core-store，按拆分范围运行非存储工作区组；core-store 专项不计为本卡通过。
合并 P5-25 后继承的唯一 ignored 是 `dynamics::tests::reference::theta_1_original_500_streams_zero_violations`，
原理由为 V2-93 的旧 Python `**2` 舍入诊断（证据 `reports/P5-25/rounding-20261024/REPORT.md`）。
本卡未改该测试或 ignore 条件，未将它计为通过；本卡 15 项测试及 1 项 doctest 全部执行。
本轮合并没有改变 Low IL 源码，因此沿用同一源码的 19:01 OS 验证；只重跑受工作区变更影响的 Rust。
无 TS/Python 改动，未运行 pnpm/pytest/Electron/GPU 测量。
资源排队不计为测试失败；记录见 resource-wait-split-20261026.txt 及最终运行日志。
[证据核验记录](evidence-verification-20261026.md) 包含机械子代理的计数和原件哈希结果，
主代理核对后归档；它不代替独立交叉审查。原件 ZIP 保留字节，防止 Git 换行规范化影响复核。

## 规格对照

本表保存在报告内，遵守本次仅修改指定目录及报告/请求的范围。

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| CONTRACTS §6：一般 T0–T3、T3 不可信任 | permissions/tier.rs | 确认矩阵、DENY、T3 ALLOW_ALWAYS 拒绝 |
| CONTRACTS §6 / I26：原样 SHA256、批准载荷一致 | permissions/approval.rs | 9 项局部正负例及 compile-fail doctest；完整发送接线移交 P5-40 |
| v2.0 §5.4/§10/§11、ADR 0010 决策 5、CONTRACTS §4.3 | permissions/growth.rs | 入口/调用者事实、面板二次确认、CLI-only review、生成命令/引用检查 |
| I18：无隐藏状态 | permissions/* | 纯函数与进程内快照，无权限缓存或持久信任；正式存储移交 P5-40 |
| 总方案 §21、施工合同第一篇 §8/10.9、I02 的 OS 局部 | tools/low-il-launcher/* | Medium 合成文件拒写和 Low 临时目录正对照；协议提交能力未实现 |
| I29 / ADR 0005：用户存储不作测试数据 | test.ps1、tests/Program.cs | 新目录/合成文件；拒绝复用已有 WorkRoot |
| V2-89 / V2-101 | P5-40 后继定义 | 社交规则/逐报警绑定未在本卡固化为旧规则 |

IDL、生成类型、黄金向量、权威存储格式均未改；没有新增 ADR 或差异登记。
本卡不新增规格解释，不将局部验证替代已移交的系统验收。

## 新增依赖

| 名称 | 版本 | 许可证 | 用途 |
| --- | --- | --- | --- |
| sha2（已有锁定包，新增 action-gate 直接引用） | =0.11.1 | MIT OR Apache-2.0 | 原始载荷 SHA256 |
| .NET SDK / Microsoft.NETCore.App | SDK 10.0.401 / 本机 runtime 10.0.12 | MIT；Windows API 为系统组件 | 启动器，仓库已有工具链；新项目无 NuGet 包 |

没有其他运行时依赖或资产产物。所有新增源码文件小于 400 行。

## 待用户确认

- [x] sha2 单文件依赖授权及缩小范围拆卡：2026-10-26 已批准。
- [ ] （不阻塞本卡独立验收）审阅后继任务定义 PR #102，保留 needs-user；未来 ADR/IDL 尚未获批。
- [ ] （合并前必需）独立交叉审查；本报告不是审查结论。最终提交 CI 也须通过后才可合并。

## 后续建议（不在本任务范围内）

P5-40 接通认证/凭证/预算/作用域/信任记忆、管理审计/H、守护和 CI，并执行原系统条件。
Low IL 提供写入完整性边界，不提供保密或网络隔离；可信部署方必须保护执行文件及祖先
路径，路径预检不能抵御并发替换。后继卡完成前不得声称生产授权边界闭合。

## 下一步

最新 main 后复验和证据归档已完成；提交报告后将 PR #100 转为可审查并写「请审查」。
等待独立交叉审查及最终提交 CI；未合并，不取下一张卡。P5-40 当前仅有任务定义。
