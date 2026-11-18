# IMPL-CAUSE-PROJ-01 第 0 步：未写死与有出入的边界清单（OPEN）

本文档对照 [ADR 0042](../../docs/adr/0042-cause-projection.md)（§0、§1.2、§1.3、§2.1–§2.4、§3.2、§3.3、§5.1、§5.3）与 [任务卡](../../tasks/requests/IMPL-CAUSE-PROJ-01.md)（§4–§6），列出同一份输入下存在两种以上合理输出结果、而 ADR 与本卡尚未共同钉死的地方（含本卡 §5 与 ADR 原文有出入或未写到的边界）。按对输出的影响从大到小排列。所有建议均为**候选**，请定夺。

---

## Q1：`D` 中部分投递条目（`Delivered::Partial`）的完整正文是否自动进 `E`

- **ADR 原文**：
  - §2.1：「同一原记录的正文出现位置固定为 **o 优先于 D，D 优先于 E**；D 内保留所选对话中该身份首次出现的位置。……实际投递片段与完整原记录不是同一个呈现对象：D 只保留已投递范围，只有完整所需原文已在 D/o 时才可扣除该原文的新增占用；不把未投递部分补成她已说过的话。」
  - §3.2：「请求取消或仅投递部分输出 | 下一轮 D 只把已记录实际投递范围当作已说的话；完整生成稿仍按原合同记录其真实身份。」
  - §5.1：「她的说话事实以 H 中实际投递记录及范围为准，未投递生成稿保持内部产物身份，不混入 E 冒充已说的话。」
- **具体例子**：
  - `records` 含 `a1`（`h_pos = 10`，`body = "前半句已说出｜后半句被打断未说出"`，`Eligible`）和 `o2`（`h_pos = 20`，`body = "继续"`，`Eligible`）。
  - `dialogue = [DItem { id: a1, delivered: Partial { start: 0, end: 18 } }]`（仅前 18 字节已投递），`latest = o2`，`required_facts = []`，`protocol_cited = []`，`relations = []`，`slots = []`，预算充足。
  - 歧义原因：本卡 §5.5 将 `dialogue` 的全部 `id` 作为基础闭包种子放入 `base_records`（故 `a1 ∈ base_records`），§5.8 又定义 `presented_full = {latest} ∪ D 中 delivered == Full 的 id`、`E = (base_records ∪ 已接纳包的闭包) \ presented_full`。按此公式，任何 `Partial` 对话条目 `a1` 因不在 `presented_full` 中，都会把包含未投递后半句在内的完整 `a1` 塞进 `E`，与 ADR §2.1/§3.2/§5.1 冲突。（附：若原始 `dialogue` 中同一 `id` 先后出现 `[Partial(a1), Full(a1)]`，本卡 §5.8 保留首次出现的 `Partial` 并丢弃后面的 `Full`。）
- **可选的读法**：
  - **读法 A（仅「完整所需原文」进 `E`）**：`D`（按 §5.8 去重后）的 `Partial` 条目仍作为基础闭包种子参与资格检查及 `BodySource`/`Correction`/`Unknown` 关系展开（其引用的原文和指向它的更正仍进入 `base_records` 与 `E`，`Binding.base_records` 仍记录完整基础闭包 `[a1, o2]`）；但记录 `r` 自身只有属于「需要完整正文的集合」——即 `{latest} ∪ 去重后 D 中 Full 的 id ∪ required_facts ∪ protocol_cited ∪ 基础闭包与已接纳包闭包中经 BodySource.to 或 Correction.from 到达的记录 ∪ 已接纳候选包的闭包`——再减去 `presented_full` 后才进入 `E`。对上例输出 `Sent`，`D = [a1(Partial 0..18)]`，**`E = []`**。（若 `o2` 另经 `BodySource` 引用 `a1` 或某接纳槽含 `a1`，因 `D` 只有 `Partial`，完整 `a1` 仍按 §2.1 进入 `E`。）
  - **读法 B（按本卡 §5.5 + §5.8 字面）**：所有 `dialogue` 的 `id` 一律进入 `base_records` 并直接参与 `(base_records ∪ 已接纳包) \ presented_full`。对上例输出 `Sent`，`D = [a1(Partial 0..18)]`，**`E = [a1（完整正文，含未投递后半句）]`**；若未投递后半句很长导致超预算，则变为 `BaseRequestFailed(OverCapacity)`。
