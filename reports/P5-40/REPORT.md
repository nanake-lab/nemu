重档。P5-40 修改 Echo Core 权限 IDL 与跨语言验证，须独立交叉审查和用户批准。

# P5-40 身份、执行凭证与成长管理认证

已提交本机授权与成长管理契约。本机契约验收通过；[草稿 PR #235](https://github.com/nanake-lab/nemu/pull/235) 保留「契约变更」与 `needs-user`。最终提交的 Ubuntu、Windows 实跑结果随 PR 的交付评论按 SHA 登记。等待审查及用户批准 ADR 0047 / V2-188。

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
| 当前 head CI | Ubuntu 由 PR 触发，草稿 Windows 按同提交手动触发。以 [PR 检查](https://github.com/nanake-lab/nemu/pull/235/checks) 和[分支 Windows 作业](https://github.com/nanake-lab/nemu/actions/workflows/windows.yml?query=branch%3Atask%2FP5-40-permission-contracts) 的实际运行及最终评论为准。 |

本机原始日志（含初次失败）、冻结字节及逐消息覆盖核验的私有路径、SHA-256 和证据提交见[证据索引](EVIDENCE-INDEX.csv)。较大的文件来自追加式黄金向量与三端生成代码。

主线增量核对从开工基线 `c5636361` 到 `469262e4`，新增 #201 的规律实施规划报告与请求，未改变 P5-40 的发布输入和验收条件。

## 待用户确认

- [ ] 阻塞。独立交叉审查后由用户批准 ADR 0047 与 V2-188。
- [ ] 默认待用户定。尚无已批准数值的连接器预算、额度、凭证有效期及钱包余额上限，须在相应资源发布前确定。不可挽回行动的默认档位已定为「先问用户，同意后做」。可挽回性判定规则由单独的版本化资源提供。

本卡没有运行生产认证、实际发送 I26、完整 SYS09、H/Q/L 原子写入、逐报警重启恢复或存储迁移 SYS17。这些系统验收由 P5-41 完成并交 P5-18 汇总。
