# P5-36 阻塞（2026-10-27）

## 2026-10-30：合并后本地验证完成，保留后续交付门槛

B=`f24f1256259c998bc213a9dc93103fcfb9cfe89a`，M0=`02948dc11aa1b43dc0dfe67c9da2e61676427385`，最终 M=`fb14feff3c35cf64ae661b642c3455ed506394e2`；已 merge，唯一追加冲突逐字节保留双方。gateway 直接受影响夹具已修复，完整 Rust 工作区、静态检查、独立 evidence 模块、三语言契约及 65 前缀真实存储恢复/续算全部通过；M0..M 仅文档/规则变化，源码相同，见 [本轮结果](main-sync-20261030/results.json)。

按用户本轮指示保持草稿，不等待或报告 CI。后续顺序：用户确认 runner 在线 → 转为可审查触发 Windows W02 → W02 通过后由独立审查方增量复审。若 runner 暂不可用，则继续保留草稿；不跳过门禁或用本地结果代替。此处没有重新判定 runner 离线，也不把旧 head 的排队/通过状态当作当前结果。

以下为先前阶段历史记录，各判断只适用于注明的阶段与提交；当前状态以上述段落和 REPORT.md 为准。

## 2026-10-28 首次审查修复状态

[首次审查](https://github.com/nanake-lab/nemu/pull/108#issuecomment-6461866895) 的固定 Limits 代码阻塞已修复；新增三语言预算回归、生产桥接/存储边界、65 前缀实际存储恢复与完整 Rust 工作区通过。写锁错误现保留原种类/errno，Store/维护持有者显式解锁；Linux 生命周期和有界并发检查随新提交 CI 验证。历史失败当时的持锁进程未被捕获，不声明其唯一根因已确认。当前提交的 CI 与独立复审仍是发布门槛，具体结论见 REPORT.md 和 PR 最新复审请求；不自行合并。

以下均为先前阶段的状态和原始失败记录。

## 2026-10-28 当前更新：B3 已获批准并完成适配

用户已批准必要 core-store 消费者与直接受影响夹具范围，目录占用核对无冲突后已实施。新写入/暂存使用 ThetaSnapshotV2 并接完整语义校验；旧原字节可读，旧头在可写连接打开前明确拒绝，不补零、不自动迁移。65 真实 C 前缀存储提交/关闭重开/读取后的独立逐字段位比较与全部后缀续算通过；完整 Rust 工作区、最终 fmt/clippy 通过，store 49/49、fact-view 18/18。证据在 `storage/`。B3 已解除，没有待实现的本卡阻塞；发布仍要求当前提交必要 CI 全绿及独立首次完整审查，最终状态/CI 入口见 REPORT.md。

以下 B3/B1/B2 段落与失败数据为历史记录，不再表示目录占用或授权缺失。旧历史迁移、全 H 重算和完整运行时编排仍未完成，属于用户明确排除的后继范围。

## 2026-10-28 实施后的当前阻塞 B3

**旧 B1/B2 已解除。** #95、#65、#110 已合并；本分支 merge 了 main `f24f1256259c998bc213a9dc93103fcfb9cfe89a`。用户已批准 `dynamics/storage.rs`、`dynamics/storage/tests.rs`、`dynamics/mod.rs` 三路径的最小扩围，并选定后继 θ 快照内嵌完整估计器状态；桥接和真实逐位恢复/续算已实施。以下旧日期章节仅保留历史。

**B3：任务卡要求标记旧消息被取代，而当前 core-store 的写入/事务读取仍绑定旧 ThetaSnapshot；本轮明确禁止扩到运行时接线。**

- 实测：`cargo test --manifest-path core/Cargo.toml --workspace --locked` exit 101；core-store 15 passed、31 failed，原始日志 `implementation/rust-workspace-01.log`。共同错误为 `Rejected("new writes must use the active successor message")`。这不是 main 原有失败或环境故障，而是本卡新增 superseded 标记与现有存储消费者的冲突。
- `core/crates/core-store/src/objects.rs:80` 检查 `!T::SUPERSEDED`；旧 `ThetaSnapshot` 保留字段/字节但按本卡要求被 `ThetaSnapshotV2` 取代，因而不能再用于新写入。
- `core/crates/core-store/src/tests/mod.rs:178` 仍 prepare 旧类型；生产代码 `transaction.rs:147`、`:326` 仍用旧类型读取 staged/current θ。只换测试夹具不足以解决生产接线问题。
- 继续执行其余 crate 后，fact-view 为 1 passed、17 failed，均在 `core/crates/fact-view/src/tests/fixture.rs:163` prepare 同一旧 θ 类型时失败，见 `implementation/rust-other-crates.log`。core-types 的黄金探针需接入新增显式语义边界、取代登记数量需更新；这些本卡范围内问题已修复，并以 `core-types-acceptance.log` 和 `clippy-acceptance.log` 复核通过，不属于残余阻塞。
- 已完成的恢复桥接不写存储，不能绕开以上入口。未删除 superseded 标记、未允许旧类型继续新写、未跳过失败测试、未将其误报为与本卡无关。其余 crate 的检查另行继续，不替代失败的完整工作区验收。

处理选项：

1. 后继存储工作完成版本化 θ 消费者接线与必要的旧读取边界后，再合入本分支并重跑完整验收；本卡保持 draft/needs-user。任何既存数据迁移仍须单独明确范围，不能默认补零。
2. 用户另行批准具体 core-store 接线范围后完成这项协调。本轮“不得扩到迁移和运行时接线”的边界继续有效，未擅自实施。

暂不把“仅文字标记取代、继续放行旧写入”列为实现修复，因为它改变既有活跃消息门禁，需要额外契约决策，不能为了全绿隐式采用。全部验收尚未通过，因此不转可审查、不评论“请审查”。继续完成其余静态检查、三语言向量、证据和报告。

## 2026-10-28 当前状态更新

旧 B1 中 #95、#65 的占用已解除：合并提交分别为 `0416e934`、`88455673`，均在最新 main `32a5888e` 内。当前待释放的是 #110（head `4c8dd453`，开放、非草稿）：增量复审及用户契约批准已完成，但尚未合并，仍占用 contracts/types/generated；不得沿用旧报告中的“等待 #95/#65”。

B2 仍成立，且最小范围已收窄为新增 `dynamics/storage.rs`、`dynamics/storage/tests.rs` 与 `dynamics/mod.rs` 的注册/重导出。现有 `pub(super)` 足够，无需放开字段或改算法。具体输入/输出、失败边界、逐字段位比较和恢复后续算方案见 [接口请求的 2026-10-28 追加](../../tasks/requests/P5-36-dynamics-roundtrip.md#2026-10-28-恢复前核对供主会话裁定的最小范围)，交主会话裁定。此次未修改实现或重跑测试；以下保留原核对历史，不作当前占用结论。

## B1：契约目录仍被未合并 PR 占用

已 fetch origin，并读取最新 PR 元数据：P5-34 / #95 为 OPEN、非草稿、未合并，head `d047b417cb0bff94f053e6a4b023fff495d292d9`；占用 `core/contracts/`、`core/crates/core-types/` 和生成目录。P5-04 / #65 同样 OPEN、未合并，head `042fba778822a669521dc59bab6e9b5d75360f57`，包含 `core-types` 改动。按 AGENTS §2，P5-36 不能同时修改这些目录。P6-17 / #97 也在排队，当前仅报告，不把它误报为已经实施。

已做：两次 fetch 主线均落实到 `3f8f98e5`，草稿 #108 认领；已完成必读、ADR 提案、真实字段核对、恢复接口请求，以及现有生成/全部兼容基线/三语言契约检查。PR 状态与占用路径见 `verification/pr-summary.json`、`pr-open-pr-changed-filenames.json`。#95/#65 在发布准备时再次读取仍为上述 head，均未合并。

选项：

1. 相关 PR 合并并释放目录后，merge 最新主线，再继续本卡（遵守串行规则，避免重复分配契约编号）。
2. 用户明确调整相关卡的占用顺序或批准具体互斥例外后继续（需协调尚未合并的契约和类型改动；本次未获此例外）。

新消息、语义检查、黄金向量、新基线及新消息往返验收均未实施，不能用旧基线检查代替。部分完成时保持草稿与 `needs-user`，不发布“请审查”的完成请求。

## B2：真实内存往返需要卡外桥接

问题：P5-25 的 `DynamicsState`（`types.rs:109`）和 `RunningTime`（`time.rs:14`）将递推字段保留为 `pub(super)`；现有 getter 不覆盖和、方差、对数统计、检测、开放段、事件/运行时间和慢漂全部状态，也没有完整恢复入口。P5-36 目录不包含 core-kernel。

已核对：实际类型、公开方法、genesis/更新/时长/检测/时间实现及 P5-25 的完整 METHODS 和请求；未用手造 lookalike 或 Debug 解析冒充真实往返。接口请求已追加至 `tasks/requests/P5-36-dynamics-roundtrip.md`。

选项：

1. core-kernel 所属工作提供基于正式生成类型的导出/验证恢复桥接，P5-36 复用。
2. 明确批准请求列出的最小 dynamics 模块路径纳入本卡，遵守目录互斥后实施，并补跑相关测试。

目录释放不会自动消除这个接口范围问题。真实 round trip 与恢复后续算均保持 `not_run`。

## 待完成门槛

ADR 0029 当前只是提案。后继消息、逐消息正反向量、正式摘要基线、三语言新语义和真实状态往返完成后，还须独立交叉审查及用户批准契约。没有放宽这些验收。无阻塞的阅读、设计提案、编号调查及现有基线核验已完成并入报告。