- **日常话**：
  - 读法 A：上一轮她说话被打断只说出一半，这一轮模型只看到她真正说出口的前半句；除非别的记录明确引用了那条完整记录，否则没说出口的后半句不会混进「本轮来由」发给模型。
  - 读法 B：只要对话窗口里有一条被打断的半截话，没说出口的后半句也会作为完整记录发进「本轮来由」；若没说完的部分太长，还会撑爆预算使她这一轮不回话。
- **你的建议（候选）**：选**读法 A**（候选）。完全贴合 ADR §2.1「不把未投递部分补成她已说过的话」与「只有完整所需原文已在 D/o 时才可扣除」两句话的组合语义，也符合 §3.2、§5.1。

---

## Q2：候选槽无外部来源记录（`Member.sources` 为空或 `Slot.members` 为空）时的处理

- **ADR 原文**：
  - §1.2：「对可用真实来源的非零槽 c，临时计算：`s(c) = max_i abs(h_c,i)`」
  - §1.3：「可用性按完整包判定：必要来源确定不合格（无真实外根、已禁止使用等）时，候选包整体排除并记录缺口；若属于基础材料则本次请求失败。必要端点缺失或资格未决时，整个投影不可得；不删除包内失效成员后继续使用合并总 h 选材。」
  - §3.3：「如果槽与来源都完整可知，却没有合格的非零真实经历，新增经历材料为空，仍可发送完整基础请求。……回执分别记录“无合格经历”……」
- **具体例子**：
  - 基础材料 `D = [x1]`（`Full`），`latest = x2`，预算充足。
  - 例 1（单槽无外根，如内部支线 `FAST_BASIS_INTERNAL_BRANCH`）：唯一候选槽 `S1`（`h = [0.8]`，`members = [Member { basis: b1, sources: [] }]`）。
  - 例 2（合并槽含无外根成员）：唯一候选槽 `S2`（`h = [0.8]`，`members = [Member { basis: b1, sources: [r1] }, Member { basis: b2, sources: [] }]`，`r1` 为 `Eligible`）。
  - 歧义原因：本卡 §4 将 `PackFate::Excluded` 定义为 `Excluded { record: Option<RecordId>, reason: String }`（预留了 `record: None`），但 §5 从未构造 `record: None`。按 §5.6 + §5.9 字面求 `sources` 并集：例 1 得到空闭包并被接纳为 `Accepted { new_records: [] }`、`K = 1`、`AllFit`；例 2 则静默丢弃空来源成员 `b2`，用合并总 `h` 把 `{r1}` 发出去，直接违背 ADR §1.3「不删除包内失效成员后继续使用合并总 h 选材」。
- **可选的读法**：
  - **读法 A（任一成员 `sources` 为空则整包 `Excluded { record: None, .. }`；`slot.members` 为空判 `Err(InvalidInput)`）**：`slot.members` 为空违反「本槽及传递合并进来的全部成员」，在 §5.1 返回 `Err(InvalidInput)`；候选槽（`s > 0`）中只要任一 `Member.sources` 为空（且无更高优先级的 `Unavailable`），该候选包整包标为 `Excluded { record: None, reason: "no_external_source".into() }`（`PackEntry.records` 仍记其余成员求出的完整闭包或空列表）。例 1 输出 `Sent { K = 0, E = [], note: NoEligibleExperience }`；例 2 中 `S2` 整包 `Excluded`，输出 `Sent { K = 0, E = [], note: NoEligibleExperience }`。
  - **读法 B（按本卡 §5.6 + §5.9 字面直接求 `sources` 并集）**：例 1 输出 `Sent { K = 1, E = [], note: AllFit }`（`S1` 为 `Accepted`）；例 2 输出 `Sent { K = 1, E = [r1], note: AllFit }`。
  - **读法 C（`slot.members` 为空或任一 `Member.sources` 为空均视为调用方错误 `Err(InvalidInput)`）**：要求调用方必须用 `Eligibility::Ineligible` 的占位 `Record` 表达无真实外根；传入空 `sources` 一律返回 `Err(InvalidInput)`。
