档位：轻档；本 PR 只追加审计报告与复现说明，诊断探针在仓库外运行，不改实现、契约或既有测试。

# AUDIT-STORE-01 — core-store 权威历史审计（交付线）

本次确认两项合法运行路径可触发的问题：同一提交内的账本条目会被查询接口静默换序；初始化中途崩溃后，恢复会接受缺少拒绝审计回执目录的存储，首次拒绝审计落库后使后续打开失败。尚未发现正常写入路径永久丢失、覆盖或重复 H 的反例。另有一项恢复与垃圾回收的边界加固，必须先人为破坏派生引用表才能触发，单独列出。FIX-CORE-01 的已知证据索引问题只核对影响范围，不重复编号为新发现。

## 基线与状态

- 仓库：`nanake-lab/nemu`；审计基线：`ce115abe6ced3ec8cc301bea2a290f47ace4807a`（开始时最新 `origin/main`）。以下行号均指该提交。
- 只读范围：`core/crates/core-store/` 全部生产模块及测试；为判断可达性追踪生成契约、gateway 的接收/重试路径、core-runtime 的存储适配、fact-view 的查询入口。
- 本 PR 交付：本报告、[复现说明](REPRO.md)。不修改程序、不放宽验收、不增加依赖、不改变 IDL/黄金向量、不新增 ADR/差异登记。
- 状态：审计及诊断完成，等待独立审查；不合并、不续卡。

## 正常路径发现

### F1 · P2：`ledger_at` 按 ID 重排同一提交内的账本，违背权威追加顺序

