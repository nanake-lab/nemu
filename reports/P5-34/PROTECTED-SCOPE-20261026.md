# P5-34 受保护存储路径：用户批准的分阶段交付

## 决定与依据

用户于 2026-10-26 明确选择分阶段交付：新写入一律使用带 q/V 的 V3 后继消息；已有 V2 历史保持只读、可解码；旧 V2 head 拒绝继续写入并明确报错。当前没有正式或排练存储，现存数据仅为测试数据（用户提供的事实，本卡未访问用户数据核实）。本次不做 V2→V3 数据迁移。

将来需要迁移时，由 **P5-26** 使用规律 G 从 H 整段重放，产生带 q/V 的新快照（v2.0 §2.2），不另开迁移卡。本次不增加重放回调、迁移输入协议或 G 算法；不会用 q=1、V=B−c 填充缺失状态。第 7/9 节旧 ADR 所述扩围不是本次实现；最新决定见 ADR 0021 §11。

之所以不能只转换旧对象：EvidenceAllocationV2 没有首次计入时的历史 q；TendencyV2 没有独立 V。例如 c=4、V=2^-53 时 B 舍入为 4，B−c=0。当前 core-kernel 只实现 effects/fast，core-replay 尚无完整 G/H 重放器。此前已按用户要求停止更大范围的迁移，提交具体选项，本文件由用户明确选择后更新。

## 文件范围：契约变更的必然后果

以下路径均位于受保护的 core/crates/core-store/，由用户专项授权纳入本 PR，并须在 PR 描述中单列、一起交叉审查。

| 文件（相对 core-store/src/） | 原因与改动 |
| --- | --- |
| objects.rs | prepare_r 改为 RSnapshotV3/TendencyV3；NSI1 重组接受旧/新快照标签，各走原编解码器与摘要校验 |
| store.rs | 专用 R 写入口使用 V3；通用入口同时拒绝两代 R；持有原 writer 锁，用 SQLITE_OPEN_READ_ONLY 预检旧 V2 head，在建立任何读写连接及配置/回执修复之前拒绝，并指向 P5-26 G/H 重放 |
| transaction.rs | staged R 与可写 head 的 revision 从 V3 读取；旧 V2 无法经 stage 发布 |
| publication.rs | 新提交的倾向/证据组索引从 V3 提取；使用原表、原查询含义、原原子提交 |
| queries.rs、lib.rs | evidence_for 返回 EvidenceGroupRecord::V2/V3，显式保留历史消息代际；导出包装枚举。枚举仅包裹生成类型，不是第二份 wire schema。最新版本优先、预算不足拒绝等 SQL/边界不变 |
| tests/mod.rs、objects.rs | 当前生产夹具改为 V3；修改 A 的夹具同步声明 V≥|A|，保持原有规范重组、复用、闭包和预算断言 |
| tests/crash.rs、crash_harness.rs | 原故障注入点的类型标签改为 V3，阈值与故障断言不变 |
| tests/legacy_v2.rs | 仅 cfg(test) 构造历史 V2 提交与派生索引；生产 API 没有旧写入逃生口；来源为旧历史夹具，适配为独立临时测试 |
| tests/retention_versions.rs | 旧平面/分段 V2 可读，正常关闭及非空未检查点 WAL 两种情形拒绝 writer 后持久字节不变；V3 q/V 相邻/次正规位值经过提交、重启、GC 后保留；可继续新写；拒绝旧新写与通用 R 绕行 |

SQLite 格式版本、表结构、NSI1 物理布局均不变。新逻辑对象有独立类型标签、字段布局和摘要；这不代表完成旧谱系的语义迁移。core-store 保持原结构校验职责，规律参数身份、完整 q/V 语义真实性与历史重放由其生产者和 P5-26 保证，不在本次加一套存储侧规律实现。

## 验收与限制

2026-10-27 复审修复仅涉及上表 store.rs 与 tests/retention_versions.rs：读写连接在拒绝路径关闭时仍可能执行 SQLite 最后连接检查点，因此“配置前拒绝”不足以满足原只读决定，必须在读写连接建立前拒绝。新增回归先在旧实现复现 DB/WAL 改写，再验证只读预检保留原字节；这属于既有 V2 拒写契约的必然后果，不增加迁移、回退接口、存储格式或预算行为。原正常关闭、V3 提交/恢复和故障注入测试不变，随受保护路径一起交叉复审。

命令与原始证据写入 verification/protected-20261026/，最终结果见 REPORT.md。新增测试在 TemporaryRoot 中构造数据；没有打开正式或排练存储。旧 head 拒绝检查验证的是分阶段只读边界和旧字节保留，不标成完整迁移/回退 SYS17 通过。旧程序无法解读 V3 对象，不能把新 V3 谱系原地降格；将来迁移仍须由 P5-26 提供备份、失败回退与 SYS17 证据。

PR #65 的已公开扩围包含 queries.rs 的追加接口，2026-10-26 解除暂停后已合入 main；本次按下述范围保留双方接口，未改 PR #65 分支。

## 2026-10-27 合入 #65 的组合范围补记

本次按用户要求 merge main 88455673。`core-store/src/queries.rs` 无冲突保留 #65 的 `transactions_at` 和本卡的 `EvidenceGroupRecord::V2/V3`、`evidence_for`；`tests/reader.rs` 的 #65 回归原样合入。没有新增存储格式、迁移路径或权限行为。额外适配仅为 `fact-view/src/tests/fixture.rs` 一行 RSnapshotV2→RSnapshotV3，使用已有 V3 黄金样例满足 `prepare_r` 新写入接口，不改生产逻辑、断言或预算。这是两卡接口组合的必要结果，与受保护路径一起交原审查者增量复核。WAL 与上传实现保持复审已认可版本，分阶段交付决定不变。
