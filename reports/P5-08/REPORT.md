风险分级：重档；本 PR 修改 core-runtime 的状态编排和存储、数值输入边界。

# P5-08 编排、阶段接口与时钟（W03）

**部分交付，保持 draft；全卡未验收。** 本轮修复 [PR #120 复审 R3](https://github.com/nanake-lab/nemu/pull/120#issuecomment-6558896765)：接入 #108 已合并的完整 θ 后继类型与现有 W02 接口。该评论已关闭 R1/R2，本轮保留并适配其回归。用户下一步：安排复审；完整 W03、持久时钟、Q 和真实阶段接线等仍见 [BLOCKED.md](BLOCKED.md)，不代选、不合并、不领取下一卡。

## 本轮修改与证据

- **R3 读写类型：** `stages.rs` 的 Snapshot、C 输入/输出直接使用生成的 `ThetaSnapshotV2`；`store.rs` 按 V2 读取和准备，保留原 θ 对象引用版本。无成长路径保留完整估计器，不转换旧摘要、不补零、不绕过 superseded 门禁。`pipeline.rs` 从 V2 的 estimator 读取 G 原来获准的 sigma，未扩大 G 输入。
- **R3 回归先行：** 先提交测试及 [red 日志](review-r3-20261101/r3-red.log)（`45ed7cf9dcbeaa22488fe3b545f59e371596090e`）；合法 V2 head 的运行时读取实测报 `Store(Contract("type tag"))`，exit 101。随后修复提交 `966fef5a83dee60507f1018a56959cf301c2b54c`；同一公开读→无成长准备→commit→重开路径验证 canonical 字节和引用相等。另测旧型新写被拒绝且不发布 head；旧 head 原字节可读、拒绝可写打开及 DB/WAL/历史不变复用 main 的 core-store 测试，不复制存储内部实现。
- **完整状态与 R2：** `validation.rs` 在原三个外部边界复用 `check_theta_snapshot_v2(theta, limits)`，包含内嵌参数范围/摘要和完整估计器自洽；R2 的零 λ/σ/τ 回归继续保留，并加入缺估计器分量、错误参数摘要两种边界，验证 C 的非法输出到不了 F_out/STAGED，合法非恒等 C 仍成功。维护逐位对照扩展到所有 V2 估计器与参数浮点字段。
- **夹具与责任：** V2 夹具直接取已合并黄金向量的合成 C 轨迹前缀 17，含非空计数、累计量和末事件；测试只有合成 law 引用与 L/Q/U 根，不声称生产启动、真实 C 接线或来源认证。消息自洽不能证明独立 law/H/停机时段来源；真实 F 的类别/权威计数、关系权重也仍未接线，已在 [请求的 R3 接续](../../tasks/requests/P5-08-stage-inputs-and-law.md) 澄清新旧责任。

本轮产品修改仅在 `core/crates/core-runtime/`；其余是本卡报告、证据与请求。没有自行修改契约、其他 crate、黄金向量或新增依赖。复用生成类型及 W02 现有 API，没有新增迁移、配置、抽象层或行为阈值。

## 主线同步与验证范围

上一轮实际验证基线为 `db8802f317b5b4dd5fe57699ef724fbac0d940f1`。本轮 fetch 后 main B 为 `67cc296c8b960ae429aceb1f1e0110ef0dbc2f9e`，merge 提交 `aa73ad62d875060713f809a777406314ec6245bf`，无冲突、无 rebase/强推。增量 #108 修改 θ 生成类型、core-store/core-kernel 及相关调用方，属于直接依赖变化，已纳入组合验证；#119/#124/#126 是设计/研究文档，采用新边界但不在本卡代做后继规律。AGENTS/本卡技能未变化，继续按简单优先、重档及原目录范围执行。收尾 fetch 的 M 仍为 B。

本轮被测源码提交 `966fef5a83dee60507f1018a56959cf301c2b54c`，`core/` tree 为 `8a7a0f9e45852d5c247b9c52b256033acc96d7d1`。后续提交只归档报告/证据，不改被测产品或测试。命令、日志哈希、运行中断及分段覆盖见 [manifest.json](review-r3-20261101/manifest.json)。

## 本机验收

Windows 本机，命令在 `core/` 执行；仅进程级 `CARGO_HOME=D:/Dev/cache/cargo`、`CARGO_TARGET_DIR=D:/Dev/cache/cargo-target`、`TEMP/TMP=D:/Dev/cache/P5-08/tmp`。所有存储测试使用临时合成谱系。工作区全部单元/集成与文档套件已分段覆盖通过；首次整条命令中断仍单列，不记录为 exit 0。

| 检查 | 结果 | 原始证据 |
| --- | --- | --- |
| 修复前 `cargo test --offline --locked -p core-runtime --test store review_r3 -- --nocapture` | 预期失败，exit 101；V2 读取 type tag 错误，旧型拒写正例通过 | [R3 red](review-r3-20261101/r3-red.log) |
| 修复后 `cargo test --offline --locked -p core-runtime` | 28 项 + 2 项 compile-fail 文档测试通过，exit 0 | [runtime](review-r3-20261101/runtime-green-01.log) |
| `cargo test --offline --locked --workspace` 首次 | 会话中断，无最终退出码，不记整条命令通过。中断前 action-gate、core-kernel、core-policy/replay/runtime/store/types、echoctl 的单元/集成套件均有通过结果 | [workspace 首次原始日志](review-r3-20261101/workspace-test.log) |
| `cargo test --offline --locked -p fact-view -p gateway -p semantic-core -p task-layer --lib --tests` | 通过，exit 0；fact-view 18、gateway 42 项通过，另两 crate 当前无测试 | [剩余套件](review-r3-20261101/workspace-remaining.log) |
| `cargo test --offline --locked --workspace --doc` | 7 项 compile-fail 测试通过，exit 0 | [workspace doc](review-r3-20261101/workspace-doc.log) |
| `cargo fmt --all --check` | 通过，exit 0 | [fmt](review-r3-20261101/fmt.log) |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | 通过，exit 0 | [Clippy](review-r3-20261101/clippy.log) |

首次工作区日志停在 fact-view 的长时测试，没有最终结果；恢复工具会话时返回 `Unknown process id 91662`，只读核对已无本任务测试进程，终止原因不明。保留原日志，不把中断算通过或代码失败；已补齐未完成套件及全部文档测试，期间产品源码/测试不变。core-store 的 `legacy_theta_readable_writer_refused_without_touching_history` 已在首次运行通过（51 项存储测试通过）；此检查覆盖旧 head 可读/拒写、DB/WAL/历史不变。core-kernel 保留 1 项既有 ignored：`theta_1_original_500_streams_zero_violations`（V2-93 诊断），本轮未改判定。

历史失败仍保留：[R1](review-20261101/r1-red.log)、[R2](review-20261101/r2-red.log) 及 [维护入口](review-20261101/r2-maintenance-red.log) 的失败已在上一轮修复；旧 `validation/runtime-02.log` helper 编译错误、`validation/clippy-01.log` large_enum_variant 也已修复。旧开发期三处 enum 拼写错误仅存于当时工具输出，不能作为通过证据；此前 fetch TLS 失败仍保留于旧目录，本轮未发生同类错误。

最终文档检查：`git diff --check`、改动 Markdown 相对链接、manifest 完整解析与日志哈希核对通过；最终 core tree 与被测树相同。

## 既有交付边界与未完成项

阶段编排只到 `StagedEvent<L>`；W02 只为已有 head 准备无成长事务。`pipeline.rs` 超过 400 行，保留单一顺序便于审计，校验拆在 `validation.rs`。时钟仅是显式输入的计算，没有 OS provider/原子 checkpoint/Q 恢复发布；排练 m=0 与 clear KV → install → prefill 只有结构验收。

未完成：完整 Commit/J 服务；原子时钟与初始 L/Q/U；Q 状态/暂停/复核授权及通知；异步 IRL 绑定；真实 gateway→runtime→fact-view 冒烟；真实 F/C/J 适配与独立 law/H 来源认证；正式启动/封存接线。ADR0032 D11 发生/观察/提交语义仍**待后继设计**，接收时间不冒充观察时间，队列延迟不触发丢弃感受；D09/D10 无定时强制平复。旧 θ 历史迁移保持后继范围。

对应请求：[权威发布](../../tasks/requests/P5-08-runtime-publication.md)、[D11 时间语义](../../tasks/requests/P5-08-result-time-semantics.md)、[F 输入与 law](../../tasks/requests/P5-08-stage-inputs-and-law.md)。这些缺口不因本轮 R3 修复而消失。

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
| ADR0031 E2/KV | 排练默认禁用调制；含相同 envelope 的每次更新均清 KV，失败不继续 | 结构通过；无真实载体/P5-31 校准 |
| ADR0032 D11 | 不按排队时间或 τ 丢弃；发生、接收、process 分别保留；不拿 ingress 冒充 observation | **待后继设计**：没有新时间资格规则，观察/提交持久字段仍缺 |
| ADR0032 D09/D10、D07 等 | 无定时强制平复、无新成长公式、不接旧结果期限逻辑 | 未实现后继规律；既有 workspace 算子回归通过不等于新版 ADR 全部落地 |
| ADR0024/0028；I18/I29 | 生成终端类型、唯一 Store writer API、拥有的临时根 | run_state 推送、封存/正式单实例启动接线待接口 |