- **日常话**：
  - 读法 A：如果某股残留感受来自纯内部念头、没有对应的外部真实事件，这一包会被排除并如实记为「无合格经历」；如果它和另一件真实旧事合并成了同一个槽，也不会拆掉一半只拿那件旧事去顶替合并后的总感受。
  - 读法 B：没有外部事件的空槽会被当成「已接纳 1 件经历」；合并槽则会悄悄扔掉无外根的那一半，只把另一半真实旧事发给模型说话。
  - 读法 C：只要传入没有来源记录的槽成员，函数直接当作传参错误拒绝运行。
- **你的建议（候选）**：选**读法 A**（候选）。严格满足 ADR §1.3「无真实外根时候选包整体排除」「不删除包内失效成员后继续使用合并总 h 选材」及 §3.3「无合格经历」，并解释了本卡 §4 中 `Excluded.record: Option<RecordId>` 为何是 `Option`。

---

## Q3：`RelationKind::Unknown` 的触发范围（双向关联、仅出边、还是全局关系表）

- **ADR 原文**：
  - §1.3：「关系类别按已记录语义及冻结版本的类型映射决定，不由投影器临时读句子猜测“是否必要”；无法区分的关系按未知处理。」
  - §3.2：「包关系未知、原文丢失、查询完整性不能确认 | 整个投影不可得，结束本次组装；不把未知包当小分量跳过。」
- **具体例子**：
  - 基础材料 `D = [x1]`（`Full`），`latest = x2`，无槽，所有记录 `Eligible`。
  - 情形 1：`relations` 含 `Relation { from: u, to: x1, kind: Unknown }`（入边指向 `x1`）。
  - 情形 2：`relations` 含 `Relation { from: u, to: v, kind: Unknown }`（`u`、`v` 与基础材料及所有非零候选槽均无任何连边）。
  - 歧义原因：本卡 §5.4 写「取出记录 x 后，扫描所有与 x 相关的关系：……碰到 `Unknown` 关系 → `UnknownRelation { from, to }`」。已知关系里 `BodySource`/`Continuation` 看 `from == x`，而 `Correction` 看 `to == x`；既然 `Unknown` 是「类型映射分不清」，`to == x` 的 `Unknown` 边可能正是对 `x` 的更正或撤回。
- **可选的读法**：
  - **读法 A（闭包已访问记录的双向关联：`rel.from == x || rel.to == x`）**：情形 1 返回 `ProjectionUnavailable(UnknownRelation { from: u, to: x1 })`；情形 2 因 `u`、`v` 未进入基础闭包或任何候选槽闭包，正常返回 `Sent`。
  - **读法 B（仅检查出边：`rel.from == x`）**：情形 1 与情形 2 均正常返回 `Sent`（只有 `from == x1` 或 `from == x2` 时才报 `UnknownRelation`）。
  - **读法 C（前沿全局检查：`input.relations` 含任一条 `Unknown` 即不可得）**：情形 1 与情形 2 均返回 `ProjectionUnavailable(UnknownRelation { .. })`。
- **日常话**：
  - 读法 A：只要本轮用到的对话或候选旧事上连着一条「分不清是什么关系」的线（不管连出去还是连进来，例如可能是条未识别的更正），她这一轮就不回话；但完全无关的两条旧记录之间有分不清的关系，不耽误她正常回话。
  - 读法 B：只有从本轮记录「连出去」的未知关系才停下；若有「连进来」指向本轮旧话的未知关系（哪怕其实是更正或撤回），她仍把旧话原样发出去回话。
  - 读法 C：只要可见历史前沿里任何两条无关记录之间有一条未知关系，她这一轮就不回话。
