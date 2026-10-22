ADR 0018 的旧参考形式会在弱支持到来时因衰减旧支持而降低整步 π。本提案选用按新信号限制衰减的实际 q，固定依恋先验与复杂度基数，并保存每组首次 q，使重新分摊中的撤回也保持单调。

新增 [ADR 0019（提议）](https://github.com/nanake-lab/nemu/blob/codex/law-lp1-reference/docs/adr/0019-law-lp1-reference.md)，比较保持 π 的压缩与选定形式，逐项列出 LP2–LP8、I23/I25、推论1–7及 PR #49 D1–D12 的影响。LP1 按用户指定的三条分别检查：无修订整步方向、相同分摊的新增边际、撤回自身方向。

验证：完整同号新增事件80,000步零违例；三类判据分别9,000/9,000/5,019次零违例；8,000标量前缀与6,000事件前缀重放零位差；strict mypy通过。原 LAW-1–10同种子同规模执行，**LAW-6仍失败：3,000序列、3,064个状态违例**，其余九项通过。旧判据及原始失败没有修改。

四种合成世界各128种子×512步：本设置下新形式MSE和稳定误翻转较低，突变察觉均值11.15625→11.453125步，略慢；仅报告，不设门槛。完整数据、失败轨迹、源快照及校验清单在 [报告](https://github.com/nanake-lab/nemu/blob/codex/law-lp1-reference/reports/law-lp1-reference/REPORT.md) 与 [最终原始输出](https://github.com/nanake-lab/nemu/tree/codex/law-lp1-reference/reports/law-lp1-reference/verification/attempt-05)。

`needs-user`：新规律、推论6撤换、历史q/计算次序、零信号处理、份额量化与后继契约需求均待批准。已同步最新main并复跑；按用户要求转为可审查，这不表示新规律或交叉审查已批准。PR不合并，P5-23仍待后续设计决定。按用户最新指示，本会话不主动请求Claude审查；后续安排由用户决定。

范围只有文档与离线报告/Python：原check_law.py、Rust、IDL、PR #49分支均未修改。现有CI路径规则跳过此类变更；不将未运行的完整核心、存储或行为验收标成通过。
