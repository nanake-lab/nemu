# Difference register — P0-01 template

P0-06 registers E1–E14 and any newly discovered differences. P0-01 introduces no semantic change.

| ID  | Specification file and section | Nemu implementation | Frozen boundary affected? | Status | Evidence |
| --- | ------------------------------ | ------------------- | ------------------------- | ------ | -------- |

## P0-06 register — 2026-10-14

F/D/R/O/S refer to exact original filenames and SHA-256 in [baseline.md](baseline.md). D section references explicitly distinguish Part I from Part II. “Approved” means the existing repository decision, not new approval by this agent. E5/E8/E13/E14 retain the recorded 2026-10-13 user approvals.

| ID | Original file + section (baseline alias) | Nemu behavior | Frozen boundary | Status / authority |
| --- | --- | --- | --- | --- |
| E1 | S §2; D Part I §6 | Current-user ACL Windows named pipe replaces Unix socket; encoding and semantics unchanged | No | Approved, ADR 0002 |
| E2 | S §§3,21; D Part I §§8,10.9 | Low-IL worker, medium-IL authority; retain protocol capability checks | No | Approved, ADR 0002 |
| E3 | S §§2,7,23; F §19.2 | Local NTFS under LOCALAPPDATA/Nemu/core; no network/sync storage | No | Approved, ADR 0002 |
| E4 | F §19.2; S §§9,11,20 | Desktop is terminal: observations/results in, output/actions out; no authority | No | Approved, ADR 0002; CONTRACTS §4 |
| E5 | F §2.1; O §§12,18 I18 | Fixed versioned m-to-body readout, including posture and expression | No; additional consumer only | Approved 2026-10-13, ADR 0002; CONTRACTS §7 |
| E6 | S §20; F §11 | Versioned authorized opportunity rules propose actions | No | Approved integration register; CONTRACTS §8 |
| E7 | O §§12–14,16; F §10 | Mode C records remote draft, local strictly modulated rewrite is final | No; strict claim only final step | Approved, ADR 0003 |
| E8 | F §10; O §14; S §21 | Mode B text projection, TEXT_PROJECTED; never strict modulation success; autonomous action exception explicit | Explicit approved exception to O §14 default; not a silent reinterpretation | Approved 2026-10-13, ADR 0003 |
| E9 | F §11; O §16.3; S §20 | Codex app-server executor stays behind action/permission gate | No | Approved, ADR 0004 |
| E10 | F §§3,4,16.6; S §§9,11 | Retraction/correction removes eligibility for c_task, not historical erasure | No; actual erasure undecided/out of scope | Approved v1.0 boundary, ADR 0002 |
| E11 | F §§3.2–3.4; D Part I §4; S §§10,19 | Chinese/English/Japanese labels and carrier calibration; unchanged identity/governance | No | Approved, ADR 0002 |
| E12 | S §1; O §§1,19; supplied F §19 and D Appendix A endings | Do not reconstruct missing F §§20–27 / D Appendix B; no growth in v1.0 | No | Handling approved, README §2; missing source 待用户提供 |
| E13 | S §§21,24,29 SYS14; O §§4,5.4 | Full event order; IRL ⊥sem, rehearsal-only F disabled/m=0; G/C identity with audit; no core-dependent autonomy at m=0 except E8 | No | Approved 2026-10-13, ADR 0005 |
| E14 | O §§2,16,19; F §§3,4,13.1; S §§6,22 | REHEARSAL/LIFE disjoint; signed genesis starts LIFE history; optional imported rehearsal text is document provenance, not lived independent evidence | No | Approved 2026-10-13, ADR 0005 |

### Newly discovered conflicts — not resolved by this register

| ID | Original file + section | Conflict and conservative interim behavior | Frozen boundary | Status |
| --- | --- | --- | --- | --- |
| W00-C01 | F §§7,7.1,16.2,16.9 versus O §§9,18 I20,19,20 C01; S §17 | F admits Eligible_multi OR Eligible_single with strict dwell/impact/budget conditions; O requires n_min > 1 independent roots and forbids single-root nonzero θ update. Keep G/C identity and NO-GROWTH in v1.0; do not implement either growth branch. Options and consequences in ADR 0006. | Potential precedence change; cannot decide autonomously | **待用户确认（阻塞）** |
| W00-C02 | S §29 versus S §32 | Matrix enumerates SYS01–SYS25; final-delivery prose says 24 system tests. Keep all 25 IDs and README's explicit M2 subset including SYS25; never drop a test to match the prose count. Confirm treating “24” as a count typo. | No threshold/ID change proposed | **待用户确认（阻塞）** |