- **你的建议（候选）**：选**读法 A**（候选）。`Correction` 是入边（`to == x`），双向检查才能防止漏掉未知更正，同时符合 ADR §3.2「包关系未知」与本卡 §6.4「包内记录碰到 Unknown 关系」的边界。

---

## Q4：排在 `FirstOverflow` 之后的候选包、或包内已有 `Ineligible` 的候选包遇到 `Unavailable` 时是否仍阻断整轮发送

- **ADR 原文**：
  - §1.3：「可用性按完整包判定：必要来源确定不合格（无真实外根、已禁止使用等）时，候选包整体排除并记录缺口；若属于基础材料则本次请求失败。必要端点缺失或资格未决时，整个投影不可得；不删除包内失效成员后继续使用合并总 h 选材。」
  - §2.2：「基础请求可用后，按 §1.2 的排序逐包试放……整包放得下就接纳；首个放不下就停止，后面的包不再试放。」
  - §3.2：「包关系未知、原文丢失、查询完整性不能确认 | 整个投影不可得，结束本次组装；不把未知包当小分量跳过。与“已知不合格而排除”区分；基础材料本身不可用也不发送」
- **具体例子**：
  - 基础材料齐全且放得下，所有槽 `merge_undecided = false`。
  - 例 1：排名第 1 的槽 `S1`（`h = [0.9]`，源 `{r1}` 合格但超长，触发 `FirstOverflow`）；排名第 2 的槽 `S2`（`h = [0.1]`，源 `{r2}` 的 `eligibility = Undetermined`）。
  - 例 2：排名第 1 的槽 `S1`（`h = [0.9]`，成员源 `{r_bad, r_u}`，其中 `r_bad` 为已知 `Ineligible`，`r_u` 为 `Undetermined`）；排名第 2 的槽 `S2`（`h = [0.5]`，源 `{r_ok}` 合格且放得下）。
  - 歧义原因：本卡 §5.6–§5.7 先对全部非零候选槽求闭包并检查 `Unavailable`，全部通过后才判 `Ineligible` 与 §5.9 装包；而按 ADR §2.2「首个放不下就停止，后面的包不再试放」及 §1.3「确定不合格时候选包整体排除」，例 1 的 `S2` 本不会被试放，例 2 的 `S1` 本已确定排除。
- **可选的读法**：
  - **读法 A（按本卡 §5.6–§5.7 字面：全部候选槽预先求完闭包并统一先判 `Unavailable`）**：例 1 与例 2 均返回 `ProjectionUnavailable(EligibilityUndetermined(..))`，本轮不发送；同时所有 `Sent` 结果中的 `Binding.packs`（含 `Excluded`、`NotTried`）都能填入完整闭包 `records`。
  - **读法 B（先全局检查候选槽 `merge_undecided`，再按排名惰性求包，且包内已知 `Ineligible` 优先排除）**：先检查所有候选槽 `merge_undecided`（防未决合并改变排名）；随后按排名逐槽求闭包并试放——例 1 在 `S1` 处 `FirstOverflow` 即停止，不求 `S2` 闭包，输出 `Sent { K = 0, note: OptionalOverBudget }`（`NotTried` 包的 `PackEntry.records` 为空）；例 2 中若闭包遍历未遇缺失/未知端点但同时含 `Ineligible` 与 `Undetermined`，优先将该包判 `Excluded` 并继续装入 `S2`，输出 `Sent { K = 1, E = [r_ok], note: AllFit }`。
- **日常话**：
  - 读法 A：只要任何非零候选旧事里有没查清或缺失的记录——哪怕排第一的大事已经占满窗口根本轮不到后面那件小事，或者那个包里已有明确禁用记录——她这一轮都停下不回话。
  - 读法 B：只要排第一的大事已经把窗口占满、后面的小事本来就不会试放（或某包已因禁用记录排除），就不会因为用不上的记录状态未决而拦住她正常回话。
