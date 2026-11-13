# MATH-U2U3-01

## 原始提示词（逐字）

请对 GitHub 仓库 nanake-lab/nemu 中的两项未证明性质做数学研究：证明或给出反例。只读，不修改仓库。先记下你读取的提交号。

材料：docs/adr/0035-law-definitions-d07-d09-d11.md（§4.1–4.6：A1 依据替换、Φ 推进、已证性质 R1–R4、B1–B7）；docs/echo-core/echo-core-v2.0-first-principles.md §6；docs/adr/0026-continuous-cognition.md（内部记录与重访）；PR #171 中的 docs/adr/0038-recall-affect-fading.md（§3 统一定义，S1 逐坐标、N0）；实现见 core/crates/core-kernel/src/fast/ 与 src/dynamics/。

问题 1（§4.6 U2）：在 binary64、固定运算顺序、无 FMA、零已规范化的前提下，扩展距离 d_◇(s, s̃) = max_i(|x_i − x̃_i| + Σ_c |h_{c,i} − h̃_{c,i}|) 在一次 A1 事件中是否非扩张？请按 Rust 实现的实际运算顺序分析，可以用代码解释器做定向和随机搜索。若不成立，给出具体反例（十六进制位模式和逐步舍入），以及最紧的可证误差界与成立条件。事件步与无事件 Φ 步分开讨论。

问题 2（§4.6 U3）：在任意多依据、重访顺序和时机可任意安排、但没有新外部事件的条件下：
(a) 只有 ADR 0035 的规律时，x 能否被重访无限期维持在高位或趋向饱和？给出上界、可达性构造或反例。
(b) 加上 ADR 0038 方案 B 后，在什么条件下（r 的形状、λ、重访频率）可以证明 x 最终回落到 x_rest 附近？给出定理和证明，或说明还缺什么假设。

要求：每个结论标明「已证明」「反例」或「猜想」，写清全部假设，不确定就直说。最后用白话写一段：这对这个角色意味着什么。

## 补充交付方式（逐字，后续指示）

补充交付方式：先完整读 AGENTS.md 并遵守。只新增 tasks/requests/MATH-U2U3-01.md（照抄本提示词）和 reports/MATH-U2U3-01/REPORT.md（结论在前）；如有反例搜索代码，放在 reports/MATH-U2U3-01/ 下作为附件，不放进 core/。不改任何其他文件。从最新 main 新建分支，开 draft PR，加 needs-user 标签，写「请审查」，然后停下。不合并。
