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

## 2026-10-20 追加：ADR 0012

来源：ADR 0012（说话的连续性、载体与移动端）。都不改变规格，状态随 ADR 0012：用户合并即批准。

| ID | 原文位置 | 读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-19 | O §13.2；v2.0 第 9.5 节、I17；P0-07 判定规则 | 零调制「与原模型一致」的基准是同一载体配置（权重、量化、内核、数值设置）不装 hook 时的输出。内核属于载体配置；参考内核与生产内核之间的差记录在案，不算通路失败；以后换内核按迁移处理 | 否；把内核纳入载体绑定 | 待用户批准，ADR 0012 第 2.1 条；P0-07 |
| V2-20 | v2.0 第 9.5、9.6 节 | 迁移时增加风格连续性检查与判断一致性检查。门槛以同一载体或同一判断者重跑的差异为基线，倍数预先写定；结果只决定切不切换，不调参 | 否；补全迁移的验证 | 待用户批准，ADR 0012 第 3.2、3.3 条；P5-31 |
| V2-21 | v2.0 第 2.3、9 节；I28 | 推理时去掉载体自带的助手身份方向，作为 Ω 中绑定载体、带版本的资源；先做信息性对比，由用户决定是否启用；诞生后启用按迁移处理 | 否；载体准备，不涉及她的状态 | 待用户批准，ADR 0012 第 4 条；P5-31 |
| V2-22 | v2.0 第 3 节 | 四个天性的覆盖报告：只读，从 H 派生，可重算；不改变天性结构，不自动增加天性 | 否 | 待用户批准，ADR 0012 第 6 条；P5-30 |
| V2-23 | v2.0 第 2.4 节；V2-14；P5-04 | c_task 检索：四路召回加 RRF 合并。人物与事物的索引只取 IRL 已接受的语义图，不取解释记录；查询计划确定、带版本 | 否；按 V2-14 | 待用户批准，ADR 0012 第 7 条；P5-04 |

## 2026-10-21 追加：ADR 0014

来源：ADR 0014（v2.0 的小幅修正，吸收 PR #40 研究中的发现）。用户 2026-10-21 决定保留四个天性、θ、R⁺ 门槛与依恋先验，关闭 PR #40。以下都不改 v2.0 原文和契约，状态随 ADR 0014：用户合并即批准。PR #40 未合并，其 EC2-R01 至 R08 不进入本登记。

| ID | 原文位置 | 读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-24 | v2.0 第 4 节 EV7、EV8 | 只有「关于她该怎样」的话是教导；说话人自己的偏好或经历（`USER_SELF_REPORT`）是关于说话人的事实，请求走行动，事实陈述是知识；听到时她的 a(e) 照常是经历 | 否；收窄 EV7 的适用范围 | 待用户批准，ADR 0014 第 2 条；P5-24、P5-28 |
| V2-25 | v2.0 第 3 节（联结）、第 4 节 EV5 | 只评价观察到的结果：没有回复本身不是被冷落；结果未知的事件不进入以该结果为依据的证据组（`EVIDENCE_UNKNOWN`），已知后按新事件进入 | 否 | 待用户批准，ADR 0014 第 3 条；P5-11、P5-24、P6-14 |
| V2-26 | v2.0 第 5.1 节 LP3、第 4 节 EV2 | 同一规范概念的别名不建立新倾向，不增加证据或先验；π 与证据组对增加别名不变 | 否；明确推论 | 待用户批准，ADR 0014 第 4 条；P5-23、P5-24 |
| V2-27 | v2.0 第 4 节 EV6 | m = 0 不保证判断无偏；IRL、A、M 的验收报告增加偏差度量，偏差敏感性在实验谱系估计；不设门槛 | 否 | 待用户批准，ADR 0014 第 5 条；P5-07、P5-11、P5-28、P5-30 |
| V2-28 | v2.0 第 5.2 节（LP2、变化率）、第 14 节 | 实验谱系增加 LP2 分摊、变化检测、等价行动习惯、评估偏差敏感性四项对照；对照结果只用于决定是否发布新规律版本 | 否 | 待用户批准，ADR 0014 第 6 条；P5-30 |

## 2026-10-20 追加：P5-21 IDL 契约变更中的重建与读法

来源：任务卡 P5-21（v2.0 的状态与记录）。v2.0 没有写到、由本卡按最小字段集重建的地方，以及对 v2.0 条文的读法。原文不改。下表各条都随 PR #36 一并请用户确认；在确认之前，P5-21 不合并（`needs-user`）。

编号说明（2026-10-21）：本节原用 V2-19 至 V2-27，与先合并的 ADR 0012（V2-19 至 V2-23）、ADR 0014（V2-24 至 V2-28）重号，改为 V2-29 至 V2-37，内容不变。