- **你的建议（候选）**：选**读法 A**（候选）。完全符合 ADR §3.2「包关系未知、原文丢失……整个投影不可得，结束本次组装」与本卡 §4 `PackEntry.records` 保存全部候选完整闭包的契约，逻辑也最简明。

---

## Q5：直接接续引用（`Continuation.to`）在 `records` 中存在但为 `Ineligible` 或 `Undetermined` 时的处理

- **ADR 原文**：
  - §1.3：「“回复上一条”或会话父消息等接续关系，仅保留本记录中已记的直接目标身份与引用；不把目标正文加入集合/工作队列，也不追它的接续祖先。」「接续边只保留该直接引用，不读取目标记录来继续遍历；只因目标正文未被选中或未读取，不判依赖缺失。」「**接续引用本身继续遵守原有身份与使用权限规则。**」
  - §5.3：「发给 API 服务方 | ……必需事实和入选经历的原始内容，以及原有说话人、记录引用、种类、时间和纠错关系」「既有来源使用权限和在途失效仍适用。」
- **具体例子**：
  - `records` 含 `r9`（`h_pos = 10`，`Ineligible { reason: "revoked" }`；变体为 `Undetermined`）和 `o`（`h_pos = 20`，`Eligible`）；`relations = [Relation { from: o, to: r9, kind: Continuation }]`；`dialogue = []`，`latest = o`，无槽。
  - 歧义原因：`r9` 不在 `records` 时，本卡 §6.3 例 1b 已明确保留 `continuation_refs = [r9]`；但当 `r9` 就在可见前沿 `records` 内且标记为禁止使用（`Ineligible`）或未决（`Undetermined`）时，本卡 §5.4「不读 to」与 ADR §1.3「接续引用本身继续遵守原有身份与使用权限规则」存在张力。
- **可选的读法**：
  - **读法 A（按本卡 §5.4 + §5.8 字面：纯函数不检查 `Continuation.to` 的 `eligibility`）**：无论 `r9` 在 `records` 中是 `Ineligible` 还是 `Undetermined`，均保留 `latest_continuation_refs = [r9]` 并返回 `Sent`（接续边本身能否使用由上游构建关系前沿时治理）。
  - **读法 B（`to` 在 `records` 中可见时按资格过滤引用）**：`r9` 不在 `records` 或为 `Eligible` 时保留 `[r9]`；`r9` 在 `records` 中为 `Undetermined` 时返回 `ProjectionUnavailable(EligibilityUndetermined(r9))`；`r9` 在 `records` 中为 `Ineligible` 时从 `continuation_refs` 中剔除 `r9`（`latest_continuation_refs = []`），本条记录 `o` 照常发送。
  - **读法 C（`to` 在 `records` 中为 `Ineligible` 时连带使所在基础材料失败或候选包排除）**：`Undetermined` 返回 `ProjectionUnavailable`；`Ineligible` 则基础材料返回 `BaseRequestFailed(Ineligible)`（候选包则整包 `Excluded`）。
- **日常话**：
  - 读法 A：只要输入的关系表里留着「回复了 `r9`」这条接续线，哪怕 `r9` 已禁止使用，它的记录编号仍会作为引用发给模型（由上游负责撤回禁用连线）。
  - 读法 B：`r9` 权限未决时她先不回话；`r9` 已禁止使用时只隐去指向 `r9` 的引用编号，她照常回话。
  - 读法 C：只要你这一句是在回复一条已禁用消息，她这一轮也无法回话。
- **你的建议（候选）**：选**读法 A**（候选）。与 ADR §1.3「不读取目标记录」及 `r9` 不在 `records` 时的行为一致，避免同一个 `r9` 仅因是否碰巧加载进 `records` 切片而产生不同输出；边级权限由上游关系治理统一负责。

---

## Q6：候选槽 `member.sources` 中的记录 ID 不在 `input.records` 时（及 `frontier_complete = false` 与基础记录缺失并存时），返回 `ProjectionUnavailable` 还是 `Err(InvalidInput)`

