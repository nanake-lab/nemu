# IMPL-CAUSE-PROJ-01：来由投影组包器（ADR 0042 的确定性纯函数）

- **线：** 交付线（ADR 0043）
- **建议执行者：** Antigravity Orchestrator (Carbon)
- **分支：** `task/IMPL-CAUSE-PROJ-01`，从 `origin/main` 新建
- **目录：** `core/crates/task-layer/src/cause_projection/`（新建）；`core/crates/task-layer/src/lib.rs` 只加一行 `pub mod cause_projection;`；`tasks/requests/IMPL-CAUSE-PROJ-01.md`（新建，内容为本卡原文）；`reports/IMPL-CAUSE-PROJ-01/`
- **不碰：** 其他 crate、`core/Cargo.toml`、`core/Cargo.lock`、`core/contracts/`、所有 ADR 与规格、`task-layer/src/projection/`（属 P6-04）、`task-layer/src/cognition/`（属 P5-38）
- **依赖：** 不加任何依赖，只用 `std`

## 1. 要做什么

把 ADR 0042 §1.2、§1.3、§2 写成一个**确定性纯函数**：输入一组抽象的记录、关系、槽和预算，输出这一轮要发给说话模型的四段请求计划（P、D、E、o）、本地绑定和回执；或者输出「不发送」及原因。

本卡**只做函数和测试**。不接真实的 H、F 槽、tokenizer、对话计划、存储或 API；不发布投影、关系映射、计量资源。这些是后续接线卡的事。ADR 0042 仍是 PROPOSED r4，本卡不改它。

本卡分两个阶段：**第 0 步只读不写代码**，交一份「没写死的地方」清单，等用户定夺；用户定了以后，再做第 3–9 节。

## 第 0 步：先找出没写死的地方（不写代码）

1. 读完第 2 节列的文件和本卡第 4–6 节。
2. 找出所有这样的地方：照 ADR 0042 和本卡，同一份输入可以有两种以上合理的结果，而 ADR 和本卡都没说选哪个。包括本卡第 5 节的规则与 ADR 原文不一致、或者本卡没写到的情况。只列会改变**输出结果**的地方（发什么、不发什么、回执是什么）；代码怎么组织、变量怎么命名这类不列。
3. 每一条写：
   - **ADR 原文**：引出处（小节号）和原句；
   - **具体例子**：一份小输入（几条记录、什么关系），说明它为什么有歧义；
   - **可选的读法**：两种或以上，每种对这个例子给出什么结果；
   - **日常话**：用不懂代码的人也能看懂的话，说清各读法下 Nemu 会怎么表现（比如「她这一轮不回话」「这件旧事不会被提起」）；
   - **你的建议**：选哪种，理由一两句。标为「候选」，不替用户定。
4. 写进 `reports/IMPL-CAUSE-PROJ-01/OPEN.md`，按影响大小排序。条数不设上下限，找到多少写多少，确实没有就写「没有」。
5. 开 draft PR（标题见第 9 节），只含 `OPEN.md` 和 `tasks/requests/IMPL-CAUSE-PROJ-01.md`，加 `needs-user`，在 PR 里写「请定夺 OPEN.md」，然后停下。**用户定夺之前不写任何代码。**

用户在 PR 里回复决定后，把决定原样追加到 `OPEN.md` 末尾，再按决定继续第 3 节起的工作，同一个分支、同一个 PR。

## 2. 先读

1. `AGENTS.md` 全文（尤其第 2、6、9 节）。
2. `docs/adr/0042-cause-projection.md` 的 §0、§1.2、§1.3、§2.1–§2.4、§3.2、§3.3、§5.1、§5.3。本卡的每条规则都来自这几节；本卡与 ADR 有出入的地方，写进第 0 步的 `OPEN.md`，不要自己选。
3. `core/contracts/law_state.proto` 第 1–80 行，只为了知道 h 的坐标是有限的有符号 `double`。本卡不引用这些类型。

其他文件不用读。

## 3. 文件布局（建议，可以自己调整）

下面是建议的拆法。你觉得别的拆法更清楚可以改，在报告里说一句为什么。

