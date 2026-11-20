重档。P5-40 修改 Echo Core 权限 IDL 与跨语言验证，须独立交叉审查和用户批准。

# P5-40 身份、执行凭证与成长管理认证

已提交本机授权与成长管理契约。本机契约验收通过；[草稿 PR #235](https://github.com/nanake-lab/nemu/pull/235) 保留「契约变更」与 `needs-user`。各次 CI 实跑结果随 PR 评论按提交号登记。实现提交的 Ubuntu 通过；Windows W18 在网关等待回执时超时，开工基线也在相同行超时。见 [BLOCKED.md](BLOCKED.md)。当前 head 的 W18 通过才能解除 CI 阻塞，另待独立审查与用户批准 ADR 0047 / V2-189。

## 交付

- [permission.proto](../../core/contracts/permission.proto) 增加 14 种消息（其中 4 个后继）和 4 个枚举，含可选桌面档位、按次确认、签发凭证、预算/撤销、信任及逐报警的管理请求/回复。云端分支复用 RemoteCognitiveReceipt，无法携带本地授权。三端生成物均已更新。
- [Rust 消费接口](../../core/crates/core-types/src/permission.rs) 比较受信发行记录、会话和 u* 绑定。三端语义层检查原样执行载荷和凭证、预算、期限、档位、同意和管理消息的内部一致性。终端上传在三端拒绝全部新标签。
- [规格与 P5-14、P5-29、P5-15 的取用清单](../../docs/traceability/P5-40.md) 列出每项结构约束、运行时拒绝条件和 P5-41 的联测入口。[ADR 0047](../../docs/adr/0047-permission-contracts.md) 和 [CONTRACTS 追加](../../docs/CONTRACTS.md) 说明当前默认、历史记录、崩溃恢复及退役条件。没有新增依赖。

## 验收

| 检查 | 结果 |
| --- | --- |
| `python core/contracts/generate.py --check`，`python core/contracts/check_compat.py --ref origin/main` | 0 个生成物过期，全部冻结基线、现行 IDL 及 main 为 0 项违规。 |
| `pnpm test:core-contracts` | Rust、Python、TypeScript 各 19,771 例，含 4,360 个正例、15,411 个反例；官方 Protobuf 往返 779 条。 |
| 开工前 IDL 及向量冻结 | [基线](../../core/contracts/baselines/contract-wire-01/golden.json) 固定 #234 的 13 个 IDL 文件和四组前缀。原 726 正、988 反、63 入口和 18 Unicode 向量逐条相同。追加 53 正、85 反，每种新增消息至少 3 正、3 反。 |
| `cargo test -p core-types` | 通过，含冻结向量的冷/热/无缓存路径、受信主体对照、过期边界及帧预算。首次测试编译发现新测试闭包持有借用，已用独立的变更会话夹具修正并复验。 |
| `pnpm test:contracts` | 8 个测试文件，354 项通过。 |
| fmt、core-types clippy、契约 Ruff、严格 mypy、根 lint 与 typecheck | 通过。首次 mypy 提示新夹具中三处字典类型推断过窄；补上类型后严格检查通过。 |
| 当前 head CI | 实现提交 bd750680 的 Ubuntu 通过。Windows 首次上传证据遇到 ECONNRESET；重跑后桌面与 W02 通过，W18 为 41 通过、1 项回执读取超时。草稿自动门卫按设计报红。以 [PR 检查](https://github.com/nanake-lab/nemu/pull/235/checks) 和[分支 Windows 作业](https://github.com/nanake-lab/nemu/actions/workflows/windows.yml?query=branch%3Atask%2FP5-40-permission-contracts) 的实际运行及最终评论为准。 |

实现提交两端 CI 的原始日志、重跑、门卫及工件已存入[私有 CI 证据索引](https://github.com/nanake-lab/nemu-evidence/blob/0c89abccfaf6c098350ad972182bb6910010366e/P5-40/ci/index.json)。本机原始日志（含初次失败）、冻结字节及逐消息覆盖核验的私有路径、SHA-256 和证据提交见[证据索引](EVIDENCE-INDEX.csv)。较大的文件来自追加式黄金向量与三端生成代码。

本次合入主线 `a13e09f8`。#239 已使用登记号 188，P5-40 顺延为 V2-189，ADR 编号仍为 0047。本轮核对差异空白与 IDL 生成物。按[审查评论](https://github.com/nanake-lab/nemu/pull/235#issuecomment-7039794089)，W18 待网关修复合入后同步主线再测。

## 待用户确认

- [ ] 阻塞。独立交叉审查后由用户批准 ADR 0047 与 V2-189。
- [ ] 默认待用户定。尚无已批准数值的连接器预算、额度、凭证有效期及钱包余额上限，须在相应资源发布前确定。不可挽回行动的默认档位已定为「先问用户，同意后做」。可挽回性判定规则由单独的版本化资源提供。

本卡没有运行生产认证、实际发送 I26、完整 SYS09、H/Q/L 原子写入、逐报警重启恢复或存储迁移 SYS17。这些系统验收由 P5-41 完成并交 P5-18 汇总。