- **ADR 原文**：
  - §1.3：「查询完整性不能确认、正文依赖端点丢失或关系语义未知时，按 §3.2 处理。」「必要端点缺失或资格未决时，整个投影不可得」
  - §3.2：「包关系未知、原文丢失、查询完整性不能确认 | 整个投影不可得，结束本次组装」
- **具体例子**：
  - 例 1：`frontier_complete = true`，基础材料齐全；候选槽 `S1`（`h = [0.6]`）的 `member.sources = [s_missing]`，而 `s_missing` 不在 `input.records` 中。
  - 例 2：`frontier_complete = false`（前沿查询本身未完成），且 `required_facts = [f_missing]` 因前沿截断未出现在 `input.records` 中。
  - 歧义原因：本卡 §5.1 只把 `latest`、`dialogue`、`required_facts`、`protocol_cited` 不在 `records` 判为 `Err(InvalidInput)`，未提 `slot.members[*].sources`；而 §5.4 第 1 步写「把 seeds 放进集合和工作队列。队列按 `(h_pos, id)` 升序取出」，若种子 `s_missing` 不在 `records` 中就没有 `h_pos`，§5.4 仅在第 2 步关系展开写了 `MissingRecord`。同时例 2 中 §5.1 先于 §5.2 运行会把前沿未查全报成调用方传参错误 `Err(InvalidInput)`。
- **可选的读法**：
  - **读法 A（§5.1 保持不变；候选槽种子缺失在 §5.4 入口判 `Ok(ProjectionUnavailable(MissingRecord))`）**：基础材料四类 ID 缺失一律在 §5.1 返 `Err(InvalidInput)`（即使 `frontier_complete = false`，调用方也须传齐已锁定的基础材料记录）；候选槽（`s > 0`）的 `seeds` 中有不在 `records` 的 ID 时，按 `RecordId` 字节序取最先缺失者返回 `Ok(Outcome::ProjectionUnavailable(Unavailable::MissingRecord(id)))`（`s == 0` 的退出槽不查）。
  - **读法 B（在读法 A 基础上，`frontier_complete = false` / `decomposition_unknown = true` 提前到基础记录存在性检查之前）**：先校验 `h` 有限性、`records` 同 ID 一致性、槽 `basis` 唯一性；随后立即按 §5.2 检查 `frontier_complete` 与 `decomposition_unknown`（例 2 返回 `Ok(ProjectionUnavailable(FrontierIncomplete))`）；通过后再校验基础记录存在性与 `Partial` 区间。
  - **读法 C（候选槽 `member.sources` 不在 `records` 也归入 §5.1 返回 `Err(InvalidInput)`）**：例 1 与例 2 均返回 `Err(InvalidInput)`。
- **日常话**：
  - 读法 A：某件候选旧事的原始记录在可见历史里丢失时，系统正常产出「投影不可得（缺失记录）」回执并停发回复；例 2 则要求上层在传参前自己先处理未查全的基础材料。
  - 读法 B：除例 1 产出「投影不可得（缺失记录）」外，例 2 在历史查询未完成导致缺记录时也正常产出「投影不可得（前沿未确认）」回执，不当成程序调用 bug。
  - 读法 C：候选旧事找不到原始记录时直接报调用参数错误，不产出 ADR §3.3 的「投影不可得」回执。
- **你的建议（候选）**：选**读法 A**（候选；若上游在 `frontier_complete = false` 时原样把不完整切片传入 `assemble`，则选**读法 B**）。候选槽源记录丢失属于 ADR §1.3/§3.2 明确规定的运行期「原文丢失 → 投影不可得」，不应归为 `InvalidInput`。

---

## Q7：基础请求已失败（`Ineligible` 或 `OverCapacity`）与可选候选槽 `Unavailable` 并存时的回执优先级

