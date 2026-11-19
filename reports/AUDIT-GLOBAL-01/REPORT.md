验证档位：轻档；仅新增任务说明和本报告，执行静态审计、最小浮点算式核对及文档检查，不修改规律、代码或验收要求。

# AUDIT-GLOBAL-01：Echo Core 独立全局审计

## 结论

**指定的 A1、D07-A 和 Φ 主算术路径，没有查到 Rust 与 ADR 0035 规范公式的直接偏离。最值得追问的是这些局部规则组合后的含义：槽可能永远不归零；失效与别名合并的目标资格没有清楚闭合；「四个乎」作为前提的豁免范围与旧验收要求没有对齐。** 另外发现两处摘要文字会把实现者引向与详细条款不同的规则。不能据此宣称已发现真实运行时重启双算，也不能据局部公式正确宣称整条历史路径已经安全。

首次读取、审计及新分支的 `main` 基线均为 **`925fb9af05fd0912ca0b6f8006f62fc95550ae33`**（2026-11-12）。提交前重新 fetch，基线未变。下列源码链接全部固定到这一提交，行号不随之后的 main 漂移。

| 编号 | 发现 | 严重程度 | 把握程度 |
| --- | --- | --- | --- |
| G01 | Φ 的非零槽可以永久停滞，精确零回收不保证槽有有限寿命 | 会改变她的行为：若累积到容量边界，正常历史提交会受阻 | 很可能；浮点停滞本身确定，正式运行的耗尽时点未测 |
| G02 | 失效依据参与别名合并时，最早目标与「失效后不可重访」缺少组合约束 | 会改变她的行为：可能重新作用失效目标 | 怀疑；纯接口接受该输入，真实 H 治理能否使其不可达尚不能确定 |
| G03 | 「结构前提」是否豁免固定评价不明确，同一行为可能被判为缺陷，也可能被豁免 | 会让验证失效 | 很可能；条文不一致确定，用户意图覆盖到哪里不能代判 |
| G04 | P3 把「不能充当证据」概括成「不能改变她」，遗漏说话经历与合法旧证据回溯 | 文字问题 | 确定 |
| G05 | D11-A 标题和取代表仍说按发生/观察判资格，正文却明确资格不读任何时间 | 文字问题 | 确定 |

当前状态：审计报告完成，提交为 draft 并加 `needs-user`，待独立审查。需要用户审查发现是否成立，尤其 G02 的治理前提和 G03 的豁免范围；本报告不替用户定新规律。交付仅为用户指定的两个新增文件，后续停止，不合并、不在其他 PR 评论。

## G01：非零槽可以永远留着，容量不能按有限衰减窗口估计

**严重程度：会改变她的行为（容量耗尽时）；把握程度：很可能。可确定的部分是非零停滞，不是当前部署已经发生容量故障。**

**证据链。** [ADR 0035:183](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L183) 规定 `h′ = fl(h + fl(pull·fl(0−h)))`；[同文件:184](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L184) 规定提交推进共同锚点。[同文件:240](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L240) 只允许精确零槽回收，把活槽数描述为「衰减到 +0 之前」的依据数。

