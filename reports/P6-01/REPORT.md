重档。改动覆盖任务层生成、行动登记的边界及 SYS14 任务路径实测。

# P6-01 1/2 任务层与协议 p v0

本次交付 1/2。[Claude 的复审意见](https://github.com/nanake-lab/nemu/pull/250#issuecomment-7065096644)要求的三项已补齐。设置文件缺失时可以用默认预算正常回复。任务层使用 #235 的作用域和凭证类型，过期绑定会在登记或校验时被拒绝。三语候选提交 `781cec16daec816853e594065dff743f503a3b74` 已核实为 [nemu-content 远端分支 task/P6-01-task-layer](https://github.com/nanake-lab/nemu-content/tree/task/P6-01-task-layer) 的分支头。

## 交付与规格对照

| 项目 | 实现与验收范围 |
| --- | --- |
| W21、I28、ADR 0027 | [协议加载器](../../core/crates/task-layer/src/task/protocol.rs)按候选或定稿来源、语言和内容绑定版本。候选状态会推送，重复状态去重，失败可重试。中英日候选仅含事实、边界、表演语法和语言规则，内容提交为 `781cec16daec816853e594065dff743f503a3b74`。三份协议和清单均已解析，N2/P6-12 由用户签字。 |
| TaskInput、X01、SYS06 | o、p、c_task、快照与预算分别装配。更换协议或观察前沿会使旧计划失效。[核心隔离测试](../../core/crates/task-layer/tests/core_isolation.rs)固定事件、事实和时间，单独改变 p 或 c_task，核对 W03 的 x/R/θ、L、绑定与算子调用顺序。 |
| ADR 0042、ADR 0045 D5 | [模式 B 组包](../../core/crates/task-layer/src/task/projection.rs)复用 cause_projection::assemble。c_task 的必需事实连同更正链参与一次去重和容量计算，包含实际输出预留。云端适配器只收到已组包字节。 |
| D3 | 缺记录提示有 none、per_event、general 三档，默认 per_event。在已授权快照及其来源内标注。 |
| D6 | [说话要点核对](../../core/crates/task-layer/src/task/speech.rs)解析一次列点，按治理来源、说话人、已投递 UTF-8 区间及语义支持逐项核对，按序拼接保留项。 |
| D7 | 自引核对可选 loose 或 strict，默认 loose。测试覆盖他人的文字、发送区间外文字及其他分支的来源。 |
| D9 | `generation.tokens` 默认 128，`generation.duration_ns` 默认 6,000,000,000，`k` 默认 2。可逐字段覆盖，缺文件或缺字段时采用默认值。配置损坏或读取失败会报错。 |
| D10 及追加 | 模式 B 的选样接口绑定语种、m、样例库版本、距离度量、选择与渲染版本、k、顺序和渲染文字，在 P 的「说话样例」段统一计量。模式 A 生成整库、人工签字及真实选样器分别由归属卡落实。 |
| ADR 0046、0047 | 已合入 #235。行动登记从目录解析真实 ConnectorScope，生成绑定含完整协议引用、权限资源与撤销代际。ExecutionCredential、ActionRequestV2 和签发表、认证连接及原样载荷经 check_dispatch_binding 与当前权限共同核对。 |
| SYS14 | [回合与回执](../../core/crates/task-layer/src/task/reply.rs)区分等待、失败、生成和关闭。已终止回合持久化并恢复所需的接口已提供。[脚本测试](../../core/crates/task-layer/tests/w21.rs)统计生成和登记次数，验证零作用及允许的零调制，并用实际等待跨过生成预算。 |
| 主动选择、SYS08、SYS13 | 替身接收输出块和行动请求。空计划不登记，载荷类别或绑定不符则拒绝。[真实中断与重放验收](BLOCKED.md)依赖 #249。 |
| CONTRACTS §§4.3、5.1 | [只读查询](../../core/crates/task-layer/src/task/user_view.rs)对接 W19 的 user_view 记忆与宿主的行动／运行历史接口，返回前留审计。[查询测试](../../core/crates/task-layer/tests/user_view.rs)覆盖三种查询及五类非成功状态。 |

默认值依据：ADR 0027 要求按句切分并尽量缩短首句，首个声音仍需在目标 TTS 上重测。P0-12 的本地 Qwen3.5-4B 在无约束 50 条输入上测得首 token 中位 283.88 ms、解码中位 116.62 token/s。128 token 约需 1.1 s 解码，整轮给 6 s，给预填充和结构化生成留余量；两条同语言近邻样例提供变化，同时限制 P 占用的上下文。这些是可调工程上限；U6 等候解释没有时限。[ADR 0027](../../docs/adr/0027-voice-model-breeze-target-gsv-interim.md) · [本机测量](../P0-12/REPORT.md)

本机 Windows 在实现提交 `4abad0f8` 上运行 task-layer 测试，67 项通过、零失败、零忽略，Clippy 全目标零警告。缺配置正例经生成和 outbox 登记完成一轮；权限测试覆盖成功以及八种绑定变化或替换。SYS14 本轮实测等待 150.596 ms，生成及登记耗时 0.462 ms，生成一次、登记一块。此前三份 Clippy 失败及后续复验均保存在证据索引中。

完整命令和验收编号见[证据索引](EVIDENCE.md)，生产接线职责见[宿主接口请求](../../tasks/requests/P6-01-runtime-consumers.md)。[条款索引](../../docs/traceability/P6-01.md)关联具体测试和规格。

## 依赖与维护

task-layer 直接使用工作区 fact-view 和 core-store。新增直接使用的外部运行时依赖均已在锁文件中，serde 1.0.229 和 serde_json 1.0.154 用于序列化，sha2 0.11.1 用于资源绑定，time 0.3.44 用于查询时间，许可证均为 MIT OR Apache-2.0。测试依赖 core-runtime 和 tempfile 3.27.0。reply.rs 集中保存同一回合从等待到登记的状态转换，w21.rs 共用同一套接收端计数器，以便逐条核对跨阶段断言。这两个文件超过 400 行。

## 并行与下一步

此次并行由 Claude 授权、用户确认。受保护范围包括 cognition 目录和 #238 在 task-layer 修改的其他文件。共享 lib.rs、Cargo.toml 仅追加本卡入口和依赖，冲突由后合并方处理。本次提交 1/2 供审查。2/2 等 #249 合并后再开，真实 outbox 的 SYS08、SYS13 仍为 `not_run`。终端接线及 N2 协议签字见[剩余工作](BLOCKED.md)。