- **ADR 原文**：
  - §2.1：「**优先级是：来源可用及更正完整 → 基础对话/任务材料 → 可选来由。** 按最终四段请求与输出预留预算计量基础材料。按 U2 决定，必需部分装不下就将本轮记为失败，不发送」
  - §2.2：「**基础请求可用后**，按 §1.2 的排序逐包试放。」
  - §2.3：「从容量失败请求恢复须用 r4 规则和实际 D 范围建立新绑定，再计量完整必要材料。」
  - §3.3：「回执分别记录“无合格经历”“可选包超预算”“投影不可得”“基础请求失败”。前两者可有合法请求，后两者本轮不发送」
- **具体例子**：
  - `D = [d_huge]` 使基础请求 `m + output_reserve > capacity`（或 `latest` 本身为 `Ineligible`），同时候选槽 `S1`（`h = [0.5]`）有 `merge_undecided = true`（或包内含 `Undetermined` 记录）。
  - 歧义原因：本卡 §5.7 先检查候选槽 `Unavailable`（第 3 步），再检查基础材料 `Ineligible`（第 4 步）和 §5.8 基础容量（`OverCapacity`）；而 ADR §2.2 写「基础请求可用后，按 §1.2 的排序逐包试放」，且上层恢复逻辑依赖区分 `BaseRequestFailed(OverCapacity)`（可通过缩减 `D` 重绑恢复）与 `ProjectionUnavailable`。
- **可选的读法**：
  - **读法 A（按本卡 §5.7–§5.8 顺序：全部 `Unavailable` 优先于 `BaseRequestFailed`）**：上例返回 `ProjectionUnavailable(MergeUndecided(S1))`。
  - **读法 B（基础请求先闭环：5.2 → 5.5 基础 `Unavailable` → 基础 `Ineligible` → 5.8 基础 `OverCapacity` → 5.6 候选槽）**：基础材料自身的缺失/未知/未决仍返回 `ProjectionUnavailable`，但若基础材料已知 `Ineligible` 或基础请求已超容量，直接返回 `BaseRequestFailed(..)`，不被后面的可选候选槽掩盖。
- **日常话**：
  - 两种读法下她这一轮都不回话；区别在于故障回执：读法 A 记为「投影不可得」，读法 B 记为「基础请求失败（如当前对话已超窗）」，后者能直接告诉外层「就算不带任何旧事，光当前对话也装不下了」。
- **你的建议（候选）**：选**读法 A**（候选）。与 ADR §2.1「来源可用及更正完整 → 基础对话/任务材料 → 可选来由」把可得性放在第一顺位一致，且无需改动本卡 §5.7 已写明的步骤次序。

---

## Q8：排在 `FirstOverflow` 之后的 `Ineligible` 候选包，在 `Binding.packs` 中标 `Excluded` 还是 `NotTried`

- **ADR 原文**：
  - §1.3：「必要来源确定不合格（无真实外根、已禁止使用等）时，候选包整体排除并记录缺口」
  - §2.2：「整包放得下就接纳；首个放不下就停止，后面的包不再试放。」
  - §2.4：「来源与装包 | ……排除/未知原因、实际接纳集合、首个溢出包与停止原因」
- **具体例子**：
  - 候选槽按排名为 `S1`（合格但超长，装不下）和 `S2`（含 `Ineligible` 记录 `r_bad`）。
  - 歧义原因：本卡 §5.7 第 5 步已先将 `S2` 标为 `Excluded { record: Some(r_bad), reason }`（§4 也写 `packs: 全部候选，按排序先后（含 Excluded）`），但 §5.9 在 `S1` 放不下时又写「放不下 → `FirstOverflow`，**后面所有包标 `NotTried`，停止**」，字面上会把 `S2` 的 `Excluded` 覆盖成 `NotTried`。
- **可选的读法**：
  - **读法 A（保留 `Excluded`）**：`FirstOverflow` 之后仅将其后**非 `Excluded`** 的合格候选包标为 `NotTried`；`Binding.packs` 为 `[S1: FirstOverflow, S2: Excluded { record: Some(r_bad), .. }]`。
  - **读法 B（其后全部标 `NotTried`）**：`S1` 触发 `FirstOverflow` 后停止装包，排在 `S1` 后面的所有候选槽一律标为 `NotTried`；`Binding.packs` 为 `[S1: FirstOverflow, S2: NotTried]`。