Both discrepancies were also checked against `word/document.xml` in the hashed originals. E12 is an existing missing-source item, not a newly invented v1.0 implementation requirement. No existing approval is reopened.

## User decision update — 2026-10-14

- **W00-C01:** The user selected option 2 in proposed ADR 0006: accept the bounded v1.0 baseline and defer the single-root versus multi-root growth conflict. The conflict remains unresolved. ADR 0006 remains **PROPOSED**, not adopted. v1.0 remains **NO-GROWTH**: R is empty and G/C are identity operations with audit. W09/W10 are blocked until the user later adjudicates the conflict; this does not claim the original specifications are reconciled. The missing `Review_and_Remediation` material remains separately marked **待用户提供**.
- **P5-01 scope direction:** Preserve qualification-branch types and single-root ledger fields only where supported by the supplied specifications. Define and encode types only; do not connect them to the runtime path. Do not infer missing fields or expand the specification. Any proposed specification extension requires a separate ADR before implementation.
- **W00-C02:** The user confirmed that S §32's “24” is a count typo. Use the complete SYS01–SYS25 list in S §29 and retain all 25 acceptance items, including SYS25. No test ID or threshold is removed or changed.

These decisions clear the user-decision gate for the bounded P0-06 baseline. W00-C01 remains an unresolved growth blocker for W09/W10.

## P5-01 initial contract proposal — 2026-10-14

| ID | Source | Proposed engineering choice | Status |
| --- | --- | --- | --- |
| W01-D01 | S §6; D Part I §§6,10.8; AGENTS §§2,7; CONTRACTS modification rules | ADR 0007 makes the initial Protobuf/CanonicalV1 presence, field order, endian, type tag, hash domain and strict-decoding rules explicit before publishing IDL. Source semantics remain unchanged. | Proposed contract; awaiting user approval. Blocks initial authoritative wire publication, not the independent numeric helpers. |

This is a new encoding-contract review, not a reopening of W00-C01 or W00-C02.
No missing Review_and_Remediation text is reconstructed. ADR 0006 remains proposed.

### W01-D01 approval — 2026-10-14

The user approved ADR 0007 with amendments: independent frozen string type
tags (tag changes require a new encoding version), a hand-calculated hex
example defining complete-header hashing and repeated nested headers, and an
explicit N3/genesis freeze point with backup/migration before birth and new
encoding versions/migrations after genesis signing. S §6's separately
versioned extensible audit-field rule is included. W01-D01's approval gate is
cleared. All golden-vector and acceptance obligations remain; P5-01's PR must
remain unmerged until all pass. C01/C02 decisions are unchanged.

## P5-01 approved implementation record — 2026-10-14

ADR 0007 amendments have been implemented and cross-language checked. 78 source-mapped messages, independently frozen tags, separately versioned AuditExtension and source-backed R/G type-only definitions are in core/contracts/. Full mapping: reports/P5-01/IMPLEMENTED-MAP.md. D10.8/SYS21 evidence: reports/P5-01/acceptance/. No original specification has changed, and no growth runtime path has been connected. W01-D01 approval gate is closed; the existing W00-C01/W00-C02 dispositions are unchanged.

## E12 状态追加记录（2026-10-14 决定）

**追加状态说明（2026-10-14 决定）：用户确认不可得（2026-10-14 决定）**。此状态取代上述“待用户提供”的当前含义；原文保留作历史说明。范围为 Review_and_Remediation 增补 F 第 20–27 节、D 附录 B。不重建缺失内容；C01 保持未决、仅阻塞 W09/W10，ADR 0006 保持 PROPOSED，v1.0 保持 NO-GROWTH。 E12 表格及此前裁决段落中的旧状态仅为历史记录，不再表示等待用户提供。

## 2026-10-19 追加：数学核验登记（ADR 0008）

