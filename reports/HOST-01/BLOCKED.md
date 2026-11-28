# HOST-01 2/2 宿主接线阻塞

在 `858714b2339096cd9e400c74aa427444f92b6cf1` 核对了普通文字从 P6-02 到持久队列 D、再到运行时的调用路径。宿主可以把现有 `LocalAuthority` 原样交给 `RehearsalHandler`。阻塞发生在新排练库取得初始状态，以及为 D 队首准备提交输入这两步。

| 入口 | 主分支上可复核的证据 | 继续接线需要什么 |
| --- | --- | --- |
| 首次打开空库 | [宿主启动](../../core/crates/nemu-core/src/host.rs)调用 `Store::startup`，空头保留为等待初始根。[bootstrap_rehearsal](../../core/crates/core-runtime/src/store.rs)要求调用方提供 `VersionedEnvironmentV2` 和 `Snapshot`，按 [ADR 0049](../../docs/adr/0049-lineage-initial-publication.md) 绑定版本资源。[部署配置](../../profiles/deployment/profile.json)是空对象，其 [schema](../../profiles/deployment/profile.schema.json) 将它标为占位。 | 提供已确定来源及版本的排练 Ω、x/R/θ 和引用对象，以及从该资源准备首次发布的入口。已有库从已发布头恢复。 |
| 处理一条已持久的文字 | [RehearsalHandler::process_next](../../core/crates/core-runtime/src/handler.rs)读 D 队首、七根和 Q，随后调用外部 `inputs(store, event)`。同一处定义的 `EventInputs` 需要绑定、处理时间、事实投影和事件属性。[输入校验](../../core/crates/core-runtime/src/validation.rs)会核对提交前缀、摘要、时间和状态版本。 | 指定可在真实排练库中调用的可信输入提供方，能对同一 D 队首和已提交前缀产生这四项输入。宿主可从现有接线点传入该提供方、认证方和唯一提交能力。 |
| 强杀后继续处理 | [时钟模块](../../core/crates/core-runtime/src/clock.rs)明确说明尚无生产重启适配器。`Clock::restore` 要求上一份可信采样、逻辑时间、当前采样及其边界。[P5-53 交付记录](../../docs/traceability/P5-53.md)把时钟恢复列入后续部分。 | 重启后要继续处理待提交或重发的文字，需能从已核验的时间证据得到下一次 `ProcessTime`。已提交的 Q 可以直接读回。 |

[现有联测输入](../../core/crates/core-runtime/tests/handler/support.rs)的 `initial` 从测试夹具构造快照与资源。`inputs` 使用分支名 `test`、`100 + raw_log_position` 的合成时间、清空的事实项和夹具属性。[真实管道联测](../../core/crates/core-runtime/tests/handler/windows.rs)把这个函数传给处理器。这验证了在给定输入下的提交和恢复；将同一函数复制进宿主会把测试数据写进真实排练历史。

**核对方式。** 在主分支基线运行 `rg -n 'RehearsalHandler::new|bootstrap_rehearsal\(|Ok\(EventInputs \{' core/crates --glob '*.rs'`，处理器构造、首次发布调用及完整输入构造的命中均在测试中。逐一对照 [P6-02 文字封装](../../packages/terminal/src/observation.ts)、[现有准入谓词](../../core/crates/core-types/src/permission.rs)和[网关提交核验](../../core/crates/gateway/src/progress.rs)。普通文字的现有字段与准入条件相符，网关在独立读者核验发布后才发送 `CORE_COMMITTED`。本次提交仅登记阻塞。

**原生验收为 `not_run`。** 缺少上述真实输入，以下三项均未执行：真实 Windows 宿主与 P6-02 单条消息提交并推送状态；重启后状态一致；连续 20 条、回执前后各终止一次测试核心后的零丢失与同键去重。前序 PR 的持久收件和合成输入联测分别见 [#258](https://github.com/nanake-lab/nemu/pull/258)、[#268](https://github.com/nanake-lab/nemu/pull/268)、[#273](https://github.com/nanake-lab/nemu/pull/273)。

**解除条件。** 若版本化排练资源和可信输入入口已经交付，请指明资源版本与调用位置；若尚未交付，由相应资源、事实与时钟负责人提供后再接宿主。#277 涉及的生产签发器与真实 H 投影仍由该 PR 负责。
