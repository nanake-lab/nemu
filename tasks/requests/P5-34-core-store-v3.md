# P5-34 请求：core-store 接入 V3 R 契约

- 日期：2026-10-23；状态：待用户安排范围与执行卡。
- 提出卡：P5-34；草稿 PR #72；依据 ADR 0021、V2-62。
- 请求目录：`core/crates/core-store/`，不在 P5-34 可修改范围，本卡不改。

## 原因

最新 main 的 `core-store` 已有 V2 消费者：`objects.rs::prepare_r`、`store.rs::prepare_r`、`publication.rs`、`queries.rs`、`transaction.rs` 与相应测试。P5-34 按卡要求把 TendencyV2、EvidenceAllocationV2、EvidenceGroupV2、RSnapshotV2 标为取代，新活跃类型为 V3。

`objects.rs::prepare` 明确拒绝 `T::SUPERSEDED`，所以现有 EvidenceGroupV2 的新写入会被拒绝；V2 专用的 `prepare_r` 仍写旧类型，通用 `Store::prepare` 只拦 V2 快照，尚未识别 V3 快照。这是必须处理的消费者迁移，不能由核心类型兼容检查代替。

## 期望接口与约束

1. 新生产者/prepare_r/物理分段、提交闭包验证和证据组查询切换至生成的 V3，不能手写替代类型；V3 快照也只能经专用分段路径写入。
2. 保留已发布 V2 对象的读取、摘要与必要的历史校验。明确区分历史读取和新生产，不能为通过旧测试放开所有被取代消息的新写入。
3. 调用 q/V 语义检查，并绑定所属规律参数后核对 B、π、r；迁移从有效 H 重建 q/V，禁止 q默认1、V=B−c。
4. 更新测试夹具，验证 V3 的单写、对象闭包、索引和重放，以及旧 V2 仍可读。测试只用 TemporaryRoot，不打开用户数据。

## 用户可选处理

- 扩大本卡范围并安排存储交叉审查；需用户授权后实施。
- 另开契约消费者迁移卡/补丁 PR，与本卡协调合并，避免 main 的新写入路径落在旧类型上。

本请求不代替用户批准，不放宽测试，不修改存储或契约现有约束。