来源：[Echo Core 规格数学结构核验](echo-core-math-audit.md)（PR #32）。原文不改，下表只记录读法与状态。

| ID | 原文位置 | 冲突与决定 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| W00-C03 | O §§5.1, 5.2, 18 I05 | 收缩界只对纯时间投影成立；A_x 与事件门读 x̄ 时，事件映射可放大差异（核验 P-1、PHI-7）。决定：事件映射对 x̄ 非扩张；v1.0 中 A_x 不读 x̄、门只依赖 z、F_event 作用为常数、F_out 恒等；超过 T_x 的情绪延续只经 R | 否；收紧 I05 的读法 | 已采纳，ADR 0008 |
| W00-C04 | O §§3.2, 10.1, 12, 15; S §12 | `CompileAuthorizedEffects` 五种签名，O §15 写法可能重复衰减（核验 X-2、PHI-9）。决定：统一为 S §12，传存储快照，内部投影一次；投影视图必须重记锚点 | 否 | 已采纳，ADR 0008 |
| W00-C05 | F §10; O §§4, 12, 12.1; S §12 | J 的签名缺 t 与 validity，与「相同合法输入必得相同 m」冲突（核验 X-3）。决定：J(θ, x, R, t, validity; carrier_config)，JStateView 含二者 | 否 | 已采纳，ADR 0008 |
| W00-C06 | O §7.1; I10; G06 | 机制祖先是否取传递闭包（核验 P-2、META-1）。建议取传递闭包 | 否 | 待用户确认（不阻塞 v1.0；阻塞 W06–W10） |
| W00-C07 | O §6.2 对 S §15; O §2; I03 | 全局测试机会编号 j 记在 R 还是 Q（核验 X-1）。建议只在 Q 记录，R、L 只存引用；涉及状态归属 | 可能涉及状态归属 | 待用户确认（不阻塞 v1.0；阻塞 W06–W10） |

## 2026-10-19 追加：v2.0 第一性核心（ADR 0009–0011）

来源：用户 2026-10-19 在对话中同意的 [Echo Core v2.0 核心规格](../echo-core/echo-core-v2.0-first-principles.md)。原文不改；下表只记录取代关系与状态，合并即生效。

| ID | 原文位置 | 决定 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-01 | F §§6–9；D 第二篇；O §§6–11；S §§14–18 | R、θ 的成长机制换成 v2.0 的证据规律（第 4、5、7、8 节）：R 是倾向，θ 是她自己的动力学 | 是：用户新冻结的本体取代旧条款 | 已采纳，ADR 0009 |
| V2-02 | O §2、§18；F §2 | (R, θ) 可由 H 重算；不变量 I03、I06、I10 改写，I07–I09、I12、I19、I20、I22 被取代，新增 I23–I29 | 是 | 已采纳，ADR 0009 |
| V2-03 | O §5.2；ADR 0008 §1 | A_x 改为天性评估 a(e)（联结、共情、好奇、成事），在 m = 0 下得出并记录；驱动为 g ⊙ (a − b) 加情境作用，不读 x̄ | 是；与 ADR 0008 §1 一致 | 已采纳，ADR 0009 |
| V2-04 | O §§12–13；ADR 0002 决定 8、10；ADR 0003 语气样例库 | 接口方向从模型自身提取，校准只验证通路与能力；协议 p 不含风格；语气样例库只由模式 A 生成 | 是 | 已采纳，ADR 0009 |
| V2-05 | E5；CONTRACTS §7 | 身体读出只保留朝向你、放松或紧绷、精力三项 | 是；CONTRACTS §7 的修订另行提交，需要用户批准 | 方向已采纳（ADR 0009）；契约修订待做 |
| V2-06 | E6；CONTRACTS §8 | 主动由天性驱动；机会规则表只保留边界 | 是；CONTRACTS §8 的修订另行提交，需要用户批准 | 方向已采纳（ADR 0009）；契约修订待做 |
| V2-07 | E10；ADR 0002 决定 7 | 经历删不掉。「忘掉」仍是撤回，不撤销那段经历作为证据；只有来源不真实时才移除证据并重算。别人要求删除其个人数据的法律义务仍未决（v2.0 第 18 节） | 否；与撤回一致 | 已采纳，ADR 0009、0011 |
| V2-08 | ADR 0002 决定 4；ADR 0005；README §3；E13、E14 | v1.0 含成长；排练谱系接通情绪、关闭成长；新增 EXPERIMENT 谱系；N3 重新定义；诞生日期重新排期 | 是 | 已采纳，ADR 0010 |
| V2-09 | 无（新增） | 网络生活的边界：共用账号、诚实底线、隐私默认、别人对她没有权力、保险丝 | 新增 | 已采纳，ADR 0011 |