| ID | 原文位置 | 重建或读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-29 | v2.0 第 13、16 节；P5-21 卡「规律版本：第 16 节的全部参数」 | 第 16 节的量按第 13、16 节自己的归属分放，每个值只出现一处：规律版本 `LawParameters` 放 n₀、κ₁、w_max、w_teach、v_u、r_min、r_max、h₀、n_h、ĥ 的下限（可选）、h₀,θ、ĥ_θ,min、n_b、n_σ、g_min、g_max、λ_min、λ_max、w_min、T_x、ε_x、k_max；genesis 另列 g₀、b₀、σ₀、τ₀ 与 A_att（第 13 节单列）；保险丝与网络上限是 u* 的边界（`FuseConfig`，初值在 genesis，之后作为事件调整）；方向、β、通路与能力的容差在方向集与验证记录里；读出参数在两个读出函数资源里（P5-12、P5-22 定格式） | 否；只是放置 | 待用户确认（阻塞，随本卡） |
| V2-30 | v2.0 第 2.5、5.4、9.5、11、12 节 | L 与保险丝的最小字段集（重建）：R 的变化 `TendencyChange` 用 π 前值缺省表示进入 R、π 后值缺省表示因容量移出 R；整段重算的差异摘要为原因（新规律版本、来源更正、重新解释）、前后规律版本、R 与 θ 的前后修订号、变化的倾向 ID、θ 是否变化；迁移记前后两个环境对象与验证记录；复核暂缓记触发（跳变、θ 单步触顶、缓慢漂移）；每个 L 条目带同一事务中提交的 H 事件 ID（V2-11）；复核结论命令不带暂缓条目 ID（同一时间只有一个暂缓）；`FuseConfig` 为 R 容量、Δθ_max、跳变阈值与探测情境、缓慢漂移的持续组数与距边界的余量、新联系人数与消息数上限及其时间窗 | 否 | 重建，待用户确认 |
| V2-31 | v2.0 第 6、8、16 节 | 单位：恢复速率 λ 以 1/秒计（`recovery_per_s`）；τ̂、τ₀、T_x 以秒计（double）；网络上限的时间窗以纳秒计（uint64）。H 中的时间仍是整数纳秒 | 否 | 重建，待用户确认 |
| V2-32 | v2.0 第 1、3、5.2、7 节 | 编解码之外的语义检查（`core-types/src/semantic.rs`）：d_k 为单位向量，容差 \|‖d‖₂ − 1\| ≤ 1e-12；π 在开区间 (−1, 1)；列出的 μ > 0；\|A_k\| ≤ B_k、B_k > 0、翻转数不超过相邻组数；A_att 只在亲近倾向上；第 16 节写明的不等式（n₀ > 0、w_teach ≤ w_max、v_u < 1、λ_min > 0 等）；genesis 中成事的初始增益严格低于其余三个（第 3 节「最弱」，相等不算） | 否 | 待用户确认 |
| V2-33 | v2.0 第 1、4 节（EV2–EV7）；P5-21 卡「解释记录」 | 解释记录的外形（重建）：每条记录有 record_id，并单列它解释的事件 event_id（EV5 回溯与重新解释时与 ProposalBinding 的事件不同）；教导、撒谎、同一件事分成三种记录；重新解释是一条单独的链接记录（新记录、旧记录、解释器版本）；天性评估带所涉之人的列表（取哪一个人做共情加权由 P5-10 定）；事件属性记录由核心按规则表给出，带规则表版本，不带 ProposalBinding | 否 | 重建，待用户确认 |
| V2-34 | v2.0 第 5.2、7、8 节 | R、θ 的存储（重建）：倾向按第 5.2 节的定义存 A_k 与 B_k（B_k 含 n₀ + κ_k），另存 ĥ_k、组数、翻转数、上一组的符号（还没有非零支持时缺省）、π、r；证据组按组存可信与可核实，按倾向存支持 S、强度与份额；θ 的快照存 (b, g, λ)、σ̂、τ̂、Σω（标量）、ĥ_θ，变点检测与同号持续段的递推状态是一个由规律版本定义格式的对象引用（P5-25 定） | 否 | 重建，待用户确认 |
| V2-35 | ADR 0007 第 2、11 条；P5-21 卡「VersionedEnvironment 增加…」等 | 改动的消息一律以带新类型标签的后继消息实现，旧消息原样保留并标为取代，已发布的字节与摘要不变：VersionedEnvironmentV2、ProposalBindingV2、FastStateSnapshotV2、CoreTransactionV2、CoreCommitResultV2、ModulationEnvelopeV2、LineageHeaderV2。后继消息沿用前任的字段号，被取代的号用 `reserved`，新字段号都大于前任的最大号。ProposalBinding 换代后，携带它的 SemanticCandidate、ConceptProposal 也有后继（卡里没有写到）。VersionedEnvironmentV2 中天性评估器、情境吻合器、方向集、规律版本四项可缺省，使尚未接通的阶段如实显示为缺省；正式谱系与实验谱系启动时要求四项都在（P5-08）。ModulationEnvelopeV2 去掉 HookCoefficients，保留 effect_receipt_digest；HookCoefficients 随之标为取代 | 否；不改任何已发布字节 | 待用户确认（阻塞，随本卡） |
| V2-36 | v2.0 第 2.4、5.4、12、13、15 节；V2-17 | 原因码与状态的判定：`GROWTH_PAUSED`、`GROWTH_REVIEW_PENDING`、`A_DEGRADED`、`M_DEGRADED` 是 RunStatus 的状态，本身说明原因，不另设 FailureCode；`GROWTH_OFF_REHEARSAL` 是 FailureCode，配 NO_GROWTH；A、M 失败的原因用既有的通用码（DEPENDENCY_FAULT、VERSION_MISMATCH、NUMERIC_INVALID 等）。取代：CORE_QUARANTINED（第 12 节不再整体隔离）、AX_DEGRADED、GROWTH_NOT_ENABLED_V1、AX_UNAVAILABLE、EVIDENCE_INCOMPLETE（C 的凝固查询计划已取代）、UNSUPPORTED_SLICE、UNKNOWN_OVERLAP。`Lineage` 增加 EXPERIMENT 后，终端的 RunState 在编解码层也能带 EXPERIMENT；按 P5-22 卡「终端的 Lineage 不含 EXPERIMENT」，由 P5-22 收紧 | 否 | 待用户确认 |
| V2-37 | v2.0 第 2.3、13 节；I28 | genesis v2 的读法：她的名字不在 genesis 里（它是 H 的第一条事实）；没有 x 的初值（x 从静息点 (b₀, 0) 开始）；载体只列 carrier（模型由载体条目确定）；天性结构以一个结构版本标识表示；边界配置、两个读出函数和协议 p 都以对象引用给出，不内嵌任何文本。I28 的类型检查：genesis 可达的字符串只有 5 个标识字段，字节只有 32 字节摘要，三语文本类型不可达 | 否 | 待用户确认 |

## 2026-10-22 追加：P5-32 契约变更

来源：ADR 0016 与任务卡 P5-32（用户 2026-10-22 同意方案 a）。V2-48 在 P5-23（PR #49）中登记。

| ID | 原文位置 | 重建或读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-50 | V2-34、V2-48；v2.0 第 5.2 节「典型幅度」「计算次序」；I23、I25 | R 的存储补五个字段：倾向存典型幅度递推的 M = Σω·μ·p_c 与 U = Σω·μ；每个分配存序号、μ_{c,k} 与 p_c（教导组缺省）。以后继消息 TendencyV2、EvidenceAllocationV2、EvidenceGroupV2、RSnapshotV2 实现，旧消息标为取代。改旧分配时，G 只用存下的 S、强度、份额、可信、可核实，以及序号、μ、p_c 重放该倾向的递推，不需要历史的 b、σ̂ 或其他倾向的 π 历史；与整段重算逐位一致。解决 V2-48 | 否；只增，不改已发布字节 | 待用户批准，ADR 0016 |

## 2026-10-22 追加：P5-22 终端契约候选

来源：P5-22 与用户在本聊天授权的 oneof 配套修改。既有条目不改，本 PR needs-user，以下具体表示随用户合并批准。