**位置与依据：** [queries.rs:71–79](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/queries.rs#L71-L79) 使用 `ORDER BY c.position,l.entry_id`。[state.proto:293–312](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/contracts/state.proto#L293-L312) 明确 `staged_ledger` 是 G/C 的 **append order**，并非按 ID 排序的集合；v2.0 §2.4–2.5 的 L 同样是追加账本。[transaction.rs:250–261](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/transaction.rs#L250-L261) 逐条入索引却未保留其事务内位置。[recovery.rs:129–140](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/recovery.rs#L129-L140) 按条目身份核对内容，没有核对查询顺序。

**合法运行状态能否触发：能，在 W02 公开存储接口所允许的规范事务上。** 同一事件有两个账本条目，ID 唯一、非空，追加顺序与字典序不同即可。不需要崩溃、并发、损坏 SQLite、绕开类型能力或缺失对象。现有契约没有要求 ID 与追加序同序；不同倾向的两项变化也不要求按 `entry_id` 排列。

**触发路径：** `ingest → bind → stage([z-first,a-second]) → commit → verify → 关闭/重开 Store → Reader::ledger_at`。`CoreTransactionV2::canonical_bytes/from_canonical` 和 `transactions_at` 都保留原序；`ledger_at` 将其改成 `[a-second,z-first]`，仍返回 `Ok`。

**最小反例及实测：** [REPRO.md](REPRO.md) 的 `ledger_order_survives_normal_commit_and_reopen`。使用现有 W02 合成夹具和公开 API，在一笔事务的 `staged_ledger` 中放入上述两个不同倾向的条目；预期账本查询与规范事务逐项相等。Windows、Rust 1.99.0 下断言失败，日志摘要：

```text
canonical ledger order=["z-first", "a-second"]
ledger_at order=["a-second", "z-first"]
verify/reopen=OK
assertion failed: ledger query must retain canonical append order
```

**不修的后果：** 账本查询的“先发生什么、后发生什么”与已提交记录不一致；若消费者依序展示或回放变化，会得到错误次序。当前 `CoreTransactionV2` 原文仍完好，H 的事件级顺序未改变；没有证据表明当前生产重放已使用该接口并损坏状态。现有单条 L/每提交的测试及只比较条数的测试不能覆盖这一反例。

**建议：** 从规范事务的 `staged_ledger` 追加序构建查询结果，优先复用现有权威记录，不要求生成器人为设计可排序 ID。若选择保存派生 ordinal，则需相应迁移与恢复一致性检查；本审计不代选格式、不实施修复。

### F2 · P2：初始化崩溃恢复接受缺少 `reject` 目录的 format-2 存储，首次拒绝审计后无法再次打开

**位置：** [witness.rs:30–51](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/witness.rs#L30-L51) 依次创建 `d`、`commits`、`reject` 三个回执目录；[witness.rs:143–152](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/witness.rs#L143-L152) 无条件接受缺少 `reject` 的情况，没有区分兼容旧格式与新格式初始化未完成。[terminal_ingress.rs:113–127](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/terminal_ingress.rs#L113-L127) 先提交拒绝审计 SQL，再写回执。

**合法运行状态能否触发：能。** 正常创建 format-2 存储，在第二次 `ReceiptKindSynced` 后进程退出，此时 SQLite 已建立，`d` 和 `commits` 已建立，`reject` 尚未建立。无须删除文件、修改数据库或伪造输入；这是现有初始化故障注入点能直接达到的状态。

**触发路径：** 初始化进程崩溃 → `open_temporary` 成功且 `verify` 成功 → 正常提交一笔 H → 合法调用 `audit_ingress_rejection(AUTH_FAILED, None, budget)`。拒绝审计先留下一个 SQL 行，再因回执目录不存在返回 `NotFound`。此时 `verify` 仍成功；关闭后再打开，恢复尝试补写该回执，又因目录不存在失败。

**最小反例及实测：** [REPRO.md](REPRO.md) 的 `initialization_crash_cannot_admit_store_with_missing_reject_namespace`，复用原有受单次许可保护的崩溃 worker，在真实子进程中退出（86）。Windows、Rust 1.99.0 下得到 `first open/verify=OK; H commit=OK; audit=Err(NotFound); durable rejection rows=1; final open=Err(NotFound)`，最终“已准入存储仍可恢复”的断言失败。原有 [初始化测试 crash.rs:267–284](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/tests/crash.rs#L267-L284) 只枚举两次目录创建，将第二次视为完整，只验证空 head，没有继续进行拒绝审计，所以原有测试全绿不能排除此问题。

**不修的后果：** 恢复把初始化不完整的存储静默准入，允许随后建立正常历史；首次拒绝审计才暴露 I/O 错误，之后普通 writer 恢复入口不能打开该存储。已提交 H 的字节仍在，并未证明永久丢失；这是恢复可用性缺陷及过早成功准入，不是拒绝审计调用悄悄返回成功，也不是 B1 那样需要额外逻辑损坏的情形。

**建议：** 格式相关的回执命名空间完整性应成为恢复准入条件；对可证明安全的初始化残态显式修复并同步，或拒绝准入并给出明确恢复路径。补充初始化每个目录边界及“恢复后首次拒绝审计再重开”的验证，保留 format-1 兼容要求。本审计不实施修复。

## 边界加固（不计入正常路径问题）

### B1：`object_refs` 缺行未被恢复拒绝，随后 GC 可把仍被权威历史引用的对象删掉

**位置：** [recovery.rs:36–38](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/recovery.rs#L36-L38) 检查“表中现有引用”的闭包，却未证明所有权威闭包成员都在引用表里。[store.rs:194–219](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/store.rs#L194-L219) 直接把该表的 digest 集合当作 GC 保留集合，不从权威根展开闭包。

**合法运行状态能否触发：未发现合法 API 路径；正常发布不可达此前提。** `publish_refs`、可见性索引与提交原子发布，且表有 append-only 触发器。本反例必须显式移除触发器并删除索引行，代表额外的逻辑损坏；不能把它当成 SQLite 正常事务中途崩溃会产生的状态。

**最小反例：** 正常提交一个合成事件并关闭；保留全部对象文件、D、commits、facts、账本、回执，只对该事件 payload digest 删除 `visible_objects` 和 `object_refs` 两行（先删除对应 DELETE 触发器）。`integrity_check` 仍为 `ok`、`foreign_key_check` 为空。重新打开并 `verify`，然后 `collect_garbage`。完整诊断见 [REPRO.md](REPRO.md) 的 `boundary_missing_reference_row_allows_gc_to_remove_committed_payload`。

**触发路径与后果：** 缺行时对象仍在磁盘上，所以从其他引用展开的闭包仍能读取；恢复检查通过。GC 随后信任不完整的引用表，删除仍被 H 使用的 payload；之后才因对象缺失报错。若无外部备份，原本可从完整对象修复的索引损坏会变成历史内容丢失。它与 FIX-CORE-01 的证据查询索引不同，后者不参与 GC 保留集。

**建议：** 恢复和 GC 边界核对从 D/已提交规范对象展开的闭包与保留索引的包含关系；不一致时先拒绝清理。无需新增第二份权威历史。

此外，当前恢复没有逐项对照 `snapshot_objects`、`accepted_results`、`runtime_records`、`outbox`、`fact_corrections`、`dependency_index` 与规范提交；这属于同类恢复完整性边界。没有发现正常事务遗漏这些表的可达反例，也未证实全部表都存在当前生产消费者，所以不拆分凑成多条实际缺陷。

## FIX-CORE-01 已知问题的影响范围（不重复报告）

- **写入端：** [publication.rs:69–91](https://github.com/nanake-lab/nemu/blob/ce115abe6ced3ec8cc301bea2a290f47ace4807a/core/crates/core-store/src/publication.rs#L69-L91) 在同一提交事务内，仅从本次 `accepted_results` 的 `EvidenceGroupV3` 写入组版本和分配索引。正常成功提交会同时写这两类索引；没有找到正常崩溃只留下其中一半的路径。
- **检查入口：** `Store::open/verify`、`Reader::open` 和 `transactions_at` 都调用当前 `recovery::verify`，都继承该缺口。独立回执只见证 D/commit 引用，不能证明 SQL 派生组/分配行完整。`evidence_for` 依赖 `evidence_groups` 确定最新版本，再经 `evidence_index` 选择倾向。
- **可见错误：** 缺少分配行可把“索引不完整”当作“没有证据”；缺少最新组版本可让已撤回的旧分配重新出现；多余/错绑分配可造成错误的倾向关联。`evidence_index.reference` 本身未被该 SELECT 使用，实际返回的是最新组表的 reference，修复应检查双方与规范 accepted group，而不只检查对象能否解码。
- **最小范围复现：** 两次正常提交同一组：第一次分给 `tendency`，第二次 allocations 为空。完整库在第二提交查询返回空；仅人为删去第二提交的组索引后，恢复仍通过，查询重新返回第一版，而 `transactions_at` 仍有两笔完整权威事务。对应诊断名为 `known_fix_core_01_missing_latest_group_resurrects_old_association`。
- **后果边界：** 该缺口本身不改写 H、L、对象字节或提交顺序，但可污染把查询结果当成完整证据的后续计算。历史兼容还需覆盖已有 V2 组读取和 V3 组写入。此次未验证 FIX-CORE-01 的修复实现，也未把已知问题改记为 AUDIT-STORE-01 的新问题。

## 审计覆盖与结论边界

| 领域 | 核对路径 | 本次结论 |
| --- | --- | --- |
| 事务与重复/顺序 | ingest、settle_duplicate、bind/stage/commit、schema；D→Γ→H 与锚点 | 未发现合法输入导致半提交、同源重复进入 H、越过 D head 或回拨锚点的新反例 |
| 崩溃与回执 | 对象写入/复用、SQL commit、receipt capture/check/reconcile | SQL 落后于既存回执会拒绝；正常命名空间下 SQL 超前可修复回执；初始化命名空间缺失导致的恢复失败见 F2 |
| 发布与 GC | prepare 的闭包 pin、引用发布、visible_objects、GC | 正常保留协议未找到丢失反例；额外索引损坏见 B1 |
| 查询/恢复一致性 | current、facts、transactions_at、ledger_at、evidence_for、recovery | F1；已知 FIX-CORE-01 范围；其他损坏边界见 B1 |
| 并发 | Store 持有 OS writer lock，能力绑定同一次 opening；Reader 回执先于 SQL 快照 | 未发现第二写者或正常并发读产生混合提交的新反例 |
| 时钟与身份 | i64/u64 检查、绑定锚点/处理时间、device+stable key、header/location/lineage | 未发现正常条件下的静默截断、跨谱系读取或重复推进时间的新反例；真实时钟捕获属运行时职责 |
| 格式迁移与历史类型 | physical 1↔2、legacy R/θ 只读、旧库回退见证 | 未发现新反例；新规律状态生产接线不是本次审计交付范围 |

相关仍有效的原文：v2.0 §§2、13、15、17.1；F §§4–5；D 第一篇 §3；O §16；S §§7–9、30；ADR 0005、0010。主要涉及 I01（历史不覆盖）、I02（唯一原子发布）、I04（幂等与时间）、I14（绑定）、I16（重放）、I18（无隐藏权威）、I29（谱系隔离）。已有实现/测试映射见 [P5-02 对照表](../../docs/traceability/P5-02.md)；本审计没有修改规格读法。

## 验证

环境：Windows，本机锁定 Rust `1.99.0`；使用缓存依赖离线运行，重负载串行。全部生产代码来自上述 main 基线。

| 命令或检查 | 结果 | 证据 |
| --- | --- | --- |
| `cargo +1.99.0 test -p core-store --locked --offline -- --test-threads=1`（`core/` 下） | 51 单元测试通过，0 失败；3 文档测试通过；exit 0；单元测试 760.23 s | 本地 `D:/Dev/cache/AUDIT-STORE-01/core-store-tests.log` |
| 仓库外四项独立诊断，完整命令见 REPRO | 2 个不变量断言失败（F1、F2），2 个范围诊断通过（B1、已知问题）；exit 101；23.09 s | 本地 `D:/Dev/cache/AUDIT-STORE-01/diagnostics.log`；可复建源码见 [REPRO.md](REPRO.md) |
| F1 首次独立运行 | 相同顺序反例，exit 101；原始失败保留，未以重跑结果替换 | 本地 `D:/Dev/cache/AUDIT-STORE-01/ledger-order.log` |
| 文档差异、相对链接与 PowerShell 复现块语法 | 通过；提交范围仅本目录的两个 Markdown 文件 | `git diff --check`、链接解析、PowerShell AST 解析 |

两个失败是本报告捕获的产品反例，不是已修复或验收通过的声明。B1 诊断实测 GC 删除 2 个对象（含已提交 payload），随后 `verify` 报错；已知问题诊断实测旧关联重新出现而两笔权威事务不变。原始日志保留在本机，不把完整测试日志提交到主仓库。未新增运行时依赖；探针只使用仓库已有版本的 `core-store`、`core-types`、`serde_json` 和 `rusqlite`。

## 限制与下一步

- 诊断只使用 `TemporaryRoot` 新建的合成存储；没有打开用户的排练存储或正式存储。结构夹具用于检查存储顺序与损坏检测，不证明规律计算或行为有效。
- 未运行整工作区测试、模型/GPU 实验或物理断电测试。已有进程退出注入与本次逻辑损坏探针不等于磁盘/控制器故障认证；持久性仍依赖操作系统、NTFS、设备正确兑现同步。
- 本次没有修复 F1/F2/B1，也未宣称审计能证明不存在其他问题。F1 可从完好的规范事务恢复正确顺序；F2 未删除历史字节；没有将二者夸大为 H 已永久损坏。
- 下一步仅为独立审查本报告；修复范围及边界加固优先级由用户安排。本任务不新建修复卡、不自动合并。
