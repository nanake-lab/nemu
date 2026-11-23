风险分级：重档；本 PR 包含 core-runtime 状态编排与存储边界，本轮仅调整条件性交付说明并合入 main。

# P5-08 编排、阶段接口与时钟（W03）— 条件性交付

## 当前状态

**条件性交付（部分交付）。P5-08 整卡未完成，禁止据此接入生产。** 按用户 2026-11-02 指示，参照 [#121 条件性交付先例](https://github.com/nanake-lab/nemu/pull/121) 将本次限定范围提交为可审查并标 `needs-user`，由用户审查并决定是否接受。转可审查或接受本次交付均不代表整卡完成、后继验收通过、生产接入许可或依赖已满足。本会话不合并、不领取下一卡。

[R3 复审已通过并关闭](https://github.com/nanake-lab/nemu/pull/120#issuecomment-6560079382)，结论绑定 `063894aae52799f6083cb1a6dee2747e724ec6f1`，仅覆盖已实现的部分。当前 runtime 源码/测试与该审查 head 逐文件相同。后继项见 [BLOCKED.md](BLOCKED.md)、[权威发布请求](../../tasks/requests/P5-08-runtime-publication.md)、[D11 时间语义请求](../../tasks/requests/P5-08-result-time-semantics.md) 与 [阶段输入及 law 请求](../../tasks/requests/P5-08-stage-inputs-and-law.md)。

## 本次交付范围

| 模块 | 条件性交付的能力 | 证据边界 |
| --- | --- | --- |
| `stages.rs`、`pipeline.rs`、`binding.rs`、`validation.rs` | 受限阶段接口、至 `StagedEvent<L>` 的顺序/绑定校验、谱系能力、降级与 STAGED 隔离；R⁺ 保留典型幅度，θ 使用完整生成 V2 并校验语义 | 同步可信适配器与合成输入上的结构机制；不是完整 Commit/J 服务或真实 F/C/J 接线 |
| `clock.rs`、维护入口 | 显式时钟输入的计算与故障检查、单次 process 取样、独立 completion 输入、只读 x 投影及 R/θ 保留 | 没有 OS provider、原子时钟 checkpoint、初始 L/Q/U 或真实跨进程重启验收 |
| `store.rs` | 使用现有 W02 API 读取已有 V2 head，准备无成长事务；提交/重开保留完整估计器和引用版本，旧 θ 原字节只读/拒写 | 临时合成谱系的实际存储回归；不生成初始根、不做旧历史迁移、不构成生产运行循环 |
| `status.rs`、载体更新接口 | 结构化降级状态、排练默认 m=0、既有 clear KV → install → prefill 顺序 | 没有 Q 权威通知或新载体接线；现有接口测试不证明 ADR0034 的循环状态清除 |

没有新增行为规则、阈值、依赖或身份设定；原验收与失败证据不放宽。`pipeline.rs` 超过 400 行，保留单一事件顺序便于审计，校验集中于 `validation.rs`。本轮自有改动仅在本卡报告目录；主线内容通过 merge 合入。

## 后继项与生产禁用边界

下列项目仍是整卡未完成工作，不能因写入请求或本次条件性交付而改记为通过：

- **持久化时钟与初始 L/Q/U：** boot/epoch、可信墙钟、独立提交时间及原子恢复发布，完整 bootstrap 与正式启动/封存尚未接通。
- **异步绑定：** 派发与完成后的绑定转换、真实异步 IRL 结果收集及资源认证尚未接通。
- **真实终端冒烟：** user_text 与结构化事件经 gateway → runtime → fact-view 的完整链路未验收；Reader::facts 与合成根测试不替代它。
- **Q 通知与权限：** 运行状态/能力、暂停/复核授权、恢复补算与 `run_state` 通知仍未接通。
- **ADR0032 D11 时间语义：** 发生、观察、提交时间的完整持久语义及可信来源、资格规则均属后继设计；接收时间不冒充观察时间，队列延迟不作为丢弃感受的理由。
- **完整阶段与新载体：** 真实 F/C/J、F 的类别/权威计数/关系输入、独立 law/H 来源认证、载体 KV/循环状态清除的运行时接线及校准仍属后继。P0-12 上游测量通过不代替本卡端到端接线验收。旧 θ 迁移也不在本次范围。

D09/D10 不加入定时强制平复，不实现被 ADR0032 取代的旧读法。上述后继实现及其正式验收完成前，**禁止据此接入生产**；补登记、补默认值或将桩替换为真实实现均不能自动解除此边界。

## 主线同步与受影响检查

上一轮 Rust 验证基线 B 为 `67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e`，被测源码 `966fef5a`、审查 head `063894aa`；原始证据见 [R3 manifest](review-r3-20261101/manifest.json)。本轮 fetch 的 M 为 `8fed313b7ca9c31423d0ad442789ddef7e6b063b`，已按用户要求 merge，提交 `8d03f934cf602fb2ff31e3a47c315269d123b4a8`，无冲突、无 rebase/强推。

B→M 增量为 #127 条件载体及状态清除要求、#128 载体测量/用户判定、#121 显式重算机制、#129 部分交付看板识别。已核对所有变动路径和相关语义：唯一 Rust 增量是未登记的 `core-kernel/src/recompute/`，不参与当前 workspace/runtime 编译；其余 core 文件、契约、锁文件、工具链、工作流与已验证组合一致。载体 spike/测量记录及看板工具不进入 runtime 构建。新载体状态清除要求已列入后继，未把旧接口测试扩充为该项通过。

因此本轮受影响的是交付说明、链接及看板识别；不重复未受影响的 Rust 套件，也不将此次文件一致性核对冒称新的 Rust 实测。合入后的 core tree 为 `5f03d138709c0528f834ce76b94595b06cd2beef`，runtime tree 仍为 `9b2dfacf0b2f4c7ba8241be91e343107d4f9b9e8`。检查命令、main 增量分类及同文件/原始证据核对见 [本轮记录](conditional-20261102/result.json)。

| 检查 | 结果与依据 |
| --- | --- |
| 本轮 `git diff --check`、Markdown 相对链接、JSON 完整解析 | 通过，见本轮记录与 [checks-03.log](conditional-20261102/checks-03.log) |
| 使用现有 `delivery_waiting()` 读取本轮 REPORT/BLOCKED | 通过：识别部分交付，并返回三份后继请求 |
| runtime 与已登记 core 编译输入相对审查 head 的逐文件核对；R3 原始日志哈希 | 通过：仅新增未登记的 recompute 目录；原 8 份日志长度/哈希相符 |
| 既有 runtime 测试 | R3 轮 28 项 + 2 项 compile-fail 通过；本轮代码不变，沿用该证据 |
| 既有 workspace 单元/集成/文档测试 | R3 轮分段完整覆盖通过，保留 1 项既有 ignored；本轮未重跑 |
| 既有 fmt / 全 workspace Clippy `-D warnings` | R3 轮通过；相关输入不变，本轮未重跑 |

本轮文档检查首次因同一后继请求使用不同链接标签，看板返回 5 个名称而失败（[原始记录](conditional-20261102/checks.log)）；统一为三份请求的固定标签后该检查通过。另一次检查因验证日志尚未落盘导致自链接不存在（[记录](conditional-20261102/checks-02.log)），本次预创建新的日志文件后复查；未修改检查断言或看板代码。

既有失败与中断全部保留：[R3 red](review-r3-20261101/r3-red.log) 先在合法 V2 读取处以 type tag 错误失败，随后修复；[R1](review-20261101/r1-red.log)、[R2](review-20261101/r2-red.log) 和 [维护入口](review-20261101/r2-maintenance-red.log) 的失败均已修复。首次 [R3 workspace](review-r3-20261101/workspace-test.log) 会话中断、无最终退出码，后以相同源码补齐剩余包与全部文档测试，未把中断记为 exit 0。旧 `validation/runtime-02.log` helper 编译错误、`validation/clippy-01.log` large_enum_variant 已修复；旧开发期三处 enum 拼写错误仅存于当时工具输出。历史 fetch TLS 失败也仍保留。core-kernel 的 `theta_1_original_500_streams_zero_violations` 是既有 V2-93 ignored，本轮未改判定。

## 规格对照与逐项验收

按用户目录限制，矩阵及按编号的 result.json 放本报告目录，不在卡外 core/tests/acceptance 或 docs/traceability 增写。结构通过只覆盖列出的子边界；未运行的完整项不记为通过。

| 规格/卡内验收 | 本次实现与测试 | 结论/未完成部分 |
| --- | --- | --- |
| 审查 R1；v2 §6 | `stages.rs` 保留典型幅度；`review_magnitude.rs` 经实际 F 输入路径得到 0.05 / 0.4 | 视图和传递边界通过；真实 F 接线仍未完成 |
| 审查 R2/R3；算子 §5.3、v2 §§6/8、ADR0029 | `validation.rs` 复用 `check_theta_snapshot_v2`；基态/C 返回/维护入口；零 λ/σ/τ、缺估计器分量、错误参数摘要拒绝及非恒等 C 正例 | 完整 V2 自洽边界通过；独立权威 law/H 来源认证未完成 |
| 审查 R3；ADR0029；W02 | `Snapshot`/C 输入输出及 `read_state` 用生成 V2；`store.rs` 回归读取、无成长准备/提交/重开，保留完整 canonical 字节和引用版本；复用 core-store 旧 head 只读/拒写测试 | 运行时接口定向通过；不等于真实 C 续算、旧历史迁移或完整运行服务 |
| X02；D I 10.1；I14 | 完整绑定及时间/环境核对；同 revision 伪造基线拒绝；真实 W02 旧 ticket 重复提交拒绝 | 结构通过；异步派发/结果收集未接入 |
| X06；v2 §2.4；I04/I12 | 检查实际 mock 调用顺序；G 读 C 前 b；非恒等 G/C；交换 G/C、重复 Φ、F_in 二次推进时间、F_out 二次脉冲均拒绝 | 完整 X06 `not_run`，Commit/J 未形成完整服务 |
| SYS14；D I 10.4；E13 | IRL/A/M 各自降级，非法数据降级；IRL 失败无语义脉冲/成长；A 失败无 G/C，M 失败仍可 C；W02 语义失败提交留下事实/审计 | SYS14 全项 `not_run`，未停真实 worker/model；D I 10.4 仅同步/存储边界通过 |
| SYS23（时钟） | 单次取样、独立 completion、跨 boot 可信墙钟恢复、倒退/epoch/跳变/未知/溢出正反例 | `not_run`：缺原子 checkpoint、真实跨进程重启、Q 通告 |
| F16.4 | W02 顺序入口/旧票据拒绝，D11 保留旧发生时间 | `not_run`：等待、计划取消、已执行动作与延迟反馈关联未接线 |
| F16.5 | 维护只读投影 x，R/θ 数据/数值 bits 保留 | `not_run`：调度评估、阻断与待办恢复未接线 |
| F16.13；D I 10.2；I02/I03/I19 | STAGED x 不可见；提交前磁盘故障不发布；重试后可见；重复过滤不推进锚点；重新打开保留提交 | 全项 `not_run`：不是 runtime 产生语义/Concept 时进程强杀；没有实际 F 分段演化或持久时钟 |
| 卡内真实排练链路冒烟 | synthetic W02 user_text 事务/Reader::facts 测试 | **not_run**：未跑 user_text + 结构化事件经 gateway → runtime → fact-view 全链路；不以 Reader::facts 替代 #65 检索 |
| 正式/实验缺阶段拒绝启动；ADR0010 | complete 检查七阶段及资源；Life 不可调用 rehearsal；STAGED 不可跨谱系 | 类型/配置边界通过；未启动真实服务/诞生 |
| 维护 R/θ 逐位不变；AC2-05 运行时部分；LP5/I11/I21 | 非空 R、多个维护时刻、所有 R/θ 数值 to_bits 与结构对照 | 本次只读运行时边界通过，不声称完整 AC2-05 |
| 暂停成长；v2 §5.4/§13 | IRL/A/M 照常、G/C 恒等，记录 GROWTH_PAUSED | 结构通过；Q 授权/持久通知/恢复补算/复核待接线；布尔输入不是用户授权 |
| ADR0031/0034 E2/推理状态 | 排练默认禁用调制；既有 clear KV → install → prefill 及失败停止结构测试 | 仅既有接口结构通过；新载体 KV/循环状态清除的运行时接线与 P5-31 校准均属后继 |
| ADR0032 D11 | 不按排队时间或 τ 丢弃；发生、接收、process 分别保留；不拿 ingress 冒充 observation | **待后继设计**：没有新时间资格规则，观察/提交持久字段仍缺 |
| ADR0032 D09/D10、D07 等 | 无定时强制平复、无新成长公式、不接旧结果期限逻辑 | 未实现后继规律；既有 workspace 算子回归通过不等于新版 ADR 全部落地 |
| ADR0024/0028；I18/I29 | 生成终端类型、唯一 Store writer API、拥有的临时根 | run_state 推送、封存/正式单实例启动接线待接口 |