```
core/crates/task-layer/src/cause_projection/
  mod.rs        公共 API：assemble()、RequestCodec，re-export 类型；模块文档写明对应 ADR 0042 哪几节
  types.rs      第 4 节的全部输入、输出类型
  validate.rs   第 5.1 步
  rank.rs       第 5.3 步（ADR §1.2）
  closure.rs    第 5.4 步（ADR §1.3）
  assemble.rs   第 5.5–5.9 步（ADR §2）
  tests/
    mod.rs      TestCodec 和构造测试输入的小工具
    rank.rs
    closure.rs
    golden_2_3.rs   ADR §2.3 的五个例子
    accept_3_2.rs   ADR §3.2 中本卡范围内的各行
    determinism.rs
```

`mod.rs` 里用 `#[cfg(test)] mod tests;`。代码注释用英文，每个函数的文档注释标出对应的 ADR 小节号。

**硬性要求：**
- 只用 `BTreeMap` / `BTreeSet` / `Vec`，模块里不出现 `HashMap`、`HashSet`。
- 不读时钟、不用随机数、不做 IO、不 `unsafe`、不 `unwrap()`/`expect()`（测试代码除外）。
- 函数只拿 `&` 引用，不改任何输入。
- `RequestPlan` 和 `Binding` 里不出现 `f64` 字段，也不出现 h、排序分数或名次（ADR §5.1、§5.3：这些永不发出）。

## 4. 类型（语义不能变；字段名和结构可以按你的判断调整）

```rust
/// Canonical identity bytes. Ord = byte order (ADR 0042 §1.3 "规范字节序").
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordId(pub Vec<u8>);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasisId(pub Vec<u8>);
/// Commit position in H. Smaller = earlier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HPos(pub u64);

pub enum Eligibility {
    Eligible,
    Ineligible { reason: String },   // 已确定不合格：无真实外根、已禁止使用等
    Undetermined,                    // 资格未决
}

pub struct Record {
    pub id: RecordId,
    pub content_version: Vec<u8>,
    pub h_pos: HPos,
    pub body: Vec<u8>,               // 原始 payload，原样，不加工
    pub eligibility: Eligibility,
}

pub enum RelationKind {
    BodySource,    // 原文来源 / 正文引用 / 转述。from = 引用方，to = 被引用的原文
    Continuation,  // 接续（回复上一条、会话父消息）。from = 回复，to = 被回复
    Correction,    // 事实更正 / 证伪 / 撤回 / 权限关系。from = 更正记录，to = 被更正记录
    LocalOnly,     // F 作用、解释、前态、资源链接、合并记录等，只用于本地重放与定位
    Unknown,       // 类型映射分不清
}
pub struct Relation { pub from: RecordId, pub to: RecordId, pub kind: RelationKind }

pub struct Member {
    pub basis: BasisId,
    /// 该成员首次「新依据」及截至 n 全部已生效「目标更新」所指的原始事件来源（全收）
    pub sources: Vec<RecordId>,
}
pub struct Slot {
    pub basis: BasisId,
    pub h: Vec<f64>,                 // 当前残量的全部坐标（nature + learned），有限值
    pub first_f_pos: HPos,           // 首次 F 作用在 H 中的位置
    pub members: Vec<Member>,        // 本槽及传递合并进来的全部成员（调用方已按生效治理解析）
    pub merge_undecided: bool,       // 有尚未定案的合并关系
}

pub enum Delivered { Full, Partial { start: u64, end: u64 } }   // body 的字节区间 [start, end)
pub struct DItem { pub id: RecordId, pub delivered: Delivered }

pub struct Budget { pub capacity: u64, pub output_reserve: u64 }

pub struct AssemblyInput {
    pub protocol: Vec<u8>,               // P，不透明字节，整段使用
    pub protocol_cited: Vec<RecordId>,   // P 引用的事实原文
    pub dialogue: Vec<DItem>,            // D：既有对话计划选定的条目，按对话顺序
    pub latest: RecordId,                // o
    pub required_facts: Vec<RecordId>,   // 原任务必需的事实查询结果
    pub records: Vec<Record>,            // 绑定事实前沿内可见的全部记录
    pub relations: Vec<Relation>,
    pub frontier_complete: bool,         // 查询完整性能否确认
    pub slots: Vec<Slot>,
    pub decomposition_unknown: bool,     // 旧迁移的分解未知（ADR §1.2、§3.2）
    pub budget: Budget,
}
```

输出：

