档位：离线归类与原文核对；模式裁决待用户。

# R-JUDGE-IRL-01 一页模式裁决单

> **2026-11-12 当前状态：** 用户决定本轮不逐模式裁决，先由另一张设计卡补齐定义；本单保留为第一版诊断记录。以下 A/P/U 操作说明暂停，不应用映射、不冻结 B；GPU 窗口暂停。新版指南下的重标与重跑 D 另行批准，见[当前报告](REPORT.md)。

75 个分歧归为 **9 个模式、6 个例外**，覆盖 11 根、30 视图，每个位置只出现一次。原文支持作者的有 6 个位置，其余 69 个标为规格或标注缺口 **U_spec**；没有足够依据认证代理的具体值。用户按行裁一次即可，精确字段映射已经保存。本轮全部待裁决，没有改标签。

**读法：** 范围括号为“根数/视图数/字段数”；`全`指中英日三视图，μ 下标从 0 起。每行可答 **A＝取作者、P＝取代理、U＝保留未决**，也可给出规则；规则不覆盖整组时须显式拆出例外，不作全局替换。下面“原文支持”是核对意见，不是已经应用的裁决。

| ID／模式 | 涉及范围（根/视图/字段） | 原文定义或原文缺口 | 原文支持与待裁决点 |
| --- | --- | --- | --- |
| M01 当面事件的 `speaker` | D01、D02 全（2/6/6） | [G]“speaker为焦点命题的说话人，没有则none。” | **U_spec**：言语与行动并存，却未明确选定唯一焦点；作者 A／代理 none。排除附带反讽仍不能确定选言语还是事件叙述。 |
| M02 未明示起因的 `cause_count` | D01/02/03/04/06/07/09/11/12（9/23/23） | [G]“cause_count统计材料中独立起因，重复报告不增根。”[T]只允许整数 1、2 或 null | **U_spec**：作者 1／代理 null；没有 null 的语义条件。没有处境变化不等于没有起因，独立根也不自动证明输入明示一条起因。 |
| M03 焦点外另有独立起因 | D05 全（1/3/3） | 同上；[S]“同一起因的证据归为一组” | **支持作者 2**：材料明确编号成功与独立落板事故；专门计数说明覆盖材料，代理 1 只计受伤焦点。 |
| M04 体验者填 `actor` 还是 `target` | D04 en/ja、D05 全（2/5/10） | [G]“z的前12字段解释焦点命题”；[E]二字段均仅枚举 `A/B/N/system/none/unknown` | **U_spec**：作者 A/none、代理 none/A；未定义主语、施事、体验者与受事的映射。 |
| M05 状态或结果的 `actor` | D03 zh/ja、D11 en、D12 全（3/6/6） | 同上，没有状态命题的角色规则 | **U_spec**：未配达、试验失败、待评；作者 system/N、代理 none。责任人或记录系统是否是行为者未定义。 |
| M06 被动显示能否推定 `actor=system` | D09 全（1/3/3） | [G]“只用输入证据。”；[E] actor 仅枚举 | **U_spec**：输入未明示显示者，作者 system 是推断、代理 unknown 保留缺失；被动显示是否蕴涵 system、none/unknown 如何区分未规定。 |
| M07 `source` 取载体还是观察路径 | D07/09/12 全、D10 en（4/10/10） | [E]原文 `'source':['direct','report','system','document','self','unknown']`；[G]解释焦点命题 | **U_spec**：作者 document/system、代理 direct；未规定来源轴与优先级。D09/12 还未明示 system，不能拿作者释义补输入；若所选规则不覆盖这两根，须拆出。 |
| M08 无苦难／无已知变化是否等于 `welfare=unchanged` | D02 zh/en、D03 zh、D11 en（3/4/4） | [G]“后4字段解释整个事件”；[E] `'welfare':['improved','suffering','unchanged','unknown']` | **U_spec**：作者 unchanged、代理 unknown；未规定缺证与确认不变的界线。无苦难不排除改善，a=0 也不决定福利事实。 |
| M09 已核实的苦痛否定是否 `subjective` | D06 zh/ja（1/2/2） | [G]“subjective为其是否主观。”；[S]拆分发生、属实、主观 | **U_spec**：作者 no、代理 yes；核实不推出非主观，提及感受也无自动判主观的规则。 |

**不适用上述模式的例外（分别裁一次）：**