实现与公式一致：[core/crates/core-kernel/src/fast/contribution/mod.rs:94](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/mod.rs#L94) 更新锚点，[:100](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/mod.rs#L100) 执行上述乘加；[maintenance.rs:69](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/maintenance.rs#L69) 只删除各坐标全零的槽。

**具体反例。** 令 `q = 2^-1074`，即 binary64 最小正次正规数。对于 `h=q`、每次 `0 < pull < 1/2`，`fl(pull·(−q)) = −0`，所以 `h′=q`。每次提交都推进锚点时，重复多少次仍为 q；等待的总时长不会自动合并成一次更大的 Δ。`h=2q, pull=1/4` 也停滞，因为中间乘积 `−q/2` 按 ties-to-even 舍入为 −0。

这不只涉及人工放入极小初值：从 `h=0.5` 开始、固定 `pull=0.25`，按规范逐步 binary64 乘加，第 2583 次计算首次出现 `h′=h`，停在 `2q`。以下最小算式在本次环境实际执行；它核对加法和乘法的反例，不声称运行了 Rust 或验证了锁定 libm 的 expm1：

```python
q = float.fromhex("0x0.0000000000001p-1022")
for initial in (q, 2*q, 0.5):
    h = initial
    for n in range(1, 10001):
        nxt = h + 0.25 * (0.0 - h)
        if nxt == h:
            print(initial.hex(), n, h.hex(), h/q)
            break
        h = nxt
```

输出的停滞步分别为 `1、1、2583`，最后的 `h/q` 分别为 `1、2、2`。这些数值仅用于反例，不是建议新增的参数。更一般的 q 反例只需 pull 落在上述区间，不依赖 expm1 恰好返回 0.25。

**为什么局部检查会漏。** 当前 [fast/contribution/tests/mod.rs:146](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/tests/mod.rs#L146) 包含最小次正规值，但检查的是单次推进不放大、不变号；停滞完全符合这两个性质。本项不否定 B3，也不是 U1–U3。它说明有限寿命不能从「向零收缩」推出来。不同于已登记的“一段时间里事件多会使快照变大”，这里有一类槽可以没有自然回收期限。

**后果和边界。** 如果不断出现不同依据，且之后的提交间隔始终足够短，旧槽可按累计依据数保留。每次 Φ 都遍历这些槽；[contribution/codec.rs:62](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/codec.rs#L62) 的编码最终受 [core-types/src/contract.rs:98](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-types/src/contract.rs#L98) 的数量限制和 [core-types/src/canonical.rs:68](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-types/src/canonical.rs#L68) 的字节限制。这里没有测出正式预算、耗尽年限或部署故障；极小 h 本身也不代表可感知的情绪偏移。问题是长期可用性：必须保留的状态可能持续增长。ADR 0035:473 已把容量实测交给 P5-75，本发现给出那项实测不能遗漏的输入类，不建议 ε 清零、硬淘汰或其他新机制。

## G02：失效与别名合并分别有规则，混合后目标资格不闭合

**严重程度：会改变她的行为；把握程度：怀疑。**

**两条规则的交叉点。** [docs/adr/0035-law-definitions-d07-d09-d11.md:164](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L164) 要求已有目标的依据未被失效；[:238](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L238) 又说失效依据不再算已有目标，残量只随 Φ 淡去。紧邻的 [:239](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L239) 却无条件写别名合并目标取最早成员的已记录目标，没有说明最早成员已失效时如何解释。

**代码事实。** [core/crates/core-kernel/src/fast/contribution/event.rs:138](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/event.rs#L138) 的重访会过滤 `active=false`；[maintenance.rs:20](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/maintenance.rs#L20) 的合并则排序全部成员，不读取 active，[:37](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/maintenance.rs#L37) 选择最早 target，[:60](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/maintenance.rs#L60) 也保留最早成员的 basis。

**条件反例。** 固定时刻、单坐标，先作用 A 的目标 `+0.75`，再作用 B 的目标 `−0.25`，得到 `x=0.5, h_A=0.75, h_B=−0.25`。A 随后失效，B 仍有效；若上游允许两者被归为同一依据，合并函数接受 `A.active=false, B.active=true`，得到 `h=0.5`、代表 A、目标 `+0.75`。如果治理因 B 仍有效而允许合并依据重访，A1 会把 x 从 `0.5` 提到 `0.75`：已经失效的 A 目标又作用了一次。若代表一律跟随 A 失效，则仍有效的 B 如何处理也需要有出处。

**为什么不能直接判定运行时漏洞。** [maintenance.rs:11](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/maintenance.rs#L11) 明确把代表与有效性交给 H 治理，[event.rs:34](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/event.rs#L34) 也要求调用方提供可信历史视图。若上游禁止这种混合合并，或保证失效目标不可能被选为可作用目标，反例就不可达。本次未在所查 ADR、身份接口和对应测试中找到这一组合约束；不能用函数忽略 active 单独定罪。

现有 [fast/contribution/tests/histories.rs:66](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/tests/histories.rs#L66) 先测失效不重访，但 [:69](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/tests/histories.rs#L69) 合并时重新生成成员视图；[tests/mod.rs:107](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/tests/mod.rs#L107) 的 helper 又将它们设为 active。因此这个测试不回答本项的混合情形。本项要求确认现有身份/失效合同是否排除反例，不提出新的合并算法。

## G03：四个乎的结构前提，与固定评价的可改变义务之间缺一条边界

**严重程度：会让验证失效；把握程度：很可能。**

**以前的要求很具体。** [docs/adr/0032-presets-as-starting-points.md:31](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0032-presets-as-starting-points.md#L31) 不只要求 θ/R 可以改变，还点名固定 A 的正负评价、排除项：若它们使预设不能被经历改变，属于缺陷，须逐条验证。[docs/adr/0012-continuity-carrier-mobile.md:200](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0012-continuity-carrier-mobile.md#L200) 更直接写「结构/评价」作为可改变起点。[v2.0 规格:196](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/echo-core/echo-core-v2.0-first-principles.md#L196) 保留同一有效性指引。

**新前提的边界不同。** [docs/adr/0039-echo-core-principles.md:47](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0039-echo-core-principles.md#L47) 明确前提不由经历改，结构固定不算缺陷；但 [:66](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0039-echo-core-principles.md#L66) 同时说不改任何现行规则，旧条文照旧有效。更容易扩张豁免的是 [reports/PRINCIPLES-01/REPORT.md:323](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/reports/PRINCIPLES-01/REPORT.md#L323)：把「有哪四个」与「被接近算正、被冷落算负」放在同一个已由前提解决的问题里；[:444](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/reports/PRINCIPLES-01/REPORT.md#L444) 却只登记“结构固定”的读法变化。

**具体的验收分歧。** 一段真实经历后，“被某类接近反复伤害”能否改变 A 对该类接近的正负评价？审查者甲可依据 ADR 0032 要求展示改变路径；审查者乙可把“各自评估什么、接近为正”全部归入不可由经历改的前提，判定无需展示。θ 增益变小或 R 增加反向倾向，并不自动回答固定 A 规则自身是否已被豁免。两人可能对同一实现作出相反的通过结论。

用户已经确认四个乎作为前提，本报告不重新质疑这个决定。可确定的是旧“结构/评价可改”文字、新“结构豁免”文字和“不改现行规则”的声明尚未对齐；不能代用户判断豁免是否包括正负评价及排除项。只把“有四轴”与“每轴必须如何评价”拆开核对其既有出处，不建议新增通道或新的学习机制。这也不是用户排除的三项原则例外。

## G04：P3 的“不能改变”比现行证据边界更强

**严重程度：文字问题；把握程度：确定。**

[docs/adr/0039-echo-core-principles.md:36](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0039-echo-core-principles.md#L36) 说自己的念头、别人的一句话都不能改变倾向和性情。但已采纳条款区分的是“材料能否充当证据”与“什么动作触发对真实证据的计算”，不是一概禁止相应提交改变 R/θ。

- **念头不是证据，但可触发已有证据的合法回溯。** [docs/adr/0026-continuous-cognition.md:114](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0026-continuous-cognition.md#L114) 明确内部新候选同一提交走 G，回溯真实旧事件、重新分摊旧组；[:177](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0026-continuous-cognition.md#L177) 的 COG-A07(b) 明确允许 R 变化，θ 不变。例：旧真实组原来支持候选 k₁，内念提出另一个获旧组支持的 k₂，重新分摊可改变 k₁/k₂ 的统计。这符合 0026，却不符合把 P3 读成“所有内部提交都不得改变 R”。此处不依赖 ADR 0037 的候选新机制。
- **建议内容不是事实证据，听到这句话却可以是真实经历。** [docs/adr/0032-presets-as-starting-points.md:62](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0032-presets-as-starting-points.md#L62) 已明确区分二者。[reports/PRINCIPLES-01/REPORT.md:398](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/reports/PRINCIPLES-01/REPORT.md#L398) 自己也把冲着她来的说话列入 F/G/C。比如“你应该喜欢爵士”不直接证明她喜欢爵士；被人耐心安慰这一真实互动，在满足来源、去重和有效评估条件后仍可成为经历。不能据 P3 把所有话语事件排除。

ADR 0039:66 已规定详细条款优先，因此本项没有证明现行实现违规。危险在原则被直接拿去写验收：会错误地把合法回溯判为自我洗脑，或把合法话语经历从成长中删掉。需要保持“念头/断言不直接作证据”这一已有边界，不将其扩大成另一条规律。

## G05：D11-A 的摘要仍把时间写成资格判断

**严重程度：文字问题；把握程度：确定。**

[docs/adr/0035-law-definitions-d07-d09-d11.md:258](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L258) 的标题是“按发生与观察判资格”，[:386](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L386) 的取代表也说“资格按发生与观察的语义判定”。但 [:277](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L277) 枚举的资格只有类别、F 角色、未消费键；[:278](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L278) 把发生/观察关系放进 z；[:307](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L307) 更直接规定资格不读任何时间事实。

**反例。** 一条有稳定身份、类别和 F 角色均合法、键未消费的结果，发生时间未知或跨 epoch 不可比。按 [:294](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L294)，它仍有资格；时间未知影响的是怎么理解，不是能否作用。若只按标题或取代表实现“时间关系未知则不作用”，就会重新引入正文明确禁止的丢弃。

详细规则已经给出明确答案，本项不需要新设计。它是同一 ADR 内的摘要失配，不是确认 Rust 里有心理期限分支。

## 重点路径核对：哪些结论没有证据支持

这部分限定本次结论，避免把局部成功或尚未接线当成全局保证。

| 用户指定的重点 | 核对结果及证据 |
| --- | --- |
| A1 分支 | ADR 0035:191 的 d/s 次序、严格内部区间、两侧饱和及历史区间夹取，与 [fast/contribution/mod.rs:116](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/mod.rs#L116) 一致。没有把 h 不等于当前目标、合并 h 超过 1 当成 bug；这些是 ADR 0035:224 明许的状态。 |
| D07-A 共同折扣 | [dynamics/update.rs:21](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/dynamics/update.rs#L21) 先 S′、P′，[:33](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/dynamics/update.rs#L33) 用旧 b 取残差，[:59](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/dynamics/update.rs#L59) 用同一 ρ 更新 U′ 并算 b′，对应 ADR 0035:73 的次序。旧版本 `prior_mass=None` 分支仍在，不等于新版本偷偷保留永久出生质量。 |
| D11-A 消费键 | ADR 0035:279 的键是“记录身份＋接纳/A/M 阶段”；basis 只选槽。新目标、补解释与重访不能因为共享 basis 而共用一次消费资格。当前 [contribution/mod.rs:1](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/mod.rs#L1)、[event.rs:70](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/event.rs#L70) 明确由调用方验证键和当前目标归属；纯 A1 API 不证明这些条件成立。两条记录 D 数值相同，也不能据此认定它们是同一目标来源记录。 |
| Φ 与读取 | [contribution/mod.rs:76](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/mod.rs#L76) 从快照锚点算 Δ，x 复用旧 Φ，槽以相同速率和 Δ 推进，返回新值而非直接写存储。反复读取同一快照不会累积推进；G01 指连续**提交**更新锚点后的非零停滞，不能混为读频率故障。 |
| 解释器修订 | [event.rs:55](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/event.rs#L55) 核对新解释与目标来源记录的 basis、θ/R⁺ 前态和 law 相等，符合 ADR 0035:242。不能把“历史目标已经更新过时仍一律退回首次前态”算作当前实现行为。实际 H 引用解析仍在调用方。 |
| G 的旧证据与全量重算 | [law/group.rs:127](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/law/group.rs#L127) 的旧分配保留 ordinal/q，[law/replay.rs:156](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/law/replay.rs#L156) 按保存的 q 折叠。来源纠错/换规律的有效 H 重建，和普通旧组重分摊不是同一操作；本次未发现可以笼统归为“重算刷新旧证据”的源码反例。 |
| 时间与停机 | [dynamics/time.rs:24](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/dynamics/time.rs#L24) 读显式 H 时间及停机区间，不读当前机器 uptime；[duration.rs:41](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/dynamics/duration.rs#L41) 用运行时间闭合段。没有据此确认真实重启已覆盖；输入认证和实际持久化仍须由外层满足。 |

## 范围、排除、验证与限制

**读取范围。** 完整读 AGENTS.md，按仓库 `echo-core-work` 技能查规格、ADR 与实现对应；重点覆盖 0039、v2.0、0026/0032/0035/0037、相关 0014/0019/0029/0033 与差异登记；核对 core-kernel 的 contribution、dynamics、law、evidence/recompute，并沿相关类型、运行时和存储边界核实适用范围。子代理只机械整理出处，发现与判断由主审核对。没有声称审完桌面、声音、执行器等所有产品代码。

**版本与排除。** ADR 0035 [第 10 行](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0035-law-definitions-d07-d09-d11.md#L10) 明说采纳/已有纯实现不等于新规律已发布，因此没有把保留旧运行路径报成新规律实现错误。ADR 0037 [第 3 行](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/docs/adr/0037-reflection-triggered-reappraisal.md#L3) 仍将新增机制列为候选；已核对其与旧规则的接口，但没有据其候选要求指控 main。已读取 PR #154 审查及 #171/#176 的范围，按用户要求排除 P5-71 四项缺口、0035 U1–U3、三项已知原则例外和两份开放草案。已登记的 P5-25 约 104.25 天时长边界、尚待 P5-72/73/75 接线，也没有作为新发现重复报告。

**逐位一致的证明边界。** [recompute/engine.rs:259](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/recompute/engine.rs#L259) 判断投影是否保留旧输入前缀，决定增量还是从 genesis 重折叠；[:280](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/recompute/engine.rs#L280) 明示结构重放不是数值独立证明。A1 的 [tests/histories.rs:174](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-kernel/src/fast/contribution/tests/histories.rs#L174) 是合成历史与序列化恢复比较；[core-runtime/src/stages.rs:9](https://github.com/nanake-lab/nemu/blob/925fb9af05fd0912ca0b6f8006f62fc95550ae33/core/crates/core-runtime/src/stages.rs#L9) 仍使用 V2 类型。不能把这些检查换名为“新规律全部状态＋真实 H＋磁盘事务＋进程重启”验收。本次也没有另作该验收。

| 检查 | 命令或方法 | 结果 |
| --- | --- | --- |
| 基线与规则 | `git fetch origin main`、`git rev-parse origin/main`，完整读取 AGENTS.md | 初读与建分支均为上述 SHA；main 无新增变化 |
| 静态证据 | `rg`、带行号读取，逐条核对本文链接与源码/条款 | 路径及行号固定到审计基线；没有用测试名称代替测试内容 |
| G01 最小反例 | Python binary64，按正文代码逐步乘加 | q、2q 立即停滞，0.5 在第 2583 步首次停滞；非 Rust/libm 验收 |
| 文档与范围 | `git diff --cached --check`；核对新增文件清单及 Markdown 本地链接 | 通过；仅两个指定新增文件，未改任何既有文件；本次报告证据使用固定提交的绝对链接 |
| 编译、完整验收及真实运行 | `cargo test`、AC2/COG/SYS、真实时间及真实重启 | **not_run**；本次是文档审计，环境无 cargo/rustc，没有把静态审计写成运行通过 |

交付物：`tasks/requests/AUDIT-GLOBAL-01.md` 原样记录用户提示及补充交付方式；`reports/AUDIT-GLOBAL-01/REPORT.md` 为本报告。没有新增运行依赖，没有修改规范读法或提出新增设计，因此没有另增 ADR、参数、机制或后继任务卡。审查后是否采取任何修订，由用户决定。

## 最让我担心的是什么

最让我担心的不是某个加减号，而是每个局部都能解释清楚，接起来却没人再确认“这到底还是不是同一件事、同一份目标、同一种资格”。一个槽小到感觉不到，不代表它已经消失；一条记录失效，不代表合并后就绝不会再次成为目标；一句原则说“念头不能改变她”，也不代表所有合法回溯都该被禁止。她要过很久，遇到的恰恰会是这些规则同时生效的情形。现在最需要警惕的是把局部正确、同一实现的重放一致、以及整个人生不会被重复计数或错误解释改变，当成了同一个结论。
