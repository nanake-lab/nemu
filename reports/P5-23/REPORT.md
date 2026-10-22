# P5-23 规律引擎 G：倾向的证据计数 — 报告

## 结果

部分完成（见 BLOCKED.md）。G 的全部阶段都已实现并测试，卡片列出的验收都通过：

- 单倾向的递推与典型幅度 r_k；
- 支持、强度、份额；
- 证据组阶段，包括后来的成员、EV5 回溯分摊和补做的解释；
- 关于人的倾向；
- 容量保险丝，包括移出后的重新分摊；
- L 条目。

LAW-1 至 LAW-10 的移植 0 违例，状态数与 `results.txt` 逐项一致。

只剩一步：契约卡 P5-32（ADR 0016，PR #52）合并之后，把 `law/pending.rs` 的临时结构换成生成的类型。按你 2026-10-22 的决定，在那之前本卡保持草稿。

## 交付物

- `core/crates/core-kernel/src/law/`（新）；`core-kernel/src/lib.rs` 只追加一行 `pub mod law;`：
  - `engine.rs`：G 的入口 `g(RState, &ThetaView, &Step, &LawContext)`，阶段按 D2：建立候选并做别名归并，建立第一次涉及的人的关系倾向，按 H 次序处理证据组，执行容量保险丝，生成 L 条目；
  - `plan.rs`、`group.rs`：一组证据（D3）。先算经历组或教导组对每个倾向的 S、强度、μ、p_c，再分份额；新分配按 D4 记入，已有分配的值变了就原地更新，并重放该倾向；
  - `replay.rs`：r_k 的递推（M、U），从初值按序号重放一个倾向（D5），存储一致性检查；
  - `pending.rs`：P5-32 合并之前的临时结构（D12）。字段与 P5-32 的后继消息一一对应，只在 `law/` 内部使用，不序列化、不持久化、不跨进程。文件里注明「P5-32 合并后换成 IDL 生成的类型」；
  - `recurrence.rs`：单倾向的计数递推（D4）、初值、s_k 与 R⁺；
  - `support.rs`：支持 S、强度、权重 W，以及教导组；
  - `shares.rs`：两边各按 (1 + π)/2 分配的份额；
  - `relation.rs`：亲近与信任的结构、u* 的依恋与信任初值、读出；
  - `fuse.rs`：容量的移出选择与移出后的重新分摊、单组上限；
  - `num.rs`：精确的组内均值、固定次序的内积、有限性检查；
  - `params.rs`：规律参数的校验；
  - `input.rs`：进程内的输入；`ThetaView` 只有 b 与 σ̂；
  - `error.rs`：显式故障；
  - `tests/`：
    - `world.rs`：带种子的合成世界，按事件生成候选、经历组、后来的成员、EV5 回溯、教导及其重复；
    - `engine.rs`：随机世界上每一步之后的性质；
    - `cases.rs`、`boundaries.rs`：手工构造的场景与边界；
    - `law_props.rs`、`group_props.rs`：LAW-1 至 LAW-10；
    - `ac2_count.rs`、`relations.rs`：其余验收；
    - `pyrandom.rs`：CPython 的 MT19937，使移植的测试与 `check_law.py` 抽到相同的样本；
    - `mean_vectors.rs`：用 Python `Fraction` 生成的 283 组均值参照。
- `docs/traceability/P5-23.md`（新）：规范定义 D1–D12、规格对照表、验收编号。
- `docs/traceability/diff-register.md`：追加 V2-38 至 V2-49。
- `tasks/requests/P5-23-law-state-idl.md`（新）：契约接口请求，已由 P5-32 落实。
- `reports/P5-23/`：本报告、`BLOCKED.md`、`gen_mean_vectors.py`、`verification/`。

## 验收

本地工具链：rustc / cargo 1.97.0 stable。`rust-toolchain.toml` 锁定的 1.99.0 下载被代理拒绝，按 AGENTS.md 第 10 节用 `RUSTUP_TOOLCHAIN=stable`；以 CI 按锁定版本运行的结果为准，本分支此前的提交在 CI 上通过。均值参照的生成用系统 Python 3.13.16，脚本的 ruff、mypy --strict 用 `core/contracts` 的 uv 环境。

