# P5-52　重档 · 初始根的 SYS21 契约验收（1/3）

本次定义了新存储的第一份状态。排练从空历史开始，正式人生的诞生记录把照料者给的名字与签署的原版 genesis 一起记入首条历史。
这为 P5-53 和 HOST-01 接通第一条真实消息提供了初始状态契约。
请安排交叉审查，并确认 ADR 0049 与差异登记 V2-191。

## 本次交付

四个独立类型分别表示 H/L/U 的空前缀、实际接通的阶段、诞生名字记录和首次发布清单。Rust 提供接收存储与阶段的核对函数，以及向受信验签器提交清单原件和签名字节的入口。终端在上传首块及接收完整对象时均拒收四类权威对象。使用 V2、V3 genesis 的生产者保留原标签。未新增运行时依赖或可调数值，既有人为档位沿用版本化资源。[ADR 0049](../../docs/adr/0049-lineage-initial-publication.md) · [IDL](../../core/contracts/initial_publication.proto) · [规格条款与测试](../../docs/traceability/P5-52.md)

## 验收

2026 年 11 月 22–23 日在 Windows 本机使用合成夹具和临时存储。实现提交 `6b281c877837d8276d64e23abd9dc01077c8fe50`，冻结起点 `306762284e01241d2ce85a86d7ef44c46ad4ce8b`。最后的报告提交只增加验收与追踪文档。

| 检查 | 本次结果 |
| --- | --- |
| `python core/contracts/generate.py --check`，`check_compat.py --ref origin/main` | 三种语言的生成物均为当前版本。220 个消息、82 个枚举，兼容违规 0 项。与 main 的 14 份 IDL 均逐字节相同。原有 779 个合法、1,073 个非法、63 个上传入口及 18 个 Unicode 向量逐项保留。 |
| `pnpm test:core-contracts`（SYS21） | Rust、Python、TypeScript 各通过 20,218 例，其中合法 4,442、拒绝 15,776。官方 protoc 和 Protobuf 运行库验证了全部 795 个合法传输向量。新增 16 个合法、152 个非法冻结向量。另用与声明匹配的规范字节逐项检验，四个新标签 × 两个入口 × 三种语言共 24/24 次拒收，原有正对照 6/6 次接受。 |
| `cargo test -p core-types --locked` | 通过。覆盖错误存储、分支和实验谱系、实际阶段不匹配、V2 和 V3 两版原始清单、本机身份和会话、签名材料被替换及验签器拒绝。冷缓存、热缓存与无缓存路径对所有合法及非法向量结论一致。 |
| Rust 工作区 | `cargo fmt --all -- --check` 与全工作区 Clippy（所有 targets，警告视为错误）通过。全工作区测试分两步通过，先测除 core-store 外的包，再串行运行 core-store 的 69 项测试与 3 项文档测试。 |
| TypeScript | `pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm test:transport` 通过。根单测 620 项、契约包 354 项通过。 |
| Python | 契约 Ruff、严格 mypy、pytest（14 项）通过。IRL、drive、model-adapter、voice-sidecar 的 Ruff、严格 mypy 和打包通过，pytest 分别通过 28、21、1、1 项。 |
| 文档与基线 | 新增相对链接逐项可解析。`git diff --check` 默认模式报告冻结副本 `growth.proto:300` 的末尾空行。逐字节比对确认它来自 main，按副本识别的检查通过。 |

实现提交触发的 PR 检查中，Ubuntu 作业通过。草稿条件使自托管 Windows 作业跳过，因此汇总门禁报失败。Windows 本机实测见上表。[Ubuntu 运行](https://github.com/nanake-lab/nemu/actions/runs/45038403656) · [草稿 Windows 运行](https://github.com/nanake-lab/nemu/actions/runs/45038403657)

**初次失败与处理。** 上传检查最初接受了新的空前缀标签，现已把四个标签加入三种语言的受保护类型，并给完整对象提供正确字节、长度与摘要重测。Rust 测试先有一次借用编译错误。修复后，缓存一致性测试又发现其检查入口漏调用新语义验证器，现已接入。单独复核还发现同一份诞生清单更换资源版本后可被接受，已按 ADR 0036 绑定原版清单的 schema 版本，并对 V2、V3 的独立和嵌套记录重测。上述失败输出均随复测结果归档。

原始日志、测试命令及逐文件摘要保存于私有证据仓库。归档 `P5-52/SYS21/initial-roots-windows.zip`，SHA-256 为 `b4e61d8c673f69223da44aead4f791f5ad270e3131dce25b5bff58ae9b2aa51d`，证据提交 `afc700ce7d4224d4d8e37c2967aa9f3f631a96b1`，已从该提交读回并比对归档字节。[打开证据归档](https://github.com/nanake-lab/nemu-evidence/blob/afc700ce7d4224d4d8e37c2967aa9f3f631a96b1/P5-52/SYS21/initial-roots-windows.zip)

## 待审查与后继

- **待批准**。本部分保持草稿。用户安排交叉审查后，确认 [ADR 0049](../../docs/adr/0049-lineage-initial-publication.md) 与 [V2-191](../../docs/traceability/diff-register.md)。
- **2/3 时钟恢复**。一周未开机后重启、开机长时间无输入两个场景，本次均为 `not_run`。后续先验证可信区间、客观经过时间、运行年龄与既有 R/θ 的契约。Windows 真实时钟及第一条消息由 P5-53、HOST-01 接线后验证。U4 乙候选 A 在 P5-52 内只表示时间事实、共享缺口及整数纳秒证据范围。区间估计器状态与规律实现由数值与存储后继接手。[已定方案与分工](../DESIGN-CLOCK-01/REPORT.md) · [候选 A](../MATH-RANGE-C-01/PRO-REPORT.md)
- **3/3 权威发布**。处理、提交时点、IRL → 已接受 z → A/M 的绑定及原子发布尚待交付。真实诞生、验签与封存、P5-53 空头提交和产品级 SYS17/SYS23 本次均为 `not_run`。[P5-52 追踪表](../../docs/traceability/P5-52.md)