既有条目的状态更新：

- **W00-C01：** 已消解。v2.0 没有单根、多根资格；ADR 0006 被 ADR 0009 取代。
- **W00-C06、W00-C07：** 已消解。计数不读 x、m、g、λ，没有 meta 自排除，也没有统计检验编号 j。
- **W00-C03、W00-C04、W00-C05：** 不变，ADR 0008 保留。
- **E12：** 不再阻塞。缺失的增补不再需要，也不重建。
- **W00-C02：** 不变。

## 2026-10-20 追加：任务卡改写时对 v2.0 的读法

来源：用户 2026-10-20 批准按 v2.0 改写任务卡。下表是改写时对 v2.0 条文的读法，都不改变规格。它们标为「待用户确认（不阻塞）」：可以在改写任务卡的 PR 里一并确认，也可以在对应的卡开工前确认。

| ID | 原文位置 | 读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-10 | v2.0 第 5.4、12 节 | 跳变检查与缓慢漂移的「暂缓应用」实现为自动暂停（`GROWTH_REVIEW_PENDING`）：事件照常进入 H，x 照常变化，R、θ 停在暂缓之前。复核结论为规律正确时按 H 补算并应用，同一变化不再暂缓；规律有错时保持暂停，直到新规律版本重算 | 否；与暂停成长的语义一致 | 待用户确认（不阻塞）；P5-26 |
| V2-11 | v2.0 第 2.5、5.4 节 | 暂停、恢复、复核暂缓与复核结论除写进 L 外，也作为事件写进 H，使暂停期间的整段重算同样逐位一致（I23） | 否 | 待用户确认（不阻塞）；P5-26 |
| V2-12 | v2.0 第 2.4、7 节；ARCHITECTURE.md 6.9 | 情境吻合 M 要读 R 的概念定义，它属于核心状态，所以 M 默认只用本地后端；用远程后端要用户允许 | 否；收紧 | 待用户确认（不阻塞）；P5-28 |
| V2-13 | v2.0 第 5.2 节「可信」、第 6、7、10 节 | 关于人的倾向：亲近与信任是结构上的关系倾向，每个人至多各一个，第一次涉及此人时以 π = 0 建立；u* 的亲近就是依恋（先验 A_att），u* 的信任由依恋先验给出初值。预言方向与证据的写法由 P5-23 给出，P5-30 检验 | 否；补全参考形式 | 待用户确认（不阻塞）；P5-23 |
| V2-14 | v2.0 第 2.4、4 节（EV5、EV6） | 解释记录（a(e)、μ、候选、标注）不进入 c_core 和 c_task：判断不受她过去反应的影响，EV5 的回溯收集在结构上看不到结果的正反 | 否；收紧 | 待用户确认（不阻塞）；P5-04、P5-28 |
| V2-15 | CONTRACTS.md 第 7 节（E5）；v2.0 第 9.3 节 | 身体读出 v2 的字段：`facing` 0..1（中性 0.5）、`tension` −1..1（中性 0）、`energy` 0..1（中性 0.5），由四个天性通道的固定线性组合得出；没有表情基线；m = 0 时为中性 | 否；按 V2-05 | 待用户确认（不阻塞）；契约修订在 P5-22，需要用户批准 |
| V2-16 | CONTRACTS.md 第 8 节（E6）；v2.0 第 9.4 节 | 机会规则表改为边界表。`helper_done` 改为助手任务的后续，说出口时门控照旧；`codex_needs_you` 改为一次立即的主动时机，受全部门控；其余七条删除 | 否；按 V2-06 | 待用户确认（不阻塞）；契约修订在 P5-22，需要用户批准 |
| V2-17 | v2.0 第 2.4、5.4、13 节 | 原因码：`GROWTH_OFF_REHEARSAL`（排练不成长，取代 `GROWTH_NOT_ENABLED_V1`）、`GROWTH_PAUSED`、`GROWTH_REVIEW_PENDING`、`A_DEGRADED`（取代 `AX_DEGRADED`）、`M_DEGRADED` | 否 | 待用户确认（不阻塞）；P5-21 |
| V2-18 | v2.0 第 10、11 节；ADR 0011 第 3、10 条；CONTRACTS.md 第 6 节 | 她自己的社交消息是新的行动种类 `social.message`（默认 T2，可设为信任）；让你承担的事（约定、付款、代表你表态）是 `social.commit`，一律 T3。发出前在 m = 0 下判定，判定不了按 T3；诚实底线也在发出前检查 | 否；按 ADR 0011 | 待用户确认（不阻塞）；P5-22、P6-14 |

