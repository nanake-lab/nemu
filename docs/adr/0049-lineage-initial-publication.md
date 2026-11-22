# ADR 0049　空存储的第一份提交

提议中，须交叉审查并由用户批准。P5-52 交付线第 1/3 部分，2026-11-22。

## 背景

W02 已有单写者、持久入口和原子发布。V4 事务从已绑定的输入或内部活动开始，两者都引用既有状态。新存储缺少第一份可读状态，目前运行时会返回 `genesis/bootstrap publication contract required`。[发布请求](../../tasks/requests/P5-08-runtime-publication.md) · [运行时](../../core/crates/core-runtime/src/store.rs)

## 决定

**初始提交。** `InitialPublication` 只用于没有父提交的新分支。它绑定存储身份、分支、`LineageHeaderV2`、x/R/θ 快照、初始 Q 和 H 的起点。x 采用当前规律对应的 FastStateSnapshotV2/V3，R 采用 RSnapshotV3，θ 采用包含完整估计器的 ThetaSnapshotV2/V3。发布者按所选规律逐个解析引用，核对规范摘要、类型、资源版本、长度、投影维度和初始值。排练使用该排练配置，实验使用相应合成世界，正式人生使用已签署的 genesis。具体数值、资源和档位保存在既有版本化资源内。[IDL](../../core/contracts/initial_publication.proto) · [v2.0 §§2、13–16](../echo-core/echo-core-v2.0-first-principles.md)

| 首次发布的部分 | 存储需要固定的内容 |
| --- | --- |
| H | 排练和实验从 `EmptyPrefixRoot(EMPTY_HISTORY)` 开始。LIFE 的 H 从 `BirthNameRecord` 开始，照料者给的名字随诞生在同一事务成为第一条事实，记录身份是当前分支与 record_id。 |
| L、U | 分别发布 `EmptyPrefixRoot(EMPTY_CHANGE_LEDGER)` 与 `EmptyPrefixRoot(EMPTY_INGRESS_DECISIONS)` 的规范对象。两个值的字段由角色唯一确定，初始清单由此导出这两个引用，ObjectRef 的资源版本为该消息的 schema 版本字符串 `1`。初始 U 表示尚无已处理入口，已持久到 D 的待处理记录仍按原顺序排队。 |
| Q | 发布清单内 `InitialRuntimeRoot` 的规范对象，资源版本同为 `1`。初始 Q 没有活动、行动、预留、消费、暂停命令或待复核警报。此后暂停由本机已认证的操作记入 H，复核按警报分别登记，恢复读取最新 Q。后续认知工作状态复用 CognitionState。接线方必须保留阶段与管理状态，并从同一提交读出完整 Q。 |
| x、R、θ | 逐个固定清单中的原样引用。准备、发布和读回均使用同一谱系、分支和同一版本组合；正式人生的初始快照须能从该清单签署的资源重建。 |

**阶段。** 初始 Q 内保存 Ω 以及已接通阶段的实现引用。IRL、A、M 分别匹配 Ω 的 irl、nature_appraiser、situation_matcher。F、G、C、J 的实现必须列在 Ω.implementations；F/G/C 绑定规律，J 绑定方向集。比较完整 ObjectRef（含版本和长度）。Trusted Core 还须用本机实际接通的适配器及受信资源重核这些绑定。阶段引用仅供核对，无法授予写权限。

| 谱系 | 接通条件和初始运行状态 |
| --- | --- |
| REHEARSAL | G、C 恒等，两者均不登记为已启用，报告 NO_GROWTH。开发期允许缺阶段。F/J 未就绪时走 CORE_UNMODULATED；IRL/A/M 缺失按现有降级路径报告。F/J 接通时按实际状态计算 m。 |
| LIFE | 接齐 IRL、A、M、F、G、C、J。从诞生起成长。核对本机认证的 u*、当前 birth 操作、签名证据、原版本 genesis 及谱系头的摘要。解析 genesis 后其 caretaker_id 必须与已认证的 principal_id 一致。 |
| EXPERIMENT | 同样接齐七阶段；使用谱系头绑定的合成世界与种子，逐项验证资源来自该实验谱系。 |

