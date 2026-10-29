# P5-08 编排、阶段接口与时钟（W03）

## 当前状态

**部分完成，保持 draft；全卡未验收。** 2026-10-30（Perth）收尾。PR #120，分支 `task/P5-08-orchestration`，工作区 `D:/Dev/nemu-wt/P5-08`。已实现可独立验证的阶段编排、谱系能力、时钟计算及既有 W02 head 的无成长准备接口。完整时钟持久化、初始 L/Q/U、异步语义派发、真实入口及运行状态发布仍缺卡外接口，见 [BLOCKED.md](BLOCKED.md)。不合并，不领取下一卡。

开工已 fetch；#64 的 merge commit `02948dc11aa1b43dc0dfe67c9da2e61676427385` 即实际 main 基线 B。#65 `8845567376e4726cc63d6f148cedd46ec563cd34`、#36 `9042c59455efa340e194ce7a53fa397c09b324af` 均经祖先检查在 B 中。收尾 fetch 后 M 仍等于 B，无新增差异。期间一次 fetch、一次 push 出现 schannel 握手失败，同命令重试成功，未修改 TLS 或任何用户/系统设置。

## 交付物与边界

- `core-runtime/src/pipeline.rs`：Γ 由调用方经 W02 过滤后，按 Φ → H/当前事件视图 → IRL → A → M → F_in → G → C → F_out → Ledger 形成 `StagedEvent<L>`。顺序常量包含 Commit/J，但当前没有完整 W03 外层服务，不能用常量代替 X06 执行证明。此文件超过 400 行，保留单一显式顺序便于审查；输入/数值校验另在 `validation.rs`。
- `stages.rs`：IRL 无 x/R/θ；A 只接 z/绑定；M 只接概念引用供定义解析；G 无 x/m/g/λ；C 无 R；F/J 只接 R+ 数值视图，无证据或审计文本。均为进程内可信适配器；存在性不证明语义/数学实现合格。线上的生成消息仍来自 core-types。
- `binding.rs`：使用真实 canonical codec 比较完整绑定；不静默 rebase。
- `clock.rs`：boot/epoch、显式可信墙钟不确定度、单次处理取样与独立 completion 取样；倒退/跳变/未知/溢出显式报错。阈值由调用方提供，无生产默认值。没有可信 OS provider、持久 checkpoint 或 Q 故障恢复策略。
- `store.rs`：复用 Store/ReadTicket/PreparedObject/PreparedTransaction。只为已有 head、R/θ 不变且无新增 L 的计算准备事务，重核基线内容；不伪造初始根，没有生产 commit loop。准备成功不等于 W03 durable completion。正式/实验的 `compile_committed` 因缺权威 Q 发布契约明确拒绝；排练默认禁用调制并暂停自主行动。每次载体更新必须 clear KV → install → prefill，任一步失败就返回。
- `status.rs`：生成 DegradedStatus/RunStateV2 的投影；状态组合不清除其他故障，EXPERIMENT 不冒充 terminal LIFE。调用方须提供真实 born_at、speech mode、strict；这里不推送消息，也不更新完整 Q 或授予权限。
- [权威发布请求](../../tasks/requests/P5-08-runtime-publication.md)与 [D11 请求](../../tasks/requests/P5-08-result-time-semantics.md)。没有修改契约、存储、其他 crate、任务卡或差异登记。

## 本机验证与证据

Windows；rustc `1.99.0 (b940084d7 2026-09-28)`；cargo `1.99.0 (5f94df478 2026-08-27)`。编译产物 `D:/Dev/cache/cargo-target`，测试临时存储 `D:/Dev/cache/P5-08/tmp`（仅进程 TEMP/TMP）。没有打开用户存储、操作 runner、等待或查询 CI。