## 2026-10-20 追加：P5-21 IDL 契约变更中的重建与读法

来源：任务卡 P5-21（v2.0 的状态与记录）。v2.0 没有写到、由本卡按最小字段集重建的地方，以及对 v2.0 条文的读法。原文不改。下表各条都随 PR #36 一并请用户确认；在确认之前，P5-21 不合并（`needs-user`）。

| ID | 原文位置 | 重建或读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-19 | v2.0 第 13、16 节；P5-21 卡「规律版本：第 16 节的全部参数」 | 第 16 节的量按第 13、16 节自己的归属分放，每个值只出现一处：规律版本 `LawParameters` 放 n₀、κ₁、w_max、w_teach、v_u、r_min、r_max、h₀、n_h、ĥ 的下限（可选）、h₀,θ、ĥ_θ,min、n_b、n_σ、g_min、g_max、λ_min、λ_max、w_min、T_x、ε_x、k_max；genesis 另列 g₀、b₀、σ₀、τ₀ 与 A_att（第 13 节单列）；保险丝与网络上限是 u* 的边界（`FuseConfig`，初值在 genesis，之后作为事件调整）；方向、β、通路与能力的容差在方向集与验证记录里；读出参数在两个读出函数资源里（P5-12、P5-22 定格式） | 否；只是放置 | 待用户确认（阻塞，随本卡） |
| V2-20 | v2.0 第 2.5、5.4、9.5、11、12 节 | L 与保险丝的最小字段集（重建）：R 的变化 `TendencyChange` 用 π 前值缺省表示进入 R、π 后值缺省表示因容量移出 R；整段重算的差异摘要为原因（新规律版本、来源更正、重新解释）、前后规律版本、R 与 θ 的前后修订号、变化的倾向 ID、θ 是否变化；迁移记前后两个环境对象与验证记录；复核暂缓记触发（跳变、θ 单步触顶、缓慢漂移）；每个 L 条目带同一事务中提交的 H 事件 ID（V2-11）；复核结论命令不带暂缓条目 ID（同一时间只有一个暂缓）；`FuseConfig` 为 R 容量、Δθ_max、跳变阈值与探测情境、缓慢漂移的持续组数与距边界的余量、新联系人数与消息数上限及其时间窗 | 否 | 重建，待用户确认 |
| V2-21 | v2.0 第 6、8、16 节 | 单位：恢复速率 λ 以 1/秒计（`recovery_per_s`）；τ̂、τ₀、T_x 以秒计（double）；网络上限的时间窗以纳秒计（uint64）。H 中的时间仍是整数纳秒 | 否 | 重建，待用户确认 |
| V2-22 | v2.0 第 1、3、5.2、7 节 | 编解码之外的语义检查（`core-types/src/semantic.rs`）：d_k 为单位向量，容差 \|‖d‖₂ − 1\| ≤ 1e-12；π 在开区间 (−1, 1)；列出的 μ > 0；\|A_k\| ≤ B_k、B_k > 0、翻转数不超过相邻组数；A_att 只在亲近倾向上；第 16 节写明的不等式（n₀ > 0、w_teach ≤ w_max、v_u < 1、λ_min > 0 等）；genesis 中成事的初始增益严格低于其余三个（第 3 节「最弱」，相等不算） | 否 | 待用户确认 |
| V2-23 | v2.0 第 1、4 节（EV2–EV7）；P5-21 卡「解释记录」 | 解释记录的外形（重建）：每条记录有 record_id，并单列它解释的事件 event_id（EV5 回溯与重新解释时与 ProposalBinding 的事件不同）；教导、撒谎、同一件事分成三种记录；重新解释是一条单独的链接记录（新记录、旧记录、解释器版本）；天性评估带所涉之人的列表（取哪一个人做共情加权由 P5-10 定）；事件属性记录由核心按规则表给出，带规则表版本，不带 ProposalBinding | 否 | 重建，待用户确认 |
| V2-24 | v2.0 第 5.2、7、8 节 | R、θ 的存储（重建）：倾向按第 5.2 节的定义存 A_k 与 B_k（B_k 含 n₀ + κ_k），另存 ĥ_k、组数、翻转数、上一组的符号（还没有非零支持时缺省）、π、r；证据组按组存可信与可核实，按倾向存支持 S、强度与份额；θ 的快照存 (b, g, λ)、σ̂、τ̂、Σω（标量）、ĥ_θ，变点检测与同号持续段的递推状态是一个由规律版本定义格式的对象引用（P5-25 定） | 否 | 重建，待用户确认 |
| V2-25 | ADR 0007 第 2、11 条；P5-21 卡「VersionedEnvironment 增加…」等 | 改动的消息一律以带新类型标签的后继消息实现，旧消息原样保留并标为取代，已发布的字节与摘要不变：VersionedEnvironmentV2、ProposalBindingV2、FastStateSnapshotV2、CoreTransactionV2、CoreCommitResultV2、ModulationEnvelopeV2、LineageHeaderV2。后继消息沿用前任的字段号，被取代的号用 `reserved`，新字段号都大于前任的最大号。ProposalBinding 换代后，携带它的 SemanticCandidate、ConceptProposal 也有后继（卡里没有写到）。VersionedEnvironmentV2 中天性评估器、情境吻合器、方向集、规律版本四项可缺省，使尚未接通的阶段如实显示为缺省；正式谱系与实验谱系启动时要求四项都在（P5-08）。ModulationEnvelopeV2 去掉 HookCoefficients，保留 effect_receipt_digest；HookCoefficients 随之标为取代 | 否；不改任何已发布字节 | 待用户确认（阻塞，随本卡） |
| V2-26 | v2.0 第 2.4、5.4、12、13、15 节；V2-17 | 原因码与状态的判定：`GROWTH_PAUSED`、`GROWTH_REVIEW_PENDING`、`A_DEGRADED`、`M_DEGRADED` 是 RunStatus 的状态，本身说明原因，不另设 FailureCode；`GROWTH_OFF_REHEARSAL` 是 FailureCode，配 NO_GROWTH；A、M 失败的原因用既有的通用码（DEPENDENCY_FAULT、VERSION_MISMATCH、NUMERIC_INVALID 等）。取代：CORE_QUARANTINED（第 12 节不再整体隔离）、AX_DEGRADED、GROWTH_NOT_ENABLED_V1、AX_UNAVAILABLE、EVIDENCE_INCOMPLETE（C 的凝固查询计划已取代）、UNSUPPORTED_SLICE、UNKNOWN_OVERLAP。`Lineage` 增加 EXPERIMENT 后，终端的 RunState 在编解码层也能带 EXPERIMENT；按 P5-22 卡「终端的 Lineage 不含 EXPERIMENT」，由 P5-22 收紧 | 否 | 待用户确认 |
| V2-27 | v2.0 第 2.3、13 节；I28 | genesis v2 的读法：她的名字不在 genesis 里（它是 H 的第一条事实）；没有 x 的初值（x 从静息点 (b₀, 0) 开始）；载体只列 carrier（模型由载体条目确定）；天性结构以一个结构版本标识表示；边界配置、两个读出函数和协议 p 都以对象引用给出，不内嵌任何文本。I28 的类型检查：genesis 可达的字符串只有 5 个标识字段，字节只有 32 字节摘要，三语文本类型不可达 | 否 | 待用户确认 |