```rust
pub struct PresentedD { pub id: RecordId, pub delivered: Delivered, pub continuation_refs: Vec<RecordId> }
pub struct PresentedE { pub id: RecordId, pub continuation_refs: Vec<RecordId> }

pub struct RequestPlan {
    pub protocol: Vec<u8>,
    pub dialogue: Vec<PresentedD>,
    pub experience: Vec<PresentedE>,     // E，按 (h_pos, id) 升序
    pub latest: RecordId,
    pub latest_continuation_refs: Vec<RecordId>,
    pub strict: bool,                    // 永远 false（TEXT_PROJECTED）
}

pub enum PackFate {
    Accepted { new_records: Vec<RecordId> },   // 新增进 E 的记录，可以为空
    FirstOverflow,
    NotTried,
    Excluded { record: Option<RecordId>, reason: String },
}
pub struct PackEntry { pub basis: BasisId, pub records: Vec<RecordId>, pub fate: PackFate }  // records = 完整闭包，按 (h_pos, id)

pub struct Binding {
    pub base_records: Vec<RecordId>,     // 基础闭包，按 (h_pos, id)
    pub packs: Vec<PackEntry>,           // 全部候选，按排序先后（含 Excluded）
    pub accepted: u32,                   // K
    pub measured: u64,                   // 最终请求的计量值
}

pub enum SentNote { AllFit, OptionalOverBudget, NoEligibleExperience }

pub enum Unavailable {
    FrontierIncomplete,
    DecompositionUnknown,
    MergeUndecided(BasisId),
    UnknownRelation { from: RecordId, to: RecordId },
    MissingRecord(RecordId),
    EligibilityUndetermined(RecordId),
}
pub enum BaseFailure {
    Ineligible { record: RecordId, reason: String },
    OverCapacity { measured: u64, output_reserve: u64, capacity: u64 },
}

pub enum Outcome {
    Sent { plan: RequestPlan, bytes: Vec<u8>, binding: Binding, note: SentNote },
    ProjectionUnavailable(Unavailable),      // 回执「投影不可得」，不发送
    BaseRequestFailed(BaseFailure),          // 回执「基础请求失败」，不发送
}

pub enum InvalidInput { /* 第 5.1 步列出的每种情况一个变体 */ }

pub trait RequestCodec {
    /// 把计划序列化成请求字节。records 用来取正文。
    fn encode(&self, plan: &RequestPlan, records: &RecordTable) -> Vec<u8>;
    /// 计量完整请求（含包装、来源、更正）。
    fn measure(&self, bytes: &[u8]) -> u64;
}

pub fn assemble<C: RequestCodec>(input: &AssemblyInput, codec: &C) -> Result<Outcome, InvalidInput>;
```

`RecordTable` 是对 `BTreeMap<RecordId, Record>` 的只读包装，提供 `get(&RecordId) -> Option<&Record>`。所有类型 derive `Clone, Debug, PartialEq`（含 f64 的只 derive `PartialEq`，其余加 `Eq`）。

`InvalidInput` 是调用方的错，和三种回执分开；`Outcome` 才是 ADR §3.3 的回执。四个回执与输出的对应：

| ADR §3.3 回执 | 输出 |
| --- | --- |
| 无合格经历 | `Sent { note: NoEligibleExperience }` |
| 可选包超预算 | `Sent { note: OptionalOverBudget }` |
| 投影不可得 | `ProjectionUnavailable(..)` |
| 基础请求失败 | `BaseRequestFailed(..)` |

## 5. 算法（按顺序实现，每一步都写进代码注释）

### 5.1 校验（返回 `Err(InvalidInput)`）
- 任一 `h` 坐标不是有限值（NaN、±inf）。
- 同一 `RecordId` 出现多次且 `content_version`、`h_pos`、`body`、`eligibility` 不全相同（ADR §1.3：同一不可变身份对应不同内容是异常）。全相同则当一条。
- 两个槽 `basis` 相同。
- `latest`、`dialogue`、`required_facts`、`protocol_cited` 中的 id 不在 `records` 里（基础材料由调用方锁定，必须给全）。
- `Partial { start, end }` 中 `start > end` 或 `end > body.len()`。

### 5.2 整体可得性
- `frontier_complete == false` → `ProjectionUnavailable(FrontierIncomplete)`。
- `decomposition_unknown == true` → `ProjectionUnavailable(DecompositionUnknown)`。

