# 持续认知方案的规格与施工映射

- 日期：2026-10-25
- 基线：`edf66a569a56313e6083b6c3c00bff4d56faefba`
- 提议：[ADR 0026](../adr/0026-continuous-cognition.md)
- 状态：设计映射，不是已实现映射；下表验收均未运行。未发布新的字段号、类型标签或数值规律。

| 规格条款 / 当前合同 | 本方案的处理 | 后续实现归属 | 候选验收 |
| --- | --- | --- | --- |
| ARCHITECTURE §1、§3.4；v2.0「她是谁」 | 明确清醒态持续认知；事项完成不终止全局循环，修改“模型只在需要时运行”的适用规则 | core-runtime、task-layer、运行状态展示 | COG-A01、A10 |
| v2.0 §2.1、§2.4；I02/I18/I21；S §8 | 新内部记录提交分支；H/Q 原子发布；F/G/C 写权限不变；读时不写回 | core-runtime、core-store、IDL 后继 | COG-A03、A07、A09 |
| S §11；ADR 0012 §7；CONTRACTS §5.1 | o/p/c_task/m 分域；m 可影响显式查询提议，执行规则仍确定；c_core 不受该选择控制 | fact-view、task-layer、model-adapter | COG-A06、A08 |
| v2.0 EV3/EV6/EV10、§6–8；I23/I24/I25 | 内部材料不造新证据，纯内部记录不新增 F 内容驱动；保留 m=0 的 IRL/A/M | 内部提交类型及规则；证据入口 | COG-A07、A08 |
| v2.0 §9.1、§9.5；S §19；I15/I17 | 每轮一个有效 m，下一轮刷新；合法零值不等于失效；本地载体负责活动选择 | model-adapter、core-runtime | COG-A04、A05 |
| v2.0 §9.4；S §20；CONTRACTS §8 | 内部接续不 thinning；对外自主机会、权限、冷却和预算保留；记账不得重新抽样 | Q.P/Q.A/Q.Z、action-gate、执行器 | COG-A01、A02、A09 |
| ADR 0012 模式 A/B/C；CONTRACTS §5 | 既有说话模式保留；B/C 不自动算本地严格连续认知，助手材料须经本地 m 下选择 | task-layer、model-adapter、模式展示 | COG-A04、A05 |
| N0 P0-09 决定第 4–6 项 | 保留先回复、后理解方向与单张 3080；无新增串行解释前置，不编造新延迟门槛 | 调度、model-adapter、语音集成 | COG-A03、A10 |
| ADR 0007 §10–11；terminal.proto 的 TaskInput/PlanAndCarrierReceipt；state.proto 的 CoreTransactionV2 | 新增有类型对象与内部提交后继；不将审计字段、普通输出块或 Mode C 草稿暗变因果状态 | core/contracts、注册表、canonical 编码、迁移 | COG-A09；保留 SYS17/SYS21 等原验收 |
| S §11 的撤回/来源资格；I18；AGENTS §5 日志要求 | 工作引用、摘要、在途输入同样受撤回影响；内部内容只由获准核心查询查看 | task-layer、fact-view、观察接口 | COG-A08、A11 |

差异编号 COG-01 至 COG-05 见 [diff-register.md](diff-register.md)。这些编号与 COG-Axx 验收候选不同，不占用其他分支的 V2 编号，不改变既有登记。