| 命令或检查（含 Echo Core 验收编号） | 结果 | 证据 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 通过 | verification/rust-fmt.log |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | verification/rust-clippy.log |
| `cargo test --workspace --locked` | 通过，92 个测试，0 失败 | verification/rust-test.log |
| AC2-01（LP1，按 V2-49 的两种意义） | 通过：4,000 组，ρ = 1 时 0 违例，相对同样衰减 0 违例 | verification/law-tests.log |
| AC2-02、I25（每一边份额之和为 1） | 通过：分配函数 20,000 组；经 G，40 个随机世界各 120 个事件（5,702 个组、7,498 个分配），每一步之后每个组每一边之和为 1，包括回溯分摊、后来的成员、容量移出 | 同上 |
| AC2-03 重复部分 | 通过：组内成员整体重复 2–9 次，(S, 强度) 逐位不变；1,760 个已有的组在别的 b、σ̂ 下原样再来一次，R 逐位不变；教导的重复不刷新 | 同上 |
| AC2-05 R 部分 | 通过：30 个随机世界，任意 b 与 σ̂ 下的空步，R 逐位不变 | 同上 |
| AC2-06（LP6） | 通过 | 同上 |
| AC2-07（LP7、推论 4） | 通过：20,000 组，两种意义都 0 违例 | 同上 |
| AC2-13 性质部分 | 通过：短冷遇、5,000 组随机冷暖、长期交替都动不了依恋；一致的反向经历在有限步内改变它，且比普通倾向慢 | 同上 |
| AC2-21 容量与单组上限 | 通过：移出证据量最小的候选，同一边重新分摊并重放；只剩性格时显式故障；随机世界每个分配 W ≤ w_max | 同上 |
| I25 | 通过（见 AC2-02、AC2-03） | 同上 |
| I10 签名检查 | 通过：函数指针类型固定了 G 的参数；源码不出现 x、m、g、λ 的类型与字段；`ThetaView` 恰好两个字段。人为加入 `gain` 字段时测试失败 | 同上 |
| 推论 1–7：LAW-1 至 LAW-10 | 0 违例，见下表 | 同上 |
| ADR 0016 第 4 节：重放恒等 | 通过：随机世界每一步之后，每个倾向按存下的值重放都与存下的倾向逐位相同 | 同上 |
| 测试能抓到错 | 把 group.rs 或 fuse.rs 里的重放去掉，9 个、7 个测试失败；教导组错误地进入 r_k 时，1 个测试失败 | verification/mutations.log |
| 临时结构只在 `law/` 内部 | 通过：`law/mod.rs` 不导出 pending、engine、replay、group、plan；`pending.rs` 不含序列化 | `pending_structures_stay_inside_law_until_p5_32` |
| 规格对照表 | `docs/traceability/P5-23.md` 第 2 节 | — |

LAW 的移植与参考核验对照。种子、规模与 `check_law.py` 相同，用 CPython 的生成器抽样；被测的是本卡的递推：

| 检查 | `results.txt` | 本卡 |
| --- | --- | --- |
| LAW-1 | 41,607 个状态，0 违例 | 41,607，0 |
| LAW-2 | 0 违例；κ = 0…5 需要 [4, 5, 6, 7, 8, 9] 组 | 0；[4, 5, 6, 7, 8, 9] |
| LAW-3 | 61,604 个状态，0 违例 | 61,604，0 |
| LAW-4 | 0 违例，最大 \|Δπ\|/上界 = 0.9899 | 0，0.9899 |
| LAW-5 | 0 处逐位差异 | 0 |
| LAW-6 | 0 违例 | 0 |
| LAW-7 | 重放 0 处差异；闭式在 410 个序列上末位不同（最大相对 4.4e-16） | 0；410（4.4e-16） |
| LAW-8 | 0 违例 | 0 |
| LAW-9 | 重复不改变 (S, 强度)（容差 1e-12）；无反应 (−1, 1)，5σ 反应 (1, 5) | 容差内 0、逐位 0；(−1, 1)，(1, 5) |
| LAW-10 | 普通 29、A_att = 60 为 51、A_att = 600 为 421；交替保持 π ≥ 0.833 | 29、51、421；0.833 |

## 规格对照（Echo Core 相关的卡必填）

逐条见 `docs/traceability/P5-23.md` 第 2 节，对应 v2.0 第 5.1–5.3 节，以及第 2.2、4、6、7、10、12 节和 ADR 0014 第 4 条。摘要：

| 规格条款 | 实现位置 | 测试 |
| --- | --- | --- |
| §5.1 LP1–LP8 | `recurrence.rs`、`shares.rs`、`support.rs`、`params.rs`、`engine.rs`、`input.rs` | `ac2_01`、`ac2_02`、`law_9`、`law_3`、`ac2_05`、`ac2_06`、`ac2_07`、`i10` |
| §5.2 参考形式（π、支持、强度、可信、份额、新近权重、变化率、κ、r_k、s_k） | 同上；`replay.rs` | `law_1` 至 `law_9`；`first_group_records_ordinal_zero_and_the_typical_magnitude` |
| §5.2 计算次序（逐组、回溯分摊在 H 次序中） | `engine.rs`、`group.rs`、`plan.rs`、`replay.rs` | `law_7`、`ev5_back_collection_*`、`late_member_*`、`step_order_of_inputs_does_not_matter` |
| §5.3 推论 1–7 | 同上；`relation.rs` | `law_1` 至 `law_10`、`ac2_13` |
| §4 EV2、EV5、EV7 | `group.rs`、`plan.rs` | `ac2_03_*`、`ev5_*`、`teaching_*` |
| §6、§7、§10 关于人的倾向 | `relation.rs`、`engine.rs` | `relation_*`、`teaching_*` |
| §12 容量与单组上限 | `fuse.rs` | `ac2_21_*` |
| ADR 0014 第 4 条 | `engine.rs`、`plan.rs` | `alias_adr_0014_*` |