### 5.3 排序（ADR §1.2，U1）
- 每个槽算 `s = max_i |h_i|`；`h` 为空时 `s = 0`。
- `s == 0.0` 的槽退出（`-0.0` 也是 0）。其余都是候选，不看正负、不看是哪个乎，不加权，不设配额。
- 排序：`s` 降序（用 `f64::total_cmp`）→ `first_f_pos` 升序 → `basis` 字节序升序。

### 5.4 闭包（ADR §1.3）

写一个函数 `closure(seeds) -> Result<ClosureResult, Unavailable>`，基础材料和每个来源包都用它：

1. 把 seeds 放进集合和工作队列。队列按 `(h_pos, id)` 升序取出。
2. 取出记录 x 后，扫描所有与 x 相关的关系：
   - `BodySource` 且 `from == x`：把 `to` 加入（未访问过才入队）。`to` 不在 `records` → `MissingRecord(to)`。
   - `Correction` 且 `to == x`：把 `from` 加入（未访问过才入队）。`from` 不在 `records` → `MissingRecord(from)`。
   - `Continuation` 且 `from == x`：只记下 `to` 作为 x 的直接接续引用，**不加入集合，不读 to，不追祖先**。`to` 不在 `records` 也不算缺失。
   - 碰到 `Unknown` 关系 → `UnknownRelation { from, to }`（ADR §1.3：无法区分的关系按未知处理）。
   - `LocalOnly` 不展开。
3. 已访问的不再展开，循环引用自然终止。不设深度、次数或 token 上限。
4. 队列空后，检查集合中每条记录的 `eligibility`：有 `Undetermined` → `EligibilityUndetermined(id)`。
5. 返回：集合（按 `(h_pos, id)` 排序）；第一条 `Ineligible` 记录（按 `(h_pos, id)` 最先者）及原因，没有就是 `None`。

### 5.5 基础材料（ADR §2.1）
- seeds = `dialogue` 的全部 id ∪ `{latest}` ∪ `required_facts` ∪ `protocol_cited`。不展开它们所属槽的目标史。
- 跑 5.4 的闭包。

### 5.6 候选包
- 对 5.3 的每个候选槽，按排序先后：
  - `merge_undecided` → `MergeUndecided(basis)`。
  - seeds = 全部成员 `sources` 的并集，跑 5.4 的闭包。

### 5.7 结果判定的先后
1. 5.2；
2. 5.5 的 `Unavailable`；
3. 5.6 候选包的 `Unavailable`；
4. 5.5 中有 `Ineligible` → `BaseRequestFailed(Ineligible)`；
5. 5.6 中有 `Ineligible` 的包 → 该包 `Excluded { record: Some(id), reason }`，继续；
6. 进入 5.8。

### 5.8 呈现规则与基础请求计量（ADR §2.1）
- **D 去重：** 按原顺序，删掉 id 等于 `latest` 的条目，同一 id 只保留第一次出现（保留那一次的 `delivered`）。
- **已完整呈现：** `presented_full` = `{latest}` ∪ D 中 `delivered == Full` 的 id。
- **E：** （基础闭包 ∪ 已接纳包的闭包）减去 `presented_full`；按 `(h_pos, id)` 升序。
- **接续引用：** D、o、E 中每条记录，`continuation_refs` = 它作为 `from` 的全部 `Continuation` 边的 `to`，去重后按 id 字节序排列。
- **计量：** 用 E_base 建计划，`bytes = codec.encode(..)`，`m = codec.measure(&bytes)`。用 `checked_add` 算 `m + output_reserve`，溢出或 `> capacity` → `BaseRequestFailed(OverCapacity)`。等于 capacity 算放得下。

### 5.9 装包（ADR §2.2，U2）
按排序先后逐包：
- `Excluded`：跳过，继续下一个（这不是「放不下」）。
- 否则，把该包集合并入当前 E（按 5.8 的规则重算 E 和计划），重新 encode、measure **完整四段请求**。
  - 放得下 → `Accepted { new_records }`，new_records = 这次新增进 E 的记录（按 `(h_pos, id)`，可以为空；全在基础里的包零新增也接纳）。
  - 放不下 → `FirstOverflow`，后面所有包标 `NotTried`，停止。不试后面更短的包。
