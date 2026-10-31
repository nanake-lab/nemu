风险分级：重档；本 PR 修改 core-runtime 的状态编排和数值输入边界。

# P5-08 编排、阶段接口与时钟（W03）

**部分交付，保持 draft；全卡未验收。** 本轮修复 [PR #120 最新审查](https://github.com/nanake-lab/nemu/pull/120#issuecomment-6514177758) 的 R1/R2，等待复审。完整 W03 服务、持久时钟、初始 L/Q/U、真实阶段适配与入口链路仍未接通，不能据本机结构测试判定整卡完成。用户下一步：安排复审；后继规律和接口问题见 [BLOCKED.md](BLOCKED.md)，本轮不代选、不合并、不领取下一卡。

## 本轮修改与证据

- **R1：** `stages.rs` 的窄数值视图新增生成记录已有的 `typical_magnitude`；F/J 仍不接完整 R、概念定义、证据或审计文本。先以两个不同幅度得到相同视图复现，再验证同一 π=.75、μ=1、方向下，F 收到的贡献分别为 0.05 和 0.4。新增正例仅测试该贡献的输入通路，不是生产 F 实现。
- **R2：** `validation.rs` 在输入边界复用现有 canonical 校验与 `core_types::semantic::check_theta`；`pipeline.rs` 在 C 返回和维护入口复用同一检查。零 λ/σ/τ 被拒绝，C 的非法结果到不了 F_out/STAGED；合法非恒等 C 仍成功。测试使用正数 θ 基态后单项破坏参数，且同步修改 ledger，排除无关编码/绑定失败。冻结 codec fixture 不变，运行时合成 fixture 明确补足正数除数。
- **范围责任：** 严格正数检查不代表 `check_theta_within` 已完成。生产 C/F 适配层须绑定精确 law 资源并检查输入/输出范围；当前权威解析链及 F 所需类别/计数/关系输入仍未接线，见 [阶段输入与 law 请求](../../tasks/requests/P5-08-stage-inputs-and-law.md)。不填默认值，不恢复 ADR0032 已取代的旧规律。

原失败测试及日志先提交于 `b55374a76d59bb29c19975d5b4a68e3cf4b9ba2c`，修复源码与测试提交为 `e1c63b16d1f8d5d38f3bda48795d68c8f611afa1`。产品改动只在 `core/crates/core-runtime/`；其余为本卡报告、证据和请求。没有新增依赖、IDL、持久格式或黄金向量。

## 本机验收

Windows 本机，命令在 `core/` 执行；仅进程级 `CARGO_HOME=D:/Dev/cache/cargo`、`CARGO_TARGET_DIR=D:/Dev/cache/cargo-target`、`TEMP/TMP=D:/Dev/cache/P5-08/tmp`。结构与存储测试使用合成夹具及临时谱系。编译/测试树、日志哈希及命令见 [manifest.json](review-20261101/manifest.json)。

| 检查 | 结果 | 原始证据 |
| --- | --- | --- |
| 修复前 `cargo test --offline --locked -p core-runtime --lib review_r1 -- --nocapture` | 预期失败，exit 101：不同幅度产生同一视图 | [R1 red](review-20261101/r1-red.log) |
| 修复前 `cargo test --offline --locked -p core-runtime --test review_theta -- --nocapture` | 预期失败，exit 101：基态/C 输出零 λ 被接受；合法 C 正例通过 | [R2 red](review-20261101/r2-red.log) |
| 修复前维护入口回归 | 预期失败，exit 101：零 λ 进入投影 | [维护 red](review-20261101/r2-maintenance-red.log) |
| 修复后 `cargo test --offline --locked -p core-runtime` | 26 测试 + 2 compile-fail 文档测试通过；最终源码复核以全工作区运行记录为准 | [开发期 green](review-20261101/runtime-green-01.log) |
| `cargo test --offline --locked --workspace` | 通过，exit 0；保留 1 项既有 ignored；含最终 runtime 26 + 2 项 | [workspace](review-20261101/workspace-test.log) |
| `cargo fmt --all --check` | 通过，exit 0 | [fmt](review-20261101/fmt.log) |
| `cargo clippy --offline --locked --workspace --all-targets -- -D warnings` | 通过，exit 0 | [Clippy](review-20261101/clippy-01.log) |

本轮 red 日志是修复前反例，全部保留。既有失败证据也未改动：`validation/runtime-02.log` 的 helper 编译错误和 `validation/clippy-01.log` 的 large_enum_variant 错误已在旧交付修复；旧开发期另有三处 enum 拼写错误仅存于当时工具输出，不能作为通过证据。旧工作区 ignored 为 core-kernel 的 `theta_1_original_500_streams_zero_violations`（V2-93 既有诊断），本轮不改其判定。

本轮已 fetch 并 **merge origin/main**，merge 提交 `222d0913fb3bd862aa3d67671154f029b2cc0144`，无冲突。实际验证 main 基线 B 为 `db8802f317b5b4dd5fe57699ef724fbac0d940f1`；原交付基线 `02948dc11aa1b43dc0dfe67c9da2e61676427385` 至 B 的增量为文档、规则和报告，已采用 AGENTS 简单优先、重档验证及新版读法。没有 rebase 或强推。收尾一次无返回的只读连接被终止；随后 [fetch TLS 握手失败](review-20261101/fetch-final.log)，[同命令重试成功](review-20261101/fetch-final-02.log)，最终 M=B。只为重试命令设置低速超时，没有修改用户或系统设置。

最终文档检查：`git diff --check` 通过；本轮 Markdown 相对链接逐项确认存在。最终报告/证据提交不改产品源码或测试；被测 `core/` tree 为 `b90cf1f92b3dd99e6a564175c137c397fec559a0`。

## 既有交付边界与未完成项

阶段编排只到 `StagedEvent<L>`；W02 只为已有 head 准备无成长事务。`pipeline.rs` 超过 400 行，保留单一顺序便于审计，校验拆在 `validation.rs`。时钟仅是显式输入的计算，没有 OS provider/原子 checkpoint/Q 恢复发布；排练 m=0 与 clear KV → install → prefill 只有结构验收。

未完成：完整 Commit/J 服务；原子时钟与初始 L/Q/U；Q 状态/暂停/复核授权及通知；异步 IRL 绑定；真实 gateway→runtime→fact-view 冒烟；真实 F/C/J 适配与 law 范围绑定；正式启动/封存接线。ADR0032 D11 发生/观察/提交语义仍**待后继设计**，接收时间不冒充观察时间，队列延迟不触发丢弃感受；D09/D10 无定时强制平复。

对应请求：[权威发布](../../tasks/requests/P5-08-runtime-publication.md)、[D11 时间语义](../../tasks/requests/P5-08-result-time-semantics.md)、[F 输入与 law](../../tasks/requests/P5-08-stage-inputs-and-law.md)。这些缺口不因本轮 R1/R2 修复而消失。

## 规格对照与逐项验收

按用户目录限制，矩阵及按编号的 result.json 放本报告目录，不在卡外 core/tests/acceptance 或 docs/traceability 增写。结构通过只覆盖列出的子边界；未运行的完整项不记为通过。

| 规格/卡内验收 | 本次实现与测试 | 结论/未完成部分 |
| --- | --- | --- |
| 审查 R1；v2 §6 | `stages.rs` 保留典型幅度；`review_magnitude.rs` 经实际 F 输入路径得到 0.05 / 0.4 | 视图和传递边界通过；真实 F 接线仍未完成 |
| 审查 R2；算子 §5.3、v2 §§6/8 | `validation.rs` 复用 `check_theta`；基态/C 返回/维护入口；`review_theta.rs` 零 λ/σ/τ 拒绝及非恒等 C 正例 | 严格正数语义边界通过；绑定 law 的范围验收未完成 |
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
