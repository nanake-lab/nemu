轻档。本次只提交两份开工核对文档，属于交付线。

# P5-11 「乎」评估 worker

## 当前状态

阻塞，停在实现前。执行结果为「未知」时，核心可以记住收到过回执，但这个回执不能证明她成功、失败，或对此毫无反应。旧卡要求结构化事件经规则表产出四维评分，已定决定要求先核对来源与覆盖范围，四个分量有充分依据才可作为有效 A 提交。正式评分量尺仍缺，恢复条件见[阻塞记录](BLOCKED.md)。

核对基线为 `da40fd067f5c659fe34b35eb03571ef5921fce08`。P5-21 已随 [#36](https://github.com/nanake-lab/nemu/pull/36) 合入，记录带事件、四维值、所涉人、ProposalBindingV2、生产者和版本。P5-07 合入的是基础库，整卡仍有正式 IPC、模型测量及人工标注未完成项。[记录契约](../../core/contracts/records.proto#L6-L22) · [P5-07 当前状态](../P5-07/REPORT.md)

## 逐份核对

| 依据 | 对 P5-11 输入、输出或验收的影响 |
| --- | --- |
| [ADR 0042 §4 U5/U6](../../docs/adr/0042-cause-projection.md#L178-L195) | U5 分配说话、独处与行动选择。U6 要求解释失败时如实显示故障，迟到的补解释不补发。派工指定「乎」评估只在本地、以 m = 0 运行。U5 文中的「本地 A」指载体路线，评估算子的读集依据仍是 EV6。 |
| [ADR 0043 §3、§5](../../docs/adr/0043-two-lines.md) | 载体研究和首版交付分别验收。本卡只需可替换的本地测试接口，真实端点联测另记；研究线的 GPU 测量不属于 worker 单测。 |
| [ADR 0045 D1、§2](../../docs/adr/0045-speak-think-wiring.md#L7-L24) | 过渡期云端只能生成独处念头文字。依本次派工收窄卡面的后端选择，移除远程日常档作为 A 的实现路径；不影响向量范围和三语质量标准。 |
| [ADR 0046 §2.1](../../docs/adr/0046-speak-think-contract-gaps.md) | 云端内部文字有单独的回执和采纳合同。P5-11 的成功输出沿用 P5-21 的 NatureAppraisalRecord，由语义核心治理。 |
| [ADR 0047 草案「背景与决定」第 3 项](https://github.com/nanake-lab/nemu/blob/59a79fab4ac2f62836a67965f2ea4ad61aacf0cc/docs/adr/0047-permission-contracts.md#L5-L15) | [#235](https://github.com/nanake-lab/nemu/pull/235) 尚为草稿。提案只约束授权、凭证和写权，未给 A 增加读取、执行或提交权限。本次没有采用草案接口。 |
| [证据合同 U1–U9](../DESIGN-EVIDENCE-CONTRACT-01/DECISIONS.md) | **影响验收。** U3 把字段到语义的适配限定为已冻结来源；U4 区分未评、已评未知、已知零和失败。对已支持且足以评价的输入，仍检查确定性和完整四维；对未覆盖或不足的输入，必须检查不会产出有效评分。U1/U2/U9 涉及来源、概念及版本，U5–U8 属下游证据处理。 |
| [时钟报告 §2](../DESIGN-CLOCK-01/REPORT.md#L15-L27) | U1–U7 涉及时间证明、提交、年龄、恢复与长时段学习。A 不从墙钟、离开时长的猜测或超时时间推算强度；接口超时仍返回失败。该组决定没有给 worker 新增时钟字段或质量阈值。 |

## 规格与验收

| 卡面项目 | 本次结果 |
| --- | --- |
| 确定性规则、四维范围及来源覆盖 | 阻塞。卡面与已定来源边界的具体反例，以及正式量尺缺口，见[阻塞记录](BLOCKED.md)。 |
| 中文、英文、日文各 100 条 | `not_run`。保留冻结人工标注、各通道符号一致率、平均有符号偏差、语言间差与 schema 合格率 ≥95% 的原要求。 |
| EV6 输入隔离、EV10 评估值与快状态分离、完整绑定及核心拒绝错绑 | `not_run`。按 [v2.0 §§2.4、3–4](../../docs/echo-core/echo-core-v2.0-first-principles.md)、[ADR 0008 §1](../../docs/adr/0008-echo-core-math-clarifications.md) 和 [P5-21 的输出类型](../../core/contracts/records.proto#L11-L22) 核对了规格，未运行 worker 或核心测试。 |
| 可替换本地后端、真实本地端点、超时降级联测 | 均为 `not_run`。桩测试属于本卡；真实端点待 [P5-13](https://github.com/nanake-lab/nemu/pull/242) 合入后联测；整链路的 A_DEGRADED 在 P5-19 验收。 |
| 本 PR 文档 | `git diff --cached --check` 及两份新增 Markdown 的相对链接检查通过。 |

## 限制

本次完成卡面、决定、设计和依赖状态核对。worker 实现与产品验收均未开始。GPU 使用次数为零。草稿 PR 等待用户选择[恢复方式](BLOCKED.md)。