- 结束后用最终 E encode 一次，得到 `bytes` 和 `measured`。
- `note`：没有任何非 `Excluded` 的候选包 → `NoEligibleExperience`；出现了 `FirstOverflow` → `OptionalOverBudget`；否则 `AllFit`。
- `plan.strict = false`。

## 6. 测试（全部必须有，写在第 3 节的文件里）

**TestCodec**（`tests/mod.rs`）：格式自定，要求确定、人能读、按 P、D、E、o 的顺序写出每条记录的 id、呈现的正文和接续引用；`measure` = 字节长度。

**预算不写魔数。** 需要「恰好放得下 / 差 1 放不下」的测试，先用 TestCodec 对期望的计划算出长度，再据此设 `capacity`。

### 6.1 排序（`tests/rank.rs`）
- `h = (-0.9, 0.1)` 排在 `(0.5, 0.5)` 前（取绝对值最大，不求和）。
- `s` 相同时 `first_f_pos` 小的在前；再相同按 `basis` 字节序。
- `h` 全为 0、含 `-0.0`、为空的槽不进候选。
- `h` 含 NaN 或 inf → `Err(InvalidInput)`。

### 6.2 闭包（`tests/closure.rs`）
- 循环 `BodySource` r0↔r1：各一次，能终止。
- 更正的更正：c1 更正 s，c2 更正 c1 → {s, c1, c2}。
- s 自己更正别人的记录 y（`Correction from = s, to = y`）：y 不进集合。
- `LocalOnly` 两个方向都不展开。
- 两个不同 id、body 完全相同的记录都保留。
- 同 id 不同 body → `Err(InvalidInput)`。

### 6.3 ADR §2.3 五例（`tests/golden_2_3.rs`）
每例另设无关的基础材料：`D = [x1]`（Full），`latest = x2`，h_pos 放在最后（100、101），与例子的记录无关系。

1. **正文引用与接续分开。** h_pos：r0=10, r1=20, r9=25, r2=30。关系：r2→r1 BodySource，r1→r0 BodySource，r2→r9 Continuation。一个槽，sources={r2}。期望：包 = [r0, r1, r2]；E = [r0, r1, r2]；r2 的 `continuation_refs = [r9]`；E 里没有 r9。变体 a：再加 r0→r1 BodySource（循环），结果不变。变体 b：`records` 里没有 r9，结果不变。
2. **目标更新。** t0=10, t1=20, t2=30, i=40, j=50。槽 sources={t0, t1, t2}；i→t0 LocalOnly，j→t2 LocalOnly。期望 E = [t0, t1, t2]，没有 i、j。
3. **别名合并。** q=1, a1=2, b1=3, a2=4, M=5。一个槽，成员 c1（sources {a1, a2}）和 c2（sources {b1}）；a1→q、b1→q BodySource；M→a1、M→b1 LocalOnly。期望：一个包，E = [q, a1, b1, a2]，q 一次，K=1，没有 M。
4. **更正超预算。** d0=10, c=20。D = [d0]（Full），c→d0 Correction，没有槽。capacity 设为「P、D、o 恰好放得下，加上 E={c} 就差 1」。期望 `BaseRequestFailed(OverCapacity)`，不是丢掉 c 发送。变体：d0 不在 D，只在一个槽的包 {d0, c} 里，基础请求放得下而这个包放不下 → `Sent`，K=0，`OptionalOverBudget`，E 为空。
5. **长对话。** r0..r6，r(k)→r(k−1) Continuation，h_pos 依次递增。D = [r4, r5]（Full），latest = r6，没有槽。期望：`Sent`，`NoEligibleExperience`，E 空，K=0；r4 的 refs=[r3]，r5 的 refs=[r4]，o 的 refs=[r5]；r0–r3 不出现在 E。变体 a：r6→q BodySource（q 是 h_pos=1 的旧原文）→ E = [q]。变体 b：在变体 a 上把 capacity 设成差 1 → `BaseRequestFailed(OverCapacity)`。