| ID | 原文位置 | 读法或重建 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-51 | ADR 0007 §2,6；P5-22 的 Readout oneof；V2-35 | 以后继 ReadoutV2 实现 oneof，旧 readout 字段号 2 reserved，v1/v2 用新号 5/6；其他变更使用 RunStateV2、QueryV2、QueryResultV2、ApprovalRequestV2、ApprovalDecisionV2，新标签、旧消息字节不变。BodyReadout v1 只在生产上标取代，迁移期间仍为受支持的兼容类型，避免活跃 oneof 引用机器停用类型；P5-19 只生产 v2。oneof 按 ADR 已规定规则，不增加编码规则 | 否；不改已发布字节 | 待用户批准，P5-22 |
| V2-52 | I26；ADR 0011 §9；P5-22 审批载荷与摘要 | 最小表示：原样执行载荷为 bytes，payload_digest 为 SHA256 该原样字节（32 字节）；请求、决定、实际执行字节必须一致。应用内总线用 approval.request.v2 / approval.decision.v2 后继事件，保留旧事件。SocialAction 只声明种类和原样载荷，载荷解释及发出前检查由 P6-14/P6-15 | 否；重建具体表示 | 重建，待用户批准，P5-22 |
| V2-53 | P5-22；v2.0 §2.5；V2-36 | QueryResultV2 用额外 growth_entries 返回已有 LedgerEntry；非 growth 的该数组为空，growth 的 items 为空。RunStateV2 用新的终端枚举排除 EXPERIMENT，不改变旧 RunState 的解码语义；Notice.at 沿用终端显示时刻字符串，l_ref 为已有 ObjectRef | 否；具体接口表示 | 待用户批准，P5-22 |
| V2-54 | P5-22 主动边界；V2-16 | 当前 main 没有 config/opportunities.json 和 helpers.json，预算沿 CONTRACTS §5.3 的 30；不从九条旧触发规则推断每类行动的新冷却，cooldownS 初值为空，最小间隔 180 秒、说话 2/小时、安静因子 0.5 照常生效。helpers.json 将来创建时需一致 | 否；边界候选 | 待用户批准，P5-22 |

### 2026-10-22 追加：P5-22 首轮审查修订

以下补充 V2-52–54 的实现表示，仍随同一 PR #57 等待批准，不改变其状态或新增编码规则。

- V2-52：异步 SHA256 校验前复制请求、决定及实际载荷；等待期间变化拒绝。返回不可变、一次性载荷快照，执行时消费该快照并重新检查有效期，不以裸布尔值授权另一个可变缓冲区。身份认证和权限执行仍属 P5-05。
- V2-53：应用内用后继 `core.queryResult.v2` 携带 kind、完整 growthEntries 与普通 items；queryResultToPanel 核对 query_id，从 IDL descriptor 校验 LedgerEntry 后传给面板。growth 的 items 必须为空，其他种类的 growthEntries 必须为空。旧 core.queryResult 保留。
- V2-54：新增 ProactivityBoundaries、ProactivityBudgets、ProactivityCooldown 及 ProactiveKind/Gate 权威 IDL；JSON 字段名、枚举字符串别名和 cooldownS 字典投影以 descriptor 元数据定义，由同一源生成资源类型/校验。既有 JSON 外形和候选值不变；新增 wire 标签，不改旧字段或字节。兼容门禁同时冻结这些 JSON 投影元数据。

## 2026-10-22 追加：ADR 0019 参考规律新版本草案

PR #61 的审查评论和原始证据中的 V2-51–V2-56，对应本表的 V2-55–V2-60

本节只登记候选，不修改原文或已发布的规律。V2-38 至 V2-49 只读自 PR #49 的 `d95f65c214897743133a3c525b550fda97cd010e`；不将那个分支的登记复制、批准或改写。用户选择 ADR 0018 方案 A 后明确了以下三类 LP1 比较，完整证明、反例、数值与推论影响见 [ADR 0019](../adr/0019-law-lp1-reference.md)。

| ID | 原文位置 | 新版本提议及影响 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-55 | v2.0 §5.1 LP1；AC2-01；PR #49 V2-49 | 无撤回/分摊时按完整事件比较 before/after；混合变化时固定同一步分摊与次序，去掉新增证据得 cf，新增支持 after>cf、反对 after<cf；撤回支持 cf 不升、撤回反对 cf 不降。逐项检查，不能把混合变化整体豁免。这是用户 2026-10-22 指定的原文补全，不是相对衰减基线的旧读法 | 补全，须审批 | 待用户确认（阻塞）；ADR 0019 §2 |
| V2-56 | v2.0 §5.2 新近权重、LP4/5/7；§5.3 推论 6、7 | 实际保留率 q 由名义 h=1−ρ、新信号与先验回拉预算确定；只有会逆转 LP1 时限制衰减，P、c 不衰减。ω 改为实际 q 的乘积，M/U 同样乘 q；零有效信号 q=1。固定名义 ρ 的原几何上界失效，原 LAW-6 保留且失败；依恋代数不变，交替抵抗的充分条件、改变速度另列 | 新规律版本，改变参考形式与推论 | 待用户确认（阻塞）；ADR 0019 §3–4、7 |
| V2-57 | v2.0 §5.2 计算次序；I23、LP2；ADR 0016；PR #49 D2–D5 / V2-40 | 每个分配首次计入时计算并保存实际 q，普通份额修订不重新估计旧 q；旧修订全部写好并重放后再加新组。否则存在“撤回反对反而降低 π”的反例。来源更正/新规律版本仍从有效 H 重算并重新生成 q，不能用旧分配重放代替 AC2-19 | 改参考次序；增加后继契约需求 | 待用户确认（阻塞）；ADR 0019 §5–6 |
| V2-58 | v2.0 §2.2、§5.2 份额；I25；PR #49 D9 | 每边正比例配额以最大余数法量化到 2^-52，规范 ID 打破平局，实数与 binary64 次序和精确为 1；正份额被量化成零时显式故障。新证据不能产生可表示的严格方向变化、下溢、边界、计数精度耗尽也显式故障，不降阈值或静默跳过 | 补全数值规范，有舍入差异 | 待用户确认（阻塞）；ADR 0019 §8 |
| V2-59 | V2-34、V2-50；ADR 0007、0016 | 草案需要分配的 actual_retention 与倾向的独立证据量 V，后者避免 B−c 的舍入损失。必须另立后继消息/契约卡，不重解释现有 TendencyV2/EvidenceAllocationV2，不修改已发布字节。本文只有 Python 合成夹具，没有 Rust/IDL 实现授权，也未解除 P5-23 阻塞 | 未来契约变更，尚未实施 | 待用户确认（阻塞）；ADR 0019 §8 |

## 2026-10-23 追加：ADR 0019 审查修复与推论 6 决策待选