- **日常话**：
  - 两种读法下发给模型的内容和回执完全相同（都不带旧事，回执「可选包超预算」）；区别只在本地核查记录里，排在后面的禁用旧事是保留「因禁用排除」的记录，还是统一记为「排在超长包后面未尝试」。
- **你的建议（候选）**：选**读法 A**（候选；若 Q4 选了惰性求包的读法 B，则联动选这里的**读法 B**）。§5.7 已完成资格判定，保留 `Excluded` 符合 ADR §1.3「记录缺口」与 §2.4「排除原因」。

---

## Q9：同一闭包中同时存在多种 `Unavailable` 条件或多条异常关系时的确定性返回次序

- **ADR 原文**：
  - §1.3：「把初始原记录身份入集合与工作队列；每次取队列中 H 位置最早者，同位置按原记录身份规范字节序……查询完整性不能确认、正文依赖端点丢失或关系语义未知时，按 §3.2 处理。」
  - §3.2：「同一绑定重复组装、回放与缓存命中变化 | 本地输入字节相同」
- **具体例子**：
  - 基础材料含 `x`，`input.relations` 同时含 `Relation { from: x, to: u, kind: Unknown }`、`Relation { from: x, to: m2, kind: BodySource }`、`Relation { from: x, to: m1, kind: BodySource }`（`m1`、`m2` 均不在 `records` 中，`m1 < m2`），把 `input.relations` 倒序后再调用 `assemble`。
  - 歧义原因：本卡 §6.5 要求打乱/倒序输入列表结果一致且 `Outcome` 确定唯一，但 §5.4 第 2 步「扫描所有与 x 相关的关系」未规定多条异常关系之间的择一次序；若按 `input.relations` 切片顺序扫描，倒序后会返回不同的 `Unavailable` 变体或不同的 `RecordId`。
- **可选的读法**：
  - **读法 A（未知关系优先于缺失端点，内部按规范字节序）**：在 `closure(seeds)` 中：
    1. 若 `seeds` 含不在 `records` 的 ID，按 `RecordId` 字节序最小者返回 `MissingRecord(id)`；
    2. 按 `(h_pos, id)` 升序弹出 `x` 时，在与 `x` 相关的关系中，先检查 `Unknown`（按 `(from, to)` 字节序最小者返回 `UnknownRelation`），再检查 `BodySource`（`from == x` 且 `to` 缺失，按 `to` 字节序最小者返回 `MissingRecord`），再检查 `Correction`（`to == x` 且 `from` 缺失，按 `from` 字节序最小者返回 `MissingRecord`）；
    3. 队列空后，在闭包集合中按 `(h_pos, id)` 升序取第一条 `Undetermined` 返回 `EligibilityUndetermined(id)`。
    对上例无论 `relations` 如何排列，均返回 `UnknownRelation { from: x, to: u }`（若无 `Unknown` 则稳定返回 `MissingRecord(m1)`）。
  - **读法 B（缺失端点优先于未知关系，内部按规范字节序）**：第 2 步先查 `BodySource` 缺失（按 `to` 字节序）、再查 `Correction` 缺失（按 `from` 字节序）、最后查 `Unknown`（按 `(from, to)` 字节序）；其余同读法 A。对上例稳定返回 `MissingRecord(m1)`。
- **日常话**：
  - 两种读法下她这一轮都不回话、回执大类都是「投影不可得」；区别只在同时撞上「关系分不清」和「缺原文」（或同时缺多条原文）时，具体错误详情先报哪一个，并保证不管输入顺序怎么变，每次报的原因都完全一致。
- **你的建议（候选）**：选**读法 A**（候选）。关系类型本身未知时无法确认边的语义与方向，先报 `UnknownRelation` 再报已知依赖边的 `MissingRecord` 更自然，且完全消除 `input.relations` 切片顺序的影响。