### 6.4 ADR §3.2 中本卡范围内的各行（`tests/accept_3_2.rs`）
- **两包共享原文与更正：** s=10, a=20, b=30, c=40；c→s Correction；槽 S1 sources {s, a}，h 更大；槽 S2 sources {s, b}。都放得下 → K=2，E = [s, a, b, c]，s 和 c 各一次。
- **包的一部分已在对话：** D = [s]（Full），s=10, a=20, c=30，c→s Correction；槽 sources {s, a}。期望：基础 E = [c]；接纳后 E = [a, c]，s 只在 D。
- **最大包放不下、后面短包放得下：** S1（排第一）的原文很长，S2 很短。capacity 设成基础 + S2 放得下、基础 + S1 放不下。期望：K=0，S1 `FirstOverflow`，S2 `NotTried`，`OptionalOverBudget`。
- **基础材料恰好用满：** capacity = 基础请求长度 + output_reserve。槽需要新增原文 a → K=0，`OptionalOverBudget`，发送基础请求。变体：capacity 再减 1 → `BaseRequestFailed`。变体：槽的包全在基础里（零新增）→ 接纳，K=1。
- **不可得：** 下列每一种单独一个测试，都得到对应的 `ProjectionUnavailable`：`frontier_complete=false`；`decomposition_unknown=true`；某候选槽 `merge_undecided`；包内记录碰到 `Unknown` 关系；`BodySource` 端点缺失；包内记录 `Undetermined`。
- **已知不合格：** 包内有 `Ineligible` → 该包 `Excluded`，下一个包照常试放；基础材料里有 `Ineligible` → `BaseRequestFailed(Ineligible)`；全部候选都 `Excluded` → `NoEligibleExperience`。
- **第 0 步定下的每一条：** 每条决定至少一个测试，按决定的结果断言。
- **D 含 o：** D 里有 latest 的 id → 从 D 删掉，只在 o 出现一次。
- **永不严格：** 所有 `Sent` 的 `plan.strict == false`。

### 6.5 确定性（`tests/determinism.rs`）
- 用 6.4 第一条的输入，`assemble` 调两次，`Outcome` 完全相等，`bytes` 逐字节相等。
- 把 `records`、`relations`、`slots`、每个槽的 `members` 和 `sources` 全部倒序后再调，`bytes` 与 `Binding` 和原来完全相等。
- 调用前 `clone` 输入，调用后与原输入相等。

## 7. 运行与检查（Windows，在 `core/` 下）

```powershell
$env:CARGO_HOME='D:\Dev\cache\cargo'
$env:CARGO_TARGET_DIR='D:\Dev\cache\cargo-target\IMPL-CAUSE-PROJ-01'
cargo fmt -p task-layer -- --check
cargo clippy -p task-layer --all-targets --locked -- -D warnings
cargo test -p task-layer --locked
```

三条全过才算完成。`Cargo.lock` 不能有变化（`git diff --exit-code core/Cargo.lock`）。不跑 GPU，不下载模型。

## 8. 报告 `reports/IMPL-CAUSE-PROJ-01/REPORT.md`

照 `docs/agents/report-template.md`。正文包括：
- 一张表：ADR 0042 §2.3 五例和 §3.2 各行 → 对应的测试函数名 → 通过/未通过。
- 第 7 节三条命令的实际输出末尾几行（原样贴）。
- **第 0 步的结果：** 链接 `OPEN.md`，逐条写用户的决定和对应的测试函数名。
- 若调整了第 3 节的文件布局或第 4 节的类型，说明改了什么、为什么。
- **本卡不覆盖、留给接线卡的 §3.2 各行：** 两次组装间的撤回/权限变化；API 拒绝或自动裁切；请求取消与部分投递的回执；U6 的等待、失败、积压与后补解释；上下文位置的语义实测。

## 9. 交付与停止

1. 把本卡原文存为 `tasks/requests/IMPL-CAUSE-PROJ-01.md`，第 0 步就提交。
2. PR 标题 `feat(IMPL-CAUSE-PROJ-01): ADR 0042 来由投影组包器（纯函数与验收测试）`，draft，加标签 `needs-user`。第 0 步结束时写「请定夺 OPEN.md」并停下；用户定夺后同一个 PR 继续。
3. 代码完成后，PR 描述补上：做了什么、第 0 步各条的决定、三条命令的结果。写「请审查」，然后停下。不合并，不取下一张卡。

**遇到以下情况停下来写 `BLOCKED.md`，不要自己换方案：** ADR 的某条规则与本卡第 5 节对不上；某个测试的期望值按 ADR 算出来和本卡写的不一样；需要加依赖或改目录外的文件。