[ADR 0005 的修订](0005-rehearsal-and-birth.md) · [ADR 0010](0010-v1-scope-birth-and-lineages.md) · [本机身份及成长管理](0047-permission-contracts.md) · [已有阶段能力](../../core/crates/core-runtime/src/stages.rs)

**发布与恢复。** 单写者在同一事务内重核可信存储身份、分支、头部、完整性见证和无父提交条件；LIFE 还需持有诞生权限、核验封存状态和当前可验证的唯一活跃实例。签名证据按对应的验证资源复验。上述验证在本机受信入口完成。初始清单、H/x/R/θ/L/Q/U 及消费位置一起成为可读头。创建前崩溃可重新准备；成功后相同请求读回已提交结果，冲突请求拒绝。已有提交、历史缺口、丢头或旧备份须走核验恢复。恢复保留 D 的待处理记录、既有 Q 屏障及 U 的决定。时间证据缺失时维持未知，首次发布本身不证明此前的运行时长。[ADR 0028](0028-lifecycle-single-instance.md) · [DESIGN-CLOCK-01 §§5–6](../../reports/DESIGN-CLOCK-01/REPORT.md)

签名证据引用 W02 的非空原始字节对象。验证器按原版清单及其签名资源核验签署人，在已认证的本机 birth 操作中核对存储、分支、清单和这次给的名字。V2 签名只覆盖其原字节，升级后的 V3 需要可复验的授权。名字记录或本机身份记录从磁盘读出后都不能单独启动诞生。[ADR 0036](0036-j-resources-and-effect-receipt.md)

Rust 提供两处受信调用入口。`check_initial_store_binding` 把整份初始记录同接收存储及实际阶段逐一核对。`verify_birth_material` 用已认证的本机身份核对原版清单、签名材料和照料者，把原始清单与签名交给调用方的验签器。单写者随后重核封存和唯一实例并提交。[消费检查](../../core/crates/core-types/src/semantic/initial_publication.rs)

## 备选与原则核对

| 新内容 | 能否省去 | 人会怎样 | 冻结后保留什么 |
| --- | --- | --- | --- |
| 空前缀和初始清单 | 借用 UserText 或伪造一次事件会混淆未处理、已处理与用户亲历。单独的可重建清单复用现有单写者。 | 排练、合成世界与正式出生各有自己的开始。 | 每份初始状态的谱系、快照和首条事实可以按原字节重放。 |
| 阶段实现绑定 | 单个「全部接通」标记无法定位实现和版本，恢复时无法复验。 | 能力来自实际具备的条件。 | 记录当时接通的版本，当前可用性仍需核验。 |
| 名字与签署凭据 | 当前 genesis 清单没有第一条名字事实，已有本机身份类型可直接复用。 | 给名字的人在诞生时亲自确认。 | 当时的名字、签署的清单、身份与证明保留在该人生的 H。 |

上述约束对应 [ADR 0039](0039-echo-core-principles.md) P3、P5、P7、P9。新增枚举值及字段号标识编码。用户设置继续按其版本资源调整，调整留有记录。

## 兼容与后果

本次新增独立的初始对象；V4 继续处理已有头上的事件及活动，旧 genesis 清单按原标签验证，旧存储沿原路径读取。后续新增事务后继时以 `@retains` 保留仍有生产者的前驱。三个终端上传入口均拒收本次的新标签。生产启用前由 P5-53 核验原子发布、真实首条消息、封存和回退；按 SYS17 从本地备份恢复旧版本。读取时不得自动给无依据的旧头补初始对象。

下一部分以本次初始身份为起点，按 U1–U7 定义 checkpoint、恢复和同一缺口共享的整数纳秒证据范围；第三部分把这些证据与处理、提交、IRL→z→A/M 的绑定一起发布。区间估计器、长段数值版本、重算和迁移由 [DESIGN-CLOCK-01 §11.1](../../reports/DESIGN-CLOCK-01/REPORT.md) 指定的后继交付。