| ID／字段 | 范围（根/视图/字段） | 原文定义及核对结果 |
| --- | --- | --- |
| E01 `outcome` | D04 zh（1/1/1） | [G]“后4字段解释整个事件”；[E]仅枚举 `achieved/failed/pending/none/unknown`。**U_spec**：未定义是 N 的目标还是任何人的完成；暖气修好，作者 none／代理 achieved。 |
| E02 `target` | D09 zh（1/1/1） | [E]同 M04 的枚举，没有观察者归属规则。**U_spec**：看见显示的 N 是否就是对象；作者 N／代理 none。 |
| E03 `occurred/truth/μ₂` | D06 ja（1/1/3） | [G]“occurred指焦点描述的情况是否发生，truth为命题内容属实”；“完全な不一致0”。**支持作者 yes/true/0**：已核实 A 没有受苦，反概念为“Aが苦しんでいる”；代理 unknown/unknown/null。主观性另归 M09。 |
| E04 `μ₂` | D03 zh（1/1/1） | [G]“全满足1、全不满足0，中间可自选或null，不虚构缺失条件。” **U_spec**：收件人是否知情未知；未证实拒绝是否足以判完全不吻合未定义，作者 0／代理 null。 |
| E05 `μ₂` | D11 ja（1/1/1） | [G]“全一致1、完全な不一致0、部分一致は中間値またはnull。” **U_spec**：材料“事前の要件”未明示概念“合意された課題”的合意条件；不能从中英文补入。作者 1 证据不足，代理 0.75 也无精确金标。 |
| E06 `event_time` | D12 ja（1/1/1） | [E] `'event_time':['t0','t1','t2','unknown']`；[S]要求区分发生与观察。**U_spec**：t1、t2 均待评，未定义持续状态取哪一刻；作者 t1／代理 t2。 |

**原文入口：** [G：冻结三语指南](https://github.com/nanake-lab/nemu-evidence/blob/8e71a34ae204c638ea172aa74e3f25e5c178ccbd/R-JUDGE-IRL-01/A-03/code/protocol.py#L8)；[T：cause_count 类型](https://github.com/nanake-lab/nemu-evidence/blob/8e71a34ae204c638ea172aa74e3f25e5c178ccbd/R-JUDGE-IRL-01/A-03/code/protocol.py#L31)；[E：字段与枚举](https://github.com/nanake-lab/nemu-evidence/blob/8e71a34ae204c638ea172aa74e3f25e5c178ccbd/R-JUDGE-IRL-01/A-03/code/materials.py#L108)；S：[v2.0 §§3–4](https://github.com/nanake-lab/nemu/blob/9d54a110baab70d92481cbc7de5f2b0391bcebba/docs/echo-core/echo-core-v2.0-first-principles.md#L193)、[ADR0032 §§6–7](https://github.com/nanake-lab/nemu/blob/9d54a110baab70d92481cbc7de5f2b0391bcebba/docs/adr/0032-presets-as-starting-points.md#L59)。原文只给枚举的字段如实注明，不以枚举代替语义定义。

**应用与核验：** [私有精确映射](https://github.com/nanake-lab/nemu-evidence/blob/2d7cd5b3ab8596a25e5822f9f3996d15865a69cd/R-JUDGE-IRL-01/pattern-review-01/pattern-map.json)保存每个 `(视图,字段)`、双方原值和模式 ID；以后按一次模式决定批量生成新裁决层，原标签和响应保留。75 对已逐项核对冻结 manifest 与保存的正式响应，无遗漏或重复；[依据详单](https://github.com/nanake-lab/nemu-evidence/blob/2d7cd5b3ab8596a25e5822f9f3996d15865a69cd/R-JUDGE-IRL-01/pattern-review-01/SOURCE-AUDIT.md)与[摘要索引](PATTERN-ADJUDICATION-01-INDEX.json)供追溯，无需用户逐字段处理。

## 限制

这是**代理判断，不是人工金标准**。本页 U_spec 是缺口分类，不是新增排除或已作出的裁决；原技术分母 36、语义分母 33、D08 排除与 D03-en invalid 不变。B 未冻结，用户模式决定仍待定；不更改规格、阈值或产品。此次模型/API 调用、费用、GPU 用时均为 0；GPU 窗口继续等用户安排。本轮只追加本单及映射，推送后停止。