| 命令/检查 | 结果 | 证据与限定 |
| --- | --- | --- |
| `cargo test --workspace` | 通过，exit 0；1 项既有 ignored | [workspace-test-01.log](validation/workspace-test-01.log)；编译被测提交 `2019efd750ea18d28b9f1a6c45888ac7c3bb8ead` |
| 最终 `cargo test -p core-runtime` | 20 测试 + 2 compile_fail doc tests 通过，0 ignored | [runtime-final.log](validation/runtime-final.log)，被测提交 `ac5aa7d302c6b9429d113621900fef7fa4f3cb70` |
| `cargo fmt --all --check` | 通过，exit 0 | [fmt-02.log](validation/fmt-02.log)，`ac5aa7d3` |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过，exit 0 | [clippy-final.log](validation/clippy-final.log)，`ac5aa7d3`；`clippy-01.log` 保留 large_enum_variant 失败，后改用 Box，无 lint 抑制 |
| 开发期运行时检查 | 一次测试 helper 编译失败，修复后通过 | `runtime-02.log` 误用 Schema 方法名；改为 encode/decode_canonical 后 `runtime-03.log`、`runtime-04.log` 通过，保留失败证据 |

Fixture 来自基线冻结 golden/legal.json 的 canonical 字节，来源见包内 `tests/fixtures/README.md`。存储测试初始根明确为 **synthetic**，不是 genesis/完整 Q。数值适配器也是结构夹具：有非恒等 G/C 和 F_out 新维度正例，但不能声称真实成长或调制有效。

验证增量：全工作区在 `2019efd7` 编译；随后只有 runtime 内部 IRL 非法输出降级、强化数值 bits/持久审计断言及 fixture 说明变化（`ac5aa7d3`）。没有公共接口、依赖、契约或其他 crate 源码变化，故重跑全部 runtime 测试和全工作区 Clippy/fmt；未重复无关包的长重放/崩溃测试。最终报告/证据提交不改源码。各项实际基线与文件哈希见 [manifest.json](validation/manifest.json)。

既有 ignored 为 core-kernel `theta_1_original_500_streams_zero_violations`（V2-93 legacy Python rounding diagnostic，指向既有失败报告）；本卡未增删忽略项，未重写旧验收结论。首次开发 check 还修正过三个生成 enum 名称拼写错误（TerminalLineage/Autonomy）；那一轮仅有对话工具输出，没有另存原始文件，不把它用作通过证据。

## 规格对照与逐项验收

按用户目录限制，矩阵及按编号的 result.json 放本报告目录，不在卡外 core/tests/acceptance 或 docs/traceability 增写。结构通过只覆盖列出的子边界；未运行的完整项不记为通过。

| 规格/卡内验收 | 本次实现与测试 | 结论/未完成部分 |
| --- | --- | --- |
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
| ADR0031 E2/KV | 排练默认禁用调制；含相同 envelope 的每次更新均清 KV，失败不继续 | 结构通过；无真实载体/P5-31 校准 |
| ADR0032 D11 | 不按排队时间或 τ 丢弃；发生、接收、process 分别保留；不拿 ingress 冒充 observation | **待后继设计**：没有新时间资格规则，观察/提交持久字段仍缺 |
| ADR0032 D09/D10、D07 等 | 无定时强制平复、无新成长公式、不接旧结果期限逻辑 | 未实现后继规律；既有 workspace 算子回归通过不等于新版 ADR 全部落地 |
| ADR0024/0028；I18/I29 | 生成终端类型、唯一 Store writer API、拥有的临时根 | run_state 推送、封存/正式单实例启动接线待接口 |

## 新增依赖与公开格式

无新增 Cargo 依赖、锁文件变更、IDL、持久格式或黄金向量。无直接 SQL/第二 writer。Sample、Projection、ActiveTendency、Bound 仅为不可序列化的进程内数据，不是手写跨进程协议。

## 待后继设计与下一步

- [ ] 原子时钟/commit wall、初始 L/Q/U、Q 状态/能力与通知契约。
- [ ] ADR0032 D11 的发生/观察/提交资格设计，保留 D09/D10 已取消统一回落的边界。
- [ ] 异步 IRL 绑定、完整阶段实现适配、真实临时入口冒烟、正式启动/封存接线。
- [ ] 用户安排高难度独立审查；本 PR 仅写「请审查」。

未决事项已写问题、选项和建议，没有等待用户回复或代为决定她是谁。按 AGENTS §2「全部验收完成后才把草稿转为可审查；部分完成时保持草稿」，本次保留 draft。下一步只补本卡缺口；本轮停止，不领取下一卡。
