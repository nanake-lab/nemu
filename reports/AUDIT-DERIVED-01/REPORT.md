档位：轻档；本 PR 只追加任务请求、只读审计报告和派生数据清单，不修改产品代码、配置、契约或既有测试。

# AUDIT-DERIVED-01 — core-store 以外的派生数据正确性审计（交付线）

本次没有确认正常合法运行可触发的新增 H 派生数据正确性缺陷。另有两项 H 快照边界加固：R 的局部等式检查不能证明它等于证据重放结果，θ 的恢复检查也不能证明快照值等于历史计算结果；两项都需要先人为提供不一致的快照，不能写成已发生的运行错误。外层还发现一项非 H 的口型资源缓存切换问题，合法库调用可达，但当前产品未接线，单列为加固项。事实检索有前缀、资源版本和返回原文核对，全部派生对象的来源、消费者、核验与重建条件见完整清单。真实 H 重算、独立第二实现、检查点恢复和运行时发布仍有已登记的接线缺口，PR #194 的存储投影核验没有填补这些缺口。

## 状态、基线与交付范围

- 状态：只读审计完成，等待独立审查；按用户要求写「请审查」后停止，不合并、不续卡。
- 源码基线：开工时最新 `origin/main`，`d1213c8666efd504b6f6cd0488716419f4077406`。收尾最新 main 为 `8ea2966fe60f521104c3d20f74f90bab29defa60`；两者之间仅 #183 的槽生命周期设计和 #167 的研究报告，所有本次审计源码、AGENTS.md 与工作规则均无变化。源码链接固定到前者，当前源码结论同样适用于后者，不外推到其他实现版本。
- 前置材料：`AGENTS.md`、仓库 `echo-core-work` 工作规则、[AUDIT-STORE-01](../AUDIT-STORE-01/REPORT.md)、[PR #193](https://github.com/nanake-lab/nemu/pull/193)、[PR #194 FIX-STORE-01](https://github.com/nanake-lab/nemu/pull/194) 的实现、修订及最终审查。#194 的合并提交 `2493b6b6df109207789f0b11221b8d9a4b8e9cf6` 已在本基线中。
- 范围：全部 11 个非 `core-store` crate；`apps/`、`packages/`、`workers/`、`executors/`、`services/` 中运行源码及相关工具入口。研究工具的缓存另外标明；不把测试夹具、接口声明或未装配的纯函数称作正在运行的生产服务。
- 交付文件：本报告、[INVENTORY.md](INVENTORY.md)、[原始任务请求](../../tasks/requests/AUDIT-DERIVED-01.md)。未新增实现、测试、依赖、ADR、迁移方案或任务卡。

## 判断口径

本次沿“来源 → 派生过程 → 消费者 → 一致性检查 → 重建”追踪。H 指已接受、已提交的权威历史；从 H 重建状态还需要对应的 genesis、规律、解释记录、来源治理和版本化资源，不能只凭事件文本重新问一次模型。v2.0 §2.2 与 I23 的要求是完整历史重算与增量结果逐位一致，摘要相等、结构合法或浮点范围合法只证明各自覆盖的条件。[规格依据](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/docs/echo-core/echo-core-v2.0-first-principles.md#L135-L157)；[I23](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/docs/echo-core/echo-core-v2.0-first-principles.md#L520-L544)。

“正常可达”包括实现允许的合法输入、重试、崩溃和恢复，不包括手改内存快照、绕过权威引用构造另一个对象，或先破坏存储。公开函数允许传入一个类型正确的值，不自动证明该值能由正常历史产生。反过来，未接线也不代表无需检查接口：本报告保留可复现的边界反例，并明确它们尚未形成正常运行路径。

设备采样、音频帧、UI 镜像、模型资源缓存和 RPC 等待队列也纳入清单，但它们的直接来源不是 H。对这类对象分别回答能否从 H 恢复；不把“重建对象后等待下一次采样”写成“已经实现 H 重放”。同一生命周期中的字段和辅助表合并成一项，避免把每个局部变量拆成问题。

## 正常路径发现

**本次关注的 H 派生数据问题：没有。** 在本基线的源码及仓内调用链中，没有建立起“正常生产者生成不一致 H 派生物 → 消费者未发现 → 产生错误权威结果”的新增反例。这里的“没有”是本次检查结果，不是不存在其他缺陷的证明；尤其不能据此宣称尚未接通的生产链已经完成验收。非 H 的资源缓存 B3 可通过合法库 API 演示，但没有当前产品接线，不混入此计数。

几个容易被误写成缺陷的边界已核对：

| 对象 | 核对结果 | 结论边界 |
| --- | --- | --- |
| FactView 的事实表、全文表和向量表 | `rebuild` 从指定前缀建立全新私有内存数据库；查询前缀必须相同，非 Audit 查询还要求当前提交未前进；返回项重新读取事件和内容核对。[构建](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/fact-view/src/index.rs#L30-L115)、[查询](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/fact-view/src/retrieval.rs#L73-L153)、[结果核验](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/fact-view/src/retrieval.rs#L324-L338) | 未发现正常更新造成磁盘旧索引被当成当前结果的路径；Audit 读固定旧前缀是明确允许的行为。此处不独立重算每个排序分数，也没有生产装配调用。 |
| runtime 的暂存状态与 current | `prepare_existing_head` 再读 current，对基态 x/R/θ 做规范字节相等比较；当前适配仅允许无成长 R/θ，且只 prepare/stage，不自行 commit。[代码](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-runtime/src/store.rs#L54-L146) | 同 revision 不代表同内容；已有检查没有仅凭 revision 信任基态。仍不证明 current 中的数学状态是 H 的正确重算值。 |
| gateway 暂存的 `published_commit` | 从 handler 前后实际 Store head 的变化取得，随后与 outcome、规范事务及事件的 ID、位置、谱系和摘要交叉核对。[代码](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/gateway/src/progress.rs#L34-L107) | 并非只相信 handler 自称提交成功；未找到合法路径把另一次提交的通知配到该请求。 |
| carrier KV | runtime 更新接口先验证 envelope，再依次 clear → install → prefill，任一步失败即返回；研究工具按完整 key 清理，手工研究聊天每次生成从空 KV 预填充。[runtime](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-runtime/src/store.rs#L221-L237)、[研究缓存](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/workers/model-adapter/probe/injection.py#L22-L58) | runtime 只有接口和测试实现，不能据此宣称真实载体缓存已接通；研究 KV 也不当成正式历史。 |

## 加固项（不计入正常路径问题）

### B1：R 快照与证据分配可分别合法，但局部检查不能证明两者计算一致

**来源与消费者。** G 接收 `RState { snapshot, groups }`，其中 `RSnapshotV3` 的 A、V、B、π、M、U 等字段和 `EvidenceGroupV3`/allocation 都是历史及其解释的派生结果。`law::g` 用现有 snapshot 初始化工作表，后续成长会沿这些值继续；R 的 π 和幅值也供 R⁺、F 和 J 的数值通路使用。[输入](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/input.rs#L99-L105)、[G 入口](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/engine.rs#L57-L85)、[工作表](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/engine.rs#L208-L246)。

**现有检查与缺口。** `validate` 核对 law 引用、规范次序、组和倾向唯一性、allocation 范围及分配和。`check_stored` 核对 `V ≥ |A|`、`B = c + V`、由计数计算的 ĥ 以及由 P/A/c/V 计算的 π；`check_with_allocations` 再核对组数、连续 ordinal、历史 q、M/U 与典型幅值。但这个入口没有调用 `replay`，所以没有逐位比较“现有汇总字段”与“从这些 allocation 重新折叠所得字段”。[validate](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/validation.rs#L9-L90)、[局部等式](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/recurrence.rs#L106-L132)、[重放及检查](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/replay.rs#L148-L201)。

**正常运行能否出现：未找到正常生成路径。** G 的正常输出经过自身校验，既有历史测试会比较增量、allocation 重放和完整事件重算。此次反例必须在正常输出之外修改 snapshot，或者由未来接线错误地把别的汇总值装配进来；未发现仓内产品入口实际执行这种装配。因此本项不是“正常 G 会算错”，也不是 PR #194 未修复的 SQL 投影问题。[现有逐位比较](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/law/tests/replay_h.rs#L20-L94)。

**出现路径。** 合法 H/fixture → 正常 G 输出 R 和 groups → 在权威生成流程外改 R 的 A，并同步调整 π 使局部等式仍成立 → 将改后的 R 与原 groups 交给当前校验入口 → 局部检查无法发现 A 已偏离原证据折叠结果。这里不假设能够修改哈希寻址对象的字节而继续通过旧引用读取；提供另一个语义合法的对象，与通过原权威引用读取原对象，是两个不同前提。

**可复现的单元测试思路（`not_run`）。** 在 `law` 的 crate 内单测复用 `law/tests/replay_h.rs` 的 `Fixture`、`World`、`stored_of` 和 `replay`：

1. 用一个非零正支持的合法组生成 `group_count > 0` 且 `evidence_a != 0` 的倾向，保留原 R、groups 和完整事件序列；先断言原值与重放逐位相等。
2. 克隆 R，把该倾向 `evidence_a` 设为 `0.0`；用现有 `recurrence::pi(prior(t), 0.0, law.base(predicate.len()), evidence_absolute)` 更新 π。其余 V、B、计数、ĥ、M/U、方向、组及分配均不变。
3. 调用同一 `validation::validate`，诊断预期为 `Ok`；再对未变的 groups 调用 `replay`，诊断预期为重算 A/π 与被改快照不相等。这只检验检查边界，不需要伪造 Store、磁盘损坏或生产 H。
4. 若以后在恢复/迁移/发布边界落实 H 核验，对同一候选应取得明确的“不一致”结果；不能通过改掉旧证据来让它相等。这个后继验收断言本次未实现。

**不处理的后果。** 如果未来导入或恢复路径把上述局部检查当成 H 认证，偏离历史的 π/幅值可能影响 R⁺ 选择、F/J 读出及后续成长。当前未证明发生生产状态污染，更未证明 H 被删除或覆盖。

**能否重建及加固位置。** 现有 `replay` 可从可信的倾向起点、law/caretaker 和 allocation 重建单倾向；要同时证明 groups 也正确，仍须从权威 H、genesis、解释及来源治理重建完整输入。应在接受外部快照、迁移或权威发布的边界完成这种证明；没有理由要求每次纯数值调用都重新遍历整个 H。本报告不指定新的协议或修复实现。

### B2：θ 恢复验证了结构和资源绑定，未证明 values/估计器等于历史递推结果

**来源与消费者。** `DynamicsState` 保存 values 及所有未来相关的统计量，V2/V3 snapshot 从这些字段导出；恢复结果用于后续 C 递推，values 还用于 F/J。恢复时验证语义、预期 law 引用与按调用方 law/fuses 重建的完整参数字节，然后把 snapshot 的 values/estimator 复制进状态。[导出](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/dynamics/storage.rs#L72-L138)、[V2 恢复](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/dynamics/storage.rs#L147-L204)、[V3 恢复](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/dynamics/common_discount.rs#L99-L124)。

**现有检查与缺口。** 检查包含维数、计数、时间恒等式、估计量正值、genesis 的空统计、参数摘要、h 初值/下界及 gain/recovery 范围。这些很有必要，但未比较 `values.baseline` 与事件重算值；即使 `event_count == 0`，也没有要求 baseline 等于 law 的 baseline prior。恢复函数没有 H 参数，源码也明确把资源/H 前缀认证留给可信调用者。[语义检查](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-types/src/semantic/theta.rs#L111-L218)、[绑定与范围](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-types/src/semantic/theta.rs#L220-L300)、[genesis 实际赋值](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/dynamics/types.rs#L142-L177)。

**正常运行能否出现：未找到正常生成路径。** `genesis` 用 law prior 初始化，`export_snapshot` 导出实际状态字段，正常路径没有步骤会单独改出本反例。必须手改 snapshot，或由未来导入层提供与 H 不一致的值。仓内搜索的调用是测试/研究驱动，没有发现已接线的生产恢复循环。本项不把“值域内可构造”误写成“正常导出会产生”。

**出现路径。** 正常 genesis → export V2 snapshot → 在流程外改一个合法范围内的 baseline 分量，保持参数、参数摘要、law 引用和空估计器不变 → restore 接受完整资源匹配的快照 → 得到与相同 law 的 genesis 不同的状态。这不需要假定摘要碰撞，也不声称旧权威对象引用会接受被改过的字节。

**可复现的单元测试思路（`not_run`）。** 在 `dynamics/storage/tests.rs` 复用已有 `law()`、`fuses()`、`reference()`、`LIMITS`：

1. `DynamicsState::genesis(law())` 并通过 `export_snapshot` 导出。夹具的 `baseline_prior[0]` 是 `0.1`，`event_count` 为零。
2. 克隆 snapshot，只把 `values.baseline.connection` 改为 `0.2`，不改任何 estimator、parameters 或引用。
3. 用原 `reference()`、`law()`、`fuses()` 调 `restore_snapshot`。按现有检查，诊断预期为 `Ok`；对比恢复值与全新 genesis 的该分量 bits，应不同。资源摘要校验仍通过，因为被改字段不是 parameters 的一部分。
4. 可在 V3 的 common-discount 夹具做同型诊断；后继边界验收需要拒绝/报告该不一致，再以相同完整 H 的逐位重算作比较。只补 genesis 特判不足以证明一般历史正确。

**不处理的后果。** 如果后继恢复把“语义合法且 law 参数一致”当成“与 H 一致”，错误基线或统计量会被带入未来 C 和 F/J。当前没有正常运行实例，不能宣称已影响她的实际行为。

**能否重建及加固位置。** 本反例无需任何事件即可由 genesis 重建正确值；一般状态需要按绑定的完整历史、解释和资源逐步重算，并比较全部未来相关字段。已有导出/恢复接口可以承载完整状态，但不等于已经提供生产 H 验证器。完整核验应由已有后继接线承担，不在本次新增一份旁路权威状态。

### B3：口型分析器切换 profile 时仍复用旧参数的滤波器（非 H 派生；当前产品未接线）

**来源与消费者。** `LipSyncAnalyser` 构造时从 profile/options 的采样率、窗口长度和 `melFilterBankChannels` 预计算 Hann 窗和 Mel 滤波器。`setProfile` 只替换 profile 引用；`analyzeFrame` 继续使用旧窗口/滤波器，再与新 profile 的 MFCC 参考值比较。`LipSyncRuntime.switchLanguage` 及 `setCustomProfiles` 会走到这个 setter。[构造与 setter](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/packages/avatar-runtime/src/lipsync/analyser.ts#L18-L50)、[使用](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/packages/avatar-runtime/src/lipsync/analyser.ts#L55-L137)、[切换入口](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/packages/avatar-runtime/src/lipsync/runtime.ts#L68-L86)。

**正常运行能否出现。** 合法公开库 API 的调用序列可达，无须手改缓存或内存；但本仓非测试产品代码未实例化 `LipSyncRuntime`，内置 zh/en/ja profile 又都使用 16000 Hz、512 样本和 16 个滤波器，因此未确认当前正常桌面运行能触发参数失配。它不是 H/记忆损坏问题。将来接入具有不同分析参数的自定义 profile 时，此路径会直接相关。[profile 参数及选择](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/packages/avatar-runtime/src/lipsync/profiles.ts#L4-L60)、[公开 profile/options](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/packages/avatar-runtime/src/lipsync/types.ts#L60-L85)。

**出现路径。** 用 profile P1 构造分析器 → 经 setter/语言切换提供分析参数不同的 P2 → profile 已更新，派生数组没有失效或重建 → 下一个 PCM 帧按 P1 的参数提取特征，却按 P2 的参考数据评分。setter 没有拒绝不兼容参数，也没有核对缓存键。

**可复现的单元测试思路（`not_run`）。** 在现有 `lipsync.test.ts` 的分析器测试组，用合成 P1/P2 保持采样率 16000、窗口 512 不变，只把 Mel 通道从 16 改为 32；两者均为合法的 `LipSyncProfile` 结构。分别构造 `a = new LipSyncAnalyser(P1)` 后 `a.setProfile(P2)`，以及 `b = new LipSyncAnalyser(P2)`。输入同一段非静音、固定振幅和频率的 512 样本合成 PCM，比较 `analyzeFrame(...).mfcc`：按“当前 profile 相同应得到相同特征”的断言，静态预期不相等；断言无需访问私有数组，也不依赖扬声器、真实语音或 H。再保留内置三语切换的相同参数用例，区分参考值切换与分析参数切换。

**不处理的后果与重建。** 后继接线若允许这类资源切换，口型特征会混用不同 profile 参数，可能产生错误的元音/嘴型读出；不会由此改变权威历史。H 单独不能重建音频输入或滤波器资源；用当前 P2 和相同 options 新建分析器即可重建一致缓存。可选择切换时重建或明确拒绝不兼容分析参数，具体实现留给所属范围，本次不代选。

## 已登记的接线缺口（不重复编号为新缺陷）

| 边界 | 当前源码事实 | 对本次结论的影响与已有归属 |
| --- | --- | --- |
| replay、verify、checkpoint、restore 命令 | `core-replay` 仍为 scaffold；`echoctl` 明确打印上述能力未实现并退出 2。[core-replay](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-replay/src/lib.rs)、[echoctl](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/echoctl/src/main.rs) | 不能写“损坏可运行 echoctl 自动修复”，也不能把不存在的恢复命令算成正常触发路径。 |
| evidence / recompute | 两个完整模块均由 `#[cfg(test)]` 门控；重算机制接收显式 history、scope 和 Kernel 比较接口。[门控](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/lib.rs#L11-L15)、[比较](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/recompute/inspection.rs#L4-L42) | 具备测试中的强制重算/逐位比较机制，不等于生产的真实 H 认证或独立实现；已有 [P5-26 接线请求](../../tasks/requests/P5-26-integration.md)，后继 P5-54。 |
| runtime 事件到权威发布 | `store.rs` 明确没有生产 process-and-commit 循环；现有 prepare 不提供 L/U/Q 初始根。非 REHEARSAL 且 F/J 均存在的 `compile_committed` 因缺 Q 发布契约而返回错误；REHEARSAL 始终返回 Unmodulated。[边界](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-runtime/src/store.rs#L1-L4)、[调制准入](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-runtime/src/store.rs#L165-L191) | 未见产品装配者；不能把测试 adapter、结构绑定或 NumericReceipt 当成已提交 Q。见 [P5-08 权威发布请求](../../tasks/requests/P5-08-runtime-publication.md)，已有 P5-29 等接线归属。 |
| F V3 的依据身份和历史资格 | `HistoryBasis` 的首位置、有效/活跃/重解释视图由调用方解析；内部检查字段一致、去重及次序。[代码](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/core/crates/core-kernel/src/fast/contribution/event.rs#L34-L64) | 不能据内部排序声称已认证 H；源码指向 P5-75，不另计为一个可达运行问题。 |
| #194 后的 Store 验证与数学验证 | #194 从规范记录比较 SQL 投影并修正 GC/账本/恢复；它未提供独立 G/C 完整历史重算。[修复报告](../FIX-STORE-01/REPORT.md)、[审查边界](https://github.com/nanake-lab/nemu/pull/194#issuecomment-6878839396) | 存储派生一致性与数学状态一致性是不同检查。本次不重复报已修复的问题，也不重新审计 core-store。 |

收尾 main 新增的 [ADR 0041](https://github.com/nanake-lab/nemu/blob/8ea2966fe60f521104c3d20f74f90bab29defa60/docs/adr/0041-f-basis-slot-lifecycle.md) 已记录用户选择 L1（精确物化逐出/恢复）与 M1（有效来源选目标、保留全部残量），同时明确未实施/发布。不能把设计要求当作当前 F V3 已有的缓存恢复实现；本次没有重开用户选择，也不实施新表示。

这些缺口不是本报告的阻塞项：用户请求的是现状审计，未授权把后继实现补齐。它们也不是“已经验收通过”的项目。P5-54/P5-75 的登记可在基线 [cards.json](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/tasks/cards.json#L1902-L1922) 和 [P5-75 条目](https://github.com/nanake-lab/nemu/blob/d1213c8666efd504b6f6cd0488716419f4077406/tasks/cards.json#L2322-L2339) 核对。

## 验证与限制

环境：Linux 云端只读源码工作区。除本任务的三个 Markdown 文件外，未修改仓库文件；没有打开真实、排练或用户存储，没有启动桌面程序、模型或 GPU 工作。

| 检查 | 结果与边界 |
| --- | --- |
| `git fetch origin main`、祖先核对，以及 `git diff --exit-code d1213c8..8ea2966 -- AGENTS.md .agents/skills/echo-core-work/SKILL.md core apps packages workers executors services tools config profiles package.json pnpm-workspace.yaml` | 通过；#194 已合入，范围内源码和工作规则差异为空。已读新增 ADR 0041 的选定设计与 diff-register 追加内容、研究报告及完整变更路径清单；均未实施到本次源码。 |
| 源码、调用者及既有测试阅读 | 覆盖 [清单](INVENTORY.md) 的模块；使用 `rg --files`、`rg -n` 及逐段源码检查。引用测试表示读过其断言，不表示本次执行通过。 |
| B1/B2/B3 诊断 | `not_run`；本报告交付的是可复现单元测试思路和静态推导，不包含已执行的失败断言或测试日志。 |
| `git diff --cached --check`；新增 Markdown 相对链接存在性及源码行号检查 | 通过；最终交付仅三个 Markdown 文件。相对链接目标和固定提交源码链接均存在，行号不越界。 |
| 全工作区测试、Windows/IPC/故障注入、模型/语音/性能验证 | `not_run`；此次没有产品改动，也没有借用 #194 或其他旧报告的通过数作为本次结果。 |

局限：静态调用检索只覆盖本仓代码，不能排除外部调用方；第三方依赖内部缓存、系统驱动、资产/模型的真实行为及尚未实现的服务不在可验证范围内。完整清单说明的是本基线的语义对象和生命周期，不承诺枚举编译器、第三方库或操作系统的一切内部缓存。结构/绑定核验能发现的错误、独立重算才能发现的错误，在清单中分别说明。

## 审查请求

请重点核对：清单是否遗漏实际消费者；B1/B2 的前提和静态诊断是否成立；B3 的合法库调用与当前未接线状态是否区分准确；是否把尚未接线的 H 核验误写成已存在或正常可达。本次没有需要用户选择的设计方案，也不创建修复卡。下一步仅为独立审查；提交「请审查」后停止。