涉及的不变量（v2.0 第 17.1 节）：

- I10：类型能力（`ThetaView` 只能从 θ 取 b、σ̂），加签名与源码检查。
- I25：份额函数，以及随机世界每一步之后的检查。
- I11 / LP5：空步返回原 R（`ac2_05`）。
- I23 的前提：
  - 递推是规范定义（`law_7`）；
  - 组内均值与次序无关（D9）；
  - 存下的 ĥ、π、r_k 必须与公式逐位一致（`check_stored`、`check_pending`）；
  - 重放恒等，同一步的输入换次序结果不变。

  整段重算的验收在 P5-26。

IDL：本卡没有改动。需要的五个字段由 P5-32（PR #52）提出。

## 新增依赖

无。测试用的 MT19937 是本卡自写的测试代码，锁文件不变。

## 待用户确认

- [ ] （阻塞）V2-38 至 V2-47：规范定义里 v2.0 没有写到的地方（组的位置与一步之内的次序、后来的成员、份额取 π 的时刻、补做解释的计入位置、关于人的倾向的写法、容量、数值写法、教导组、别名、L 条目）。卡片把这几处定为「不阻塞」，但按 AGENTS.md 第 6 节，新增「重建」条目要你批准后合并。
- [ ] （阻塞）V2-49：AC2-01、AC2-07 在有新近权重时的读法。参考形式下，强的旧支持之后来一组弱的支持，π 可以下降。
- [ ] （阻塞）V2-48：由 P5-32 解决。P5-32 合并后本卡换成生成的类型，然后转为可审查。
- [ ] （不阻塞）信任的预言方向取好奇与成事各半（V2-42），由 P5-30 的实验 3、4 检验。
- [ ] （不阻塞）A_att = 60、n₀ = 3、κ₁ = 1 时，连续约 30 组冷遇就能让依恋降到 ½ 以下。「极难动摇」要多难，由 P5-30 校准 A_att。

## 后续建议（不在本任务范围内）

- **P5-24：** 按 `input.rs` 的 `GroupItem` 交付证据组，包括第一个成员的位置（D1）。组的成员每次给全集。教导组按（说话人、倾向、正反）归组。μ 改变而成员不变属于重新解释，由 P5-26 处理。
- **P5-29：** G 的 `RState.groups` 要读到相关倾向的全部分配，存储需要按倾向索引证据组（ADR 0016 后果）。
- **P5-06：** 两个结构概念（亲近、信任）要作为固定条目登记进概念注册表，G 只引用它们。
- **P5-26：** 整段重算按 D2 逐事件重放 G，增量与整段逐位一致（AC2-08）；重新解释（μ 改变）也在这里。

## 下一步

等待 P5-32（#52）的 Codex 交叉审查与你的合并。之后本卡依次：

1. 合并最新 main；
2. 换成 TendencyV2、EvidenceAllocationV2、EvidenceGroupV2、RSnapshotV2，删去 `pending.rs`；
3. 重跑全部测试；
4. 转为可审查，请 Codex 交叉审查。

接下来取的卡：无，本卡未完成。

## 2026-10-22 跟随 P5-32 审查的修正（追加）

Codex 对 P5-32（#52）的交叉审查用本卡的 G 复现了一个数值边界：M、U 分别舍入。μ = 2⁻¹⁰⁷⁴ 的经历组之后接一个教导组，得到 M = 2⁻¹⁰⁷⁴、U = 0。本卡的 `check_pending` 原先拒绝这个状态，于是下一步连没有新证据也会失败。

- **修正：** 去掉 `check_pending` 里的「U = 0 ⇒ M = 0」，改为精确推论「第一组之前 M = U = 0」。「r_k = clip(M/U) 或 U = 0 时为 0」的检查保留。与 ADR 0016 末节一致。
- **新测试：** `subnormal_edge_u_zero_with_m_nonzero_is_carried_and_recovers` 复现审查的两步和位模式（M `0x1`、U `0x0`），确认重放恒等，之后没有新证据的一步 R 不变、有证据的一步照常记入。换回旧规则时这个测试失败（`verification/review-fix-mutation.log`）。
- **同类问题：** 我顺着检查了同类的舍入边界。n₀ + κ 相对 B 小到 binary64 分辨不出时（例如 n₀ = 10⁻³⁰⁰），记入一组可以得到 |A| = B、π = 1，落到 v2.0 第 1 节要求的开区间之外，原先要到下一步才被拒绝。现在在产生它的那一步返回显式故障 `PrecisionExhausted`，不存下；初值同样检查（D9、V2-44）。实际参数（n₀ 量级为 1）下 B 要大到约 10¹⁶ 才会发生，所以这只是防护。测试：`precision_exhaustion_is_an_explicit_fault_at_the_step`。

合并最新 main（#50、#54、#55）后重跑：cargo fmt、clippy、test（94 个），law 测试 47 个，全部通过。日志已更新。