本节不新增规律或改写原推论6。对应 [PR #61 阻塞审查](https://github.com/nanake-lab/nemu/pull/61#issuecomment-6356460860)，原失败、原统计与源快照保留；实现修复证据见 `reports/law-lp1-reference/review-20261023/REPORT.md`。

| 条目 | 更正 / 实现范围 | 状态 |
| --- | --- | --- |
| V2-55、V2-57 | 回溯中新候选对旧组的首次分配也是新增，必须从 cf 中省去。实现改为显式首次分配计划，旧份额修改后先捕获 cf，再加入首次分配。原随机检查漏测候选自身，现在补全其边际方向，并用仅折叠事件前分配身份的独立 oracle 检查 cf | 修复实现与漏测；设计仍待批准，修复待复审 |
| V2-58 | 份额乘法本身也保留正因子信息：正数×正数下溢到0是显式故障；插入、回溯首次分配、旧份额重放与 W×\|S\| 都检查。合法零因子、M/U次正规数语义不改；完整事件暂存，失败不发布部分状态 | 修复遗漏；不提高权重下限、不放宽 LP1 |
| V2-56 | 候选不满足原推论6，不意味着已经决定撤换它。ADR 0019 §11 单列失去的保证、行为后果和A–D待选做法；新增步的开放上界仅证明局部可行，历史分摊/固定q等完整约束尚未解决，未纳入当前实现 | 原 LAW-6 仍 failed；由用户决定，不自动放弃或修改原推论 |

## 2026-10-23 追加：PR #61 第二轮分侧修复

对应[第二轮阻塞审查](https://github.com/nanake-lab/nemu/pull/61#issuecomment-6357602369)：原参考夹具虽然保持每侧份额和为1，仍可能错误重算未变侧。以下只修复既有分侧规则，不发布新规律，也不改变推论6的A–D待选项。

| 条目 | 实现修复 / 证据限定 | 状态 |
| --- | --- | --- |
| V2-55、V2-57（PR #49 D3 的既有分侧规则） | 新候选按 S>0 / S<0 的直接比较归侧，不用支持值相乘判断符号；只把成员增加的非零侧送入份额量化和旧分配重放。S=0不改变任一侧，仍保留首次零支持记录；未变侧既不重新量化也不重放，避免无关的PrecisionExhaustedShare。新检查独立依据旧分配的符号判定未变侧，不复用实际修订份额来证明计划正确 | 修复参考实现；原失败保留于review-20261023-r2，待复审 |
| V2-56 | ADR 0019整份原样保留，含§11全部A–D选项；未选择方案或改变原LAW-6失败判定 | 继续待用户决定 |

## 2026-10-23 追加：推论 6 选择 A（当前参考版本）

| 编号 | 原文位置 | 差异 / 替代保证 | 性质 | 状态 |
| --- | --- | --- | --- | --- |
| V2-60 | v2.0 §5.3 推论6；V2-56；ADR 0019 §11.5 | 用户选择方案A，保留候选B，接受失去固定ρ下 E/C≤w_max/(1−ρ) 的总量上界。替代保证：任意固定旧组的权重最终趋于0（已证明充分条件：固定份额、后续每步固定非零有效权重，固定0<h<1或当前自适应h且h₀n_h>0）；在固定h、P=0、先一致支持后持续反对的单倾向场景，跌破π=1/2的适应时间随一致历史长度非递减增长，无统一常数上界。极限限于实数递推，binary64遇精度耗尽仍显式故障。保证只适用于当前参考版本；“长期稳定后几次反例应很快察觉”是现有h估计的已知不足，P5-30将对照含变点检测的写法，再决定是否以新规律版本替换。原推论6与LAW-6原文保留，LAW-6继续按原判据报告失败 | 用户确认的保证变化；未改参考公式，不代表整份ADR或规律发布获批 | 用户已确认（2026-10-23） |

## 2026-10-25 追加：未定义清单第 1 批（H 的根基，ADR 0025）

来源：用户 2026-10-25（Perth）在「Echo Core 未定义清单」中逐条确认的 H-1 至 H-6。V2-61 至 V2-63 已被 PR #49、#72 使用；V2-64 至 V2-69 留给 P5-35（PR #84）。

| ID | 原文位置 | 读法 / 补充 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-70 | v2.0 §1（H 只追加）、§12 | H 的增长：全部保留原文，永不删除；大块内容存对象、H 存引用，对象同样不可删；存储将满时显式停止接收并通知 u*，不删除、不压缩替代 | 否；补全运行边界 | 用户已确认（2026-10-25） |
| V2-71 | v2.0 §1「外源事件」、§8；CONTRACTS §4.2 | 外源事件的范围：冲着她来或回应她的事（u* 对她说话、触摸与停留、回复、网络上他人对她的回应）与缺席事实算外源事件，进入 C；只关于 u* 自己的桌面观察（前台切换、会议、勿扰等）只进 H 与 G，不进 C；她自己的念头、行动及她触发的程序结果不算。按事件种类定在带版本的规则表里（同 V2-33）。所有进入 H 的事件照常经 F 推动 x | 否；补全定义 | 用户已确认（2026-10-25） |
| V2-72 | v2.0 §4 EV2、§8 N_e | 停留：鼠标当作 u* 的手，停够门限（约 300–500 ms，带版本）即为接触，带部位；接触持续时按由密到疏的节奏报告（如 1、3、10、30 s，之后每 30 s，带版本），每次带已持续时长。同一次接触对 G 归为一组（EV2），对 C 只计一个外源事件，后续报告不增加 N_e | 否；补充 §8 的计数读法 | 用户已确认（2026-10-25）；实现见 P6-17 |
| V2-73 | v2.0 §2.4；CONTRACTS §4.2 | 桌面信号的颗粒度：每次前台切换都进 H，精确到时刻；照常经 IRL、A、M，可以成为 R 的证据 | 否 | 用户已确认（2026-10-25） |
| V2-74 | CONTRACTS §4.2 | 分层：几何只留终端；语义事件全部精确进 H | 否；与契约一致 | 用户已确认（2026-10-25） |
| V2-75 | ADR 0025 §3.5 | 敏感应用排除名单内的窗口：照常记录进程名，不读标题 | 否 | 用户已确认（2026-10-25） |
| V2-76 | v2.0 §2.4；ADR 0026 §5.2 | 她查自己的记忆：能查到原文与精确时间；可查范围不设上限，只限单次返回量，可翻页；无命中、无权限、索引滞后分别返回。解释记录是否可查按 V2-14 | 否；补全 c_task 的读法 | 用户已确认（2026-10-25） |

## 2026-10-25 追加：未定义清单第 2、3 批

来源：用户 2026-10-25（Perth）在「Echo Core 未定义清单」中的决定。第 2 批中关于念头的 N-1、N-2、N-6 与 COG-01 至 COG-05 记在 PR #82（ADR 0026，COG-06、COG-07），这里只登记其余各项。「用户委托 Claude 确认」表示用户明确表示看不懂细节，委托 Claude 判断后确认，实现时仍须经独立审查与逐位对照。

| ID | 原文位置 | 读法 / 补充 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-77 | v2.0 §1「外源事件」、§8；V2-71、V2-72；#78 问题 3 | 缺席（她运行时 u* 离开）：空闲超过门限或锁屏即为开始；离开期间按由密到疏的节奏报告（如 30 min、1、2、4 h……，带版本），每次带已离开时长；u* 回来为结束。同一段离开对 C 只计一个外源事件，后续报告推动 x、不增加 N_e。u* 在电脑前但没有理她不算缺席，她可在思考中注意到，只影响 x。u* 的承诺（「一小时后回来」）不另加规则，由评估参照 | 否；补全定义 | 用户已确认（2026-10-25） |
| V2-78 | v2.0 §6、§13；#78 问题 4 | 停机（关机或程序关闭）：开机时记一条「停了多久」的事实，不算 u* 的缺席，不进 C；x 按实际经过时间经 Φ 回落，不冻结；不补写停机期间没有发生的思考 | 否 | 用户已确认（2026-10-25） |
| V2-79 | v2.0 §9.4、§12；V2-54；P5-20 卡 2026-10-23 追加；#78 问题 5 | 主动：不设行为上的次数与间隔限制，§9.4 的预算、最小间隔、每小时上限只作保险丝（防程序出错或被利用），取值在正常相处中不应触发，由 u* 设定；何时来找 u* 由她从 u* 的回应（EV4）与教导（EV7）中学会。勿扰、安静时段与 u* 的「安静模式」照常生效（u* 的边界）。对话中的追问、自主身体动作、内在活动都不计入。代价：初期可能打扰 u*，靠回应与教导学习 | 是；改变 §9.4 门控的角色 | 用户已确认（2026-10-25）；P5-20 的 `config/opportunities.json` 初值另按 ADR 修改 |
| V2-80 | V2-43；v2.0 §12「容量」 | 取代 V2-43 的移出规则：最新提出的 K 个候选受保护（K 暂定 1），其余候选中移出证据量最小的，平局按倾向 ID；已成为性格的不移出。容量与 K 是保险丝，容量设宽使日常基本不触发。候选移完仍超容量时暂停成长（GROWTH_PAUSED）并通知 u*，事件照常进 H，处理后补算，不以故障中止。被移出倾向的证据仍在 H，再次提出时按 EV5 全部收集 | 否；改写重建读法 | 用户已确认（2026-10-25）；玩具模拟见清单 G-6，在 P5-30 用生活情景再检验 |
| V2-81 | V2-45；v2.0 §5.2「变化率」、§4 EV7 | 教导组按 V2-45 原文。翻转计数采用读法 1：跳过 S = 0 的组后，相邻两组方向相反记一次翻转（即现有实现与 check_law.py）；中性组不掩盖方向的变化 | 否；登记对「相邻」的读法 | 用户已确认（2026-10-25），解除 V2-45 的阻塞 |
| V2-82 | V2-13、V2-42；v2.0 §10 | 关于人的倾向按 V2-13、V2-42 确认：亲近对应联结，信任对应好奇与成事（能力型信任）。已知缺口：信任不含「是否诚实」，被骗不会直接降低信任，目前靠「经历最终胜过教导」兜底；在网络生活（P6-14、P6-15）之前补上，候选做法是按一个人说过的话后来被证实或推翻的记录 | 否；缺口待补 | 用户已确认（2026-10-25）；缺口登记为后续 |
| V2-83 | V2-38、V2-39、V2-40、V2-41、V2-44、V2-46、V2-47（含 2026-10-22 更正） | 按登记内容确认；其中 V2-44 大部分已被 ADR 0019 §8 取代，V2-38 一步内的次序以 ADR 0019 §6 为准 | 否 | V2-39、V2-46 用户已确认；V2-38、V2-40、V2-41、V2-44、V2-47 用户委托 Claude 确认（2026-10-25） |
| V2-84 | V2-55 至 V2-59；ADR 0019（PR #61） | 确认这些条目已随 ADR 0019 在 #61 合并时采纳，登记表中「待用户确认（阻塞）」的状态过时。V2-59 的存储实现原在 PR #72，已关闭，第 3 批定完后重新开卡 | 否 | 用户委托 Claude 确认（2026-10-25）；G-11、G-12 为用户亲自所定 |

## 2026-10-25 追加：持续认知方案（ADR 0026，待审查）

用户授权最终方案自审并提交 PR。以下是 [ADR 0026](../adr/0026-continuous-cognition.md) 的显式扩展登记，未修改现行原文、运行代码、已发布契约或任务卡。状态均为待交叉审查、待用户合并采纳（阻塞后续实施）；不是运行验收通过。COG 前缀仅用于本方案，不重排其他分支的 V2 编号。

| 编号 | 当前依据 | 扩展 / 影响 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| COG-01 | ARCHITECTURE §3.4；v2.0「她是谁」、§9.4；S §20 | 清醒 RUNNING 与局部活动结束/等待分离；内部一轮接一轮，不再把需要时运行或随机主动机会作为内部接续条件 | 修改运行规则及主动调度的适用范围 | 待审查、待合并采纳；ADR 0026 §3、§8 |
| COG-02 | v2.0 §2.1、§2.4；I02/I18/I21；state.proto CoreTransactionV2 | 新增 H 内部活动记录、Q 工作状态及专用原子提交后继；纯内部记录不造 IRL/A/M、不新增 F 内容驱动，G/C 恒等 | 新记录/提交分支；不改 F/G/C 数值规律 | 待审查、待合并采纳；ADR 0026 §5–6 |
| COG-03 | S §11；ADR 0012 §7；CONTRACTS §5.1 | m 下模型提出下一查询与关注；查询执行保持确定/有界，c_core 保持独立；撤回传播至工作引用、摘要及在途输入 | 补齐有类型认知输入/结果及查询承接 | 待审查、待合并采纳；ADR 0026 §4–5 |
| COG-04 | v2.0 §9；S §19；ADR 0003/0012 | 持续认知由本地严格载体承担；每请求一个 m，合法零值可通过零路径；B/C 不自动取得严格连续认知结论 | 新增认知链的严格范围，不重写既有说话模式 | 待审查、待合并采纳；ADR 0026 §7 |
| COG-05 | ADR 0007 §10–11；I15/I16/I18；N0 P0-09 | 新类型/编码与迁移由后续合同发布；声明抢占、资源公平、实际时间和延迟/功耗验收；自审不代替行为或交叉审查 | 新合同与候选验收需求；不放宽原门槛 | 待审查、待合并采纳；ADR 0026 §9–10 |

## 2026-10-25 追加：持续认知的用户确认与修改（ADR 0026）

用户在「Echo Core 未定义清单」第 2 批确认 COG-01 至 COG-05，并按 N-1、N-2、N-6 修改。以下更正前文 COG-02 的相应叙述，原记录保留。

| 编号 | 更正 / 补充 | 状态 |
| --- | --- | --- |
| COG-06 | 更正 COG-02「纯内部记录不造 IRL/A/M、不新增 F 内容驱动」：内部记录在 m = 0 下经 IRL、A、M 并记录，只用于 F 推动 x 和 M 提出候选倾向（假设）；G 不把它们计为证据，C 恒等。念头不能只靠反复想就让自己越来越确信（N-1）。稳定性要求有界且不自锁（N-6），新增验收 COG-A12；H 的增长实测为 COG-A13；实现前置条件为 P0-10 与 3080 同驻实测 | 用户已确认（2026-10-25） |
| COG-07 | 交叉审查（PR #82）后的修改，用户 2026-10-25 批准：内部事件对 x 的推动乘 κ^{n(e)}，n 为自上一个计为新的外源事件以来的内部事件数，κ ∈ (0,1) 为规律参数；保证两次新外源事件之间内部推动总量有界、x 收敛到 x_rest，校准使回落时间上界 ≤ T_x（v2.0 §6）。内部提出的候选按 ADR 0026 §6.10 由 G 处理，只回溯真实旧事件。是对 v2.0 §6 F 的扩展，发布为新规律版本的一部分 | 用户已确认（2026-10-25）；待复审 |
| COG-08 | 复审（PR #82 第二轮）后的修改，用户 2026-10-25 批准：收紧 COG-07 的保证范围。事件分三类：新外源事件（原式推动 x，重置 n）；外部的其他事件（只关于 u* 的桌面观察、同一次停留或缺席的后续报告，原式推动，不改变 n）；自致事件（内部记录与她自己的行动、工具、程序结果，按 κ^n 加权，只有内部记录递增 n）。收敛保证只针对纯内生区间（没有新外源事件和外部的其他事件），并要求部署声明时间包络 B_N，T_ret = B_N + ln(4/ε)/λ_min ≤ T_x；越出适用域时验收记为「超出适用域」。外部真实刺激持续存在时维持偏离不算自锁 | 用户已确认（2026-10-25）；待复审 |
| COG-09 | 第三轮复审（PR #82）后的修改：行动结果的分量取发出它的那一轮的 κ^{n_发出}（记录在 Q 的在途行动里），不按返回时的 n；每轮至多发出 r 个行动，同时在途至多 M 个，每个行动有结果期限 τ（r、M、τ 为带版本的保险丝）；超过期限才返回的结果照常记入 H，但不直接经 F 推动 x，她可在之后的思考中读到并经内部记录产生感受；跨新外源事件的在途行动保留各自的分量与期限。回落时间改为 T_ret = B_N + τ + ln(4/ε)/λ_min；「超出适用域」只由前 N 条内部记录未在 B_N 内提交或资源不符触发 | 用户已确认（2026-10-25）；待复审 |
| COG-10 | 第四轮复审（PR #82）后的更正：COG-09 的结果期限 τ 从行动发出算到该结果**实际经 F 提交**为止，不是到返回为止；按时返回但排队后才提交的结果同样算超期，记入 H 但不直接推动 x。期限由 Core 在提交时用可信时钟对照 Q 中的发出时刻判定，判定随提交记录、重放沿用 | 用户已确认（2026-10-25）；待复审 |
| COG-11 | 第五轮复审（PR #82）后的更正：结果期限 τ 的起点是发出它的那条内部记录的原子提交时刻（行动在此登记进 Q 的 outbox），Q 记下绝对截止点；outbox 等待、实际发送、执行与结果处理共用同一份 τ，发送或重试不刷新期限。据此第 j < N 轮的起点不晚于 t_N，T_ret = B_N + τ + ln(4/ε)/λ_min 的推导成立 | 用户已确认（2026-10-25）；待复审 |

## 2026-10-26 追加：未定义清单第 5 批（运行、权限与成长暂停）与她的一生（ADR 0028）

用户 2026-10-26（Perth）在对话中逐条决定或采纳 Claude 的建议，各行「状态」标明来源。

| ID | 原文位置 | 读法 / 补充 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-85 | V2-12、V2-15、V2-16、V2-17、V2-36；未定义清单第 5 批 R-3、R-5、R-6、R-7 | 按登记内容确认。R-6（V2-16）中「受全部门控」按 V2-79 理解：保险丝与 u* 的勿扰、安静时段，不含行为上的次数限制。R-5、R-6 已随 #76、#57 合并实现 | 否 | R-3、R-5、R-6 用户已确认（2026-10-26，无异议）；R-7 用户委托 Claude 确认（2026-10-26） |
| V2-86 | V2-10；v2.0 §5.4、§12；第 5 批 R-1 | 成长异常时自动暂停的语义按 V2-10 保留。设计目标（用户 2026-10-26）：**正常相处中永远不触发；触发即说明程序出错或被利用**，同 V2-79 对保险丝的定法；阈值按此目标校准，不按「她变化太大」校准。检查只能推迟、不能改写：规律无误时结果与没有检查相同。待定，移入第 4 批（θ）一起定：探测情境与阈值；复核由谁做、怎么做（候选：机器用独立实现从 H 重算并核对不变量、列出引起变化的证据组，u* 只确认这些事件是否真实，不真实按来源更正） | 否 | 方向用户已确认（2026-10-26）；细节待第 4 批 |
| V2-87 | V2-11；第 5 批 R-2 | 暂停、恢复、复核暂缓与复核结论除写进 L 外，作为**系统记录**写进 H：不经过 F，不是外源事件，不是 G、C 的证据，不推动 x；她查自己的记忆时能查到（V2-76）。她读到这类记录时的反应按 EV8 是她的经历。从备份恢复的记录（ADR 0028 D4）同样处理。这是 V2-71「每一条进入 H 的事件都照常经过 F」的例外：系统记录不是发生在她身上的事件，是对她的维护记录；重算时按 H 次序重放，结果仍逐位一致（§2.2） | 否；补全 | 用户已确认（2026-10-26） |
| V2-88 | V2-14；ADR 0025 §4 第 7 条；ADR 0026 §6.3；第 5 批 R-4 | 取代 V2-14 的第 2 点。第 1 点不变：解释记录（a(e)、μ、候选、标注）不进入 c_core，天性评估、情境吻合与证据收集看不到她过去的反应（EV5、EV6）。第 2 点改为：说话和回想时（c_task 与她的记忆查询），可以查到她当时的反应，并标明「这是她当时的反应」，不冒充事实；由此在对话或思考中产生的内部记录按 ADR 0026 §6.3 递减加权，不会因此自锁。内部记录的内容可能引述她过去的反应，A 评估这条内部记录时会受内容影响，这是「越想越难过」的来源，不违反第 1 点：第 1 点限制的是 c_core 的查询计划，而内部记录的评估结果按 ADR 0026 §6.2 只用于 F 和提出候选，不是 G、C 的证据（EV3）。代价：对话模型能看到她的情绪史，她可能提起旧事 | 否；V2-14 的 c_task 部分被取代 | 用户已确认（2026-10-26）；影响 P5-04（PR #65 事实视图）的 c_task 构造 |
| V2-89 | V2-18；ADR 0011 第 3、10 条；v2.0 §11；CONTRACTS.md §6；第 5 批 R-8 | 社交权限按用户 2026-10-26 的决定修改，原则是「给她和人一样的权力，不预设，只上保险，错了就教」：她以自己身份发的消息（`social.message`）**默认信任**；她以自己名义做的约定，默认信任；**付款只用她自己的小钱包**，从钱包付钱是她花自己的钱，默认信任，钱包只能由 u* 充值、有余额上限、不绑定 u* 的卡、不能透支、不自动充值；涉及 u* 的事（约 u* 的时间、代表 u* 表态）不需要事先同意，保险是**事后通知 u***，u* 据此教她。不变：u* 的账户、密码与秘密在结构上隔离；诚实底线发出前检查；新认识的人数与发出消息数的保险丝。网络生活仍在 V2-90 的检验通过后开启 | 是；改变 v2.0 §11「会让 u* 承担的事要 u* 同意」与 ADR 0011、CONTRACTS §6 的 `social.commit` 一律 T3 | 用户已确认（2026-10-26）；契约修订另开卡（契约变更，交叉审查） |
| V2-90 | V2-82；第 5 批讨论 | 信任缺口的补法：**不增加新的倾向**，走自然路线。被骗吃亏时，成事的负反应是反对信任的证据；被骗没吃亏时，模型在 m = 0 下可提出普通候选倾向（如「这个人说的话不太可靠」），由证据决定能否长成，经 m 体现为谨慎。验收放在实验谱系：合成世界中放一个反复撒谎的人，检验她能否在登记的经历量内学会（信任下降或警惕长出）；不通过再讨论。代价：她会被骗，可能不止一次，钱包余额是保险丝 | 否 | 用户已确认（2026-10-26）；网络生活（P6-14、P6-15）之前完成检验 |
| V2-91 | V2-37；v2.0 §13；第 5 批 R-9 | genesis 的读法按 V2-37 记录，暂不定稿。用户 2026-10-26 提出的方向：很多人一开始没想好名字，首次启动时用诞生界面引导 u* 逐项讨论并填写诞生清单（名字、u* 是谁、权限、钱包、读取开关、保险丝等），诞生不可撤销须讲清；Claude 建议允许不带名字诞生，名字之后作为一次普通经历给出（需改 v2.0 §13「名字是 H 的第一条事实」）。以后再调整 | 待定；可能改 v2.0 §13 | 记录，待后续讨论（用户 2026-10-26） |
| V2-92 | v2.0 §13、§18；V2-78；ADR 0012；ADR 0028 | 她的一生（ADR 0028）：D1 长时间离开关闭她的选项默认关闭；D2 删除是 u* 的权利，提示备份、两次确认、不写进 H、系统不替 u* 告诉她；D3 只读查看器，查看不写进 H；D4 **同一时刻只能有一个她**，手机上也是同一个她，换电脑封存旧存储，只在当前存储丢失或损坏时允许从备份恢复，恢复写系统记录，备份点到恢复时刻按停机处理；D5 v1 不支持转交，列入 §18 | 是；补充 v2.0 §13、§18 | D4 用户已确认；D1、D2、D3、D5 用户采纳 Claude 建议（2026-10-26） |

## 2026-10-26 追加：未定义清单第 4 批（θ 动力学）与成长异常检查

用户 2026-10-26（Perth）在对话中逐条决定或委托，各行「状态」标明来源。T-2 至 T-5 原是 PR #77（P5-25）里编号「待定」的临时做法；本节定稿后由 #77 按此继续。异常检查的细节由第 5 批 V2-86 移入。

| ID | 原文位置 | 读法 / 补充 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-93 | ADR 0019 §8；P5-25 THETA-1；#77 BLOCKED；第 4 批 T-1 | THETA-1 的跨语言逐位对照，新增一个按 ADR 0019 §8 显式运算次序（平方用 `x*x`）写的 binary64 参考，Rust 与它严格逐位比较；原 Python 检查（`**2`）、它的失败记录与原 THETA-1 的性质断言全部保留，不把原失败改写为通过。原因：`**2` 在 37 处与正确舍入差 1 个最低位，已用精确算术独立核实 | 否 | 用户委托 Claude 确认（2026-10-26） |
| V2-94 | v2.0 §8「ĥ_θ 是外源反应序列中变点出现的频率」；#77 METHODS；第 4 批 T-2 | 「世界变了」的参考检测：四个天性通道各自的双侧高斯均值变化 CUSUM，固定次序计算；任一通道达到门限算一次联合变点。参数 shift_sigma、threshold、n_h 在实验谱系校准，测试前登记。已知局限：看不出只有波动变化、长期低幅变化，对重尾或相关噪声不作保证。更好的方法可作为新规律版本替换，从 H 重算 | 否；补全参考形式 | 用户已确认（2026-10-26）；最终检查由审查方完成 |
| V2-95 | v2.0 §8「τ̂_i 是外源反应 a_i − b_i 同号持续时间的加权平均」；V2-78；#77 METHODS；第 4 批 T-3 | 恢复速度的持续时间按以下修改：(a) **只计她运行的时间**：同号段的时长由 H 中可重放的事件时间与停机记录（V2-78）算出，扣除停机区间；重算时不重新测量机器运行时间。这只作用于 θ 的时长统计，x 经 Φ 的回落仍按实际经过的时间（V2-78 不变）。(b) **平均改为对数平均（几何平均）**：每个完成段 T > 0 以 ln T 计入，τ̂_i = exp((n_tau · ln τ₀ + Σ ω · ln T)/(n_tau + Σ ω))，权重 ω 的递推与 #77 METHODS 相同（每个外源事件乘本步的 ρ）；ln、exp 属于版本化数值定义（ADR 0019 §8）。理由：持续时间是时间尺度，差别在倍数；算术平均会被单段极长的持续时间拉偏（模拟：300 段约 10 分钟的心情加一段 14 天，算术平均 14.6 → 81.6 分钟，几何平均 11.0 → 11.2 分钟）。(c) **零时长段**（同一时刻开始又结束）不作为完成观测，不进入对数和与计数，在估计器状态中单独计数供审计；不补任何 ε 时长。(d) **未结束的一段，本版本不计入**（同 #77 原候选：视为右删失，不把当前长度当作完整观测）。把它作为删失观测参与估计（完成段用密度项、未结束段用生存项）是后续规律版本的候选，须先定出估计模型与先验、同一未结束段多次观察时的更新、闭合时的替换、无完成段时的估计、权重随 ρ 的更新、求解与舍入的版本化数值定义，且不得用当前 λ 外推尾部（LP8）；定稿后按新规律版本从 H 重算。只在外源事件到来时更新（LP5）。n_tau 列入第 16 节待校准量 | 是；改 v2.0 §8 的「加权平均」 | 用户已确认（2026-10-26）；(a)(b) 随 T-4 的选择 D 一并采纳，用户可在合并时更正；(c)(d) 按交叉审查（PR #90）修订，(d) 由「删失观测」改为「本版本不计入、删失留作后续版本候选」；最终检查由审查方完成 |
| V2-96 | v2.0 §12「单个事件引起的 θ 变化不超过 Δθ_max，触顶按跳变处理」；#77 METHODS；第 4 批 T-4 | **上限只用来抓程序错误。** Δθ_max 不是「她最多能变多少」的设定，而按参考公式推出的界设定，使合法结果不可能超出；判定用严格大于（等于界不触发）。两层界：(1) **范围界（总是成立）**：abs(Δb_i) ≤ 2，abs(Δg_i) ≤ g_max − g_min，abs(Δλ_i) ≤ λ_max − λ_min；(2) **单事件界（更紧，由递推恒等式推出）**：设本步实际保留率为 ρ（含 ĥ 的变化）、旧有效权重和为 S、当前反应为 a，按先衰减后加新事件，b′ − b = [n_b(1 − ρ)(b₀ − b) + (a − b)]/(n_b + ρS + 1)，其中第一项是旧证据衰减后先验重新占比造成的变化，即使 a = b 也可能不为零；对 v = σ̂²、v₀ = σ₀² 同理 v′ − v = [n_σ(1 − ρ)(v₀ − v) + (a − b)² − v]/(n_σ + ρS + 1)；g 经倒平方根与 clip，令 K = g₀σ₀，abs(Δg) ≤ min(g_max − g_min, K · abs(v′ − v)/[√v′ √v (√v′ + √v)])；h ≥ h_min > 0 时 S ≤ 1/h_min，可给出方差的正下界。λ 的单事件界随 V2-95 的时长算法与时间输入范围推出。这些是证明用的实数关系，不替代规范的逐步浮点递推；浮点余量按实际运算分析。**保证范围**：超出界必是违约（程序错误或输入违约）；界内的程序错误不保证能检出。触发时整个事件保持事件前的 θ 与估计器状态（不截断后继续），按 V2-97 处理。不悄悄截断：截断是隐藏的预设，且使输出与估计器状态不一致 | 是；改变 Δθ_max 的含义与取值方式 | 用户已确认（2026-10-26，选项 D）；界的写法按交叉审查（PR #90）补正 |
| V2-97 | v2.0 §5.4「大的变化」、§12 跳变检查与缓慢漂移检查；V2-10、V2-86；AC2-20、AC2-21；第 4 批 | 成长异常检查**只报警，不暂停**（用户 2026-10-26 选择：她永远不需要暂停）。原则：保险是防我们的错误，不是防她自己的生活；它提醒开发方，不刹住她。(1) **实现一致性检查（机器）**：触发时用独立的第二套完整实现从 H 重算，核对不变量。它只能判定程序错误，不能判定规律正确或来源真实。(2) **规律复核照常进行，但不暂停**：变化照常生效，同时通知开发方与 u*；开发方按 §5.4 复核规律本身对不对，不对结果投票（AC2-20 不变）。规律有错时发布新规律版本，从 H 重算 (R, θ)；来源不真实时 u* 作来源更正，从 H 重算。(3) **唯一不应用的情况是程序故障**：结果在数学上不可能（超出 V2-96 的界、非有限数等数值违约）或两套实现不一致时，那不是成长而是故障，同 §12 结构故障：这次更新不应用，成长等修复后从 H 补算；她的当下（x、对话、行动）照常。第二套实现缺失、失败或超时时无法判定，按 (2) 照常应用并报警。(4) 阈值在实验谱系校准：正常相处（含真实的大事）不触发，注入的程序错误与操纵会触发；测试前登记，看结果后不放宽。(5) 探测情境只用 8 个中性探测：四个天性通道各一正一反，不写与性格有关的情境。检查结果、通知、复核结论与故障判定绑定规律版本与 H 前缀，作为系统记录写进 H 与 L（V2-87），重放沿用记录，不受重算耗时、当前时钟或重新调用模型影响。**代价（用户已知并接受）**：规律设计有错时，她会带着错误的状态生活到修复为止，那段时间她的言行和别人的回应作为真实经历永久留在 H；回应回路失控时，修复前她会继续滑向边界。 | 是；改 v2.0 §5.4、§12「暂缓应用并复核」为「照常应用、报警并复核」；AC2-21 的跳变检查部分随之由「暂缓」改为「报警」 | 用户已确认（2026-10-26，选项 Y：先应用、后复核）；第 (3) 条程序故障不应用由 Claude 补充，用户可在合并时更正；按交叉审查（PR #90）区分实现一致性检查与规律复核 |
| V2-98 | V2-34；`tasks/requests/P5-25-dynamics.md`；第 4 批 T-5 | θ 估计器的全部递推状态（各通道的权重与和、σ、持续时间统计、未结束段、检测器累积量、ĥ、慢漂计数，绑定规律版本与参数摘要）要有正式的存储契约：规范编码、校验、完整恢复与迁移。另开契约变更卡，按 #77 的契约请求列出的内容定义；在此之前，P5-25 的内存结构不当作已冻结契约 | 否 | 用户委托 Claude 确认（2026-10-26）；契约卡待开 |
