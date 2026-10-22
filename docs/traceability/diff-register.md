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

## 2026-10-21 追加：P5-23 规律引擎 G 的规范定义

来源：任务卡 P5-23。v2.0 第 5.2 节把计算次序定为规范定义，但有几处没有写到；卡片要求本卡写定并登记（「待用户确认，不阻塞」）。完整定义见 [P5-23.md](P5-23.md) 第 1 节 D1–D11。原文不改。V2-48 是契约缺口，单独列为阻塞；V2-49 是 AC2-01、AC2-07 在有新近权重时的读法。

| ID | 原文位置 | 重建或读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-38 | v2.0 第 4 节 EV7、第 5.2 节「计算次序」 | 组在 H 中的位置取第一个成员的提交位置（教导组同样取第一次出现）。一个分配 (c, k) 第一次建立时得到序号（k 的第几组），此后不变。一步处理一个已提交事件：先按倾向 ID 的规范次序建立新倾向（候选、第一次涉及的人的关系倾向），再按组的位置升序（平局按 group_id）逐组处理，最后执行容量保险丝。没有证据项的一步 R 逐位不变 | 否；补全次序 | 重建，待用户确认 |
| V2-39 | v2.0 第 4 节 EV2、第 5.2 节 | 后来加入组的成员：用本步的 b、σ̂、r̃ 重算这一组的 Δ_c、峰值和各分配的 S、p_c、强度；分配原地更新，序号不变，不算新的一组，不使旧证据再衰减；受影响的倾向按序号重放递推。教导组的重复不改变组的任何值 | 否 | 重建，待用户确认 |
| V2-40 | v2.0 第 5.2 节「份额」 | 份额取 π 的时刻：一组的分配在某一步建立或重新分摊时，各倾向取本步处理这一组之前的 π（同一步中先处理的组已计入）。组的内容没变、只是吻合的倾向变了时，已有分配只改份额，S、强度、p_c 保持原值 | 否 | 重建，待用户确认 |
| V2-41 | v2.0 第 2.2、5.2 节；EV5；ARCHITECTURE 第 5 节「M 之后在后台补做」 | 补做的解释（M 降级后补做的 μ、EV5 的回溯）在追加的位置按次序计入：对相应的倾向是新的一组，得到新序号，先乘 ρ 再加入；b、σ̂ 取追加时本事件之前的 θ；同一组里其他倾向的份额按 V2-40 调整并重放。增量计算与整段重算都按追加位置处理，所以仍然逐位一致 | 否 | 重建，待用户确认 |
| V2-42 | v2.0 第 5.2 节「可信」、第 6、7、10 节；V2-13 | 关于人的倾向的写法：每人至多一个亲近、一个信任，ID 为 `rel:closeness:<person_id>`、`rel:trust:<person_id>`；谓词各一条（κ = κ₁），概念引用由调用方按注册表给出；亲近 d = (1, 0, 0, 0)，信任 d = (0, 0, 1/√2, 1/√2)（依靠此人能学到东西、能把事做成）；u* 的两个倾向在 R₀ 中：亲近带先验 A_att，信任以初始证据 A_att 起步（随新近权重让位）；读出：亲近度 clip((1 + π)/2, w_min, 1)，信任 (1 + π)/2，没有倾向时按 π = 0；带关系的候选归并到结构倾向。P5-30 的实验 3、4 检验 | 否；补全参考形式 | 重建，待用户确认 |
| V2-43 | v2.0 第 12 节「容量」 | 每步的证据组处理完后执行。证据量 V_k = (B_k − n₀ − κ_k) + A_att（只在依恋上有先验）；在候选（π ≤ ½）中移出证据量最小的，平局按倾向 ID 的规范次序排前的先移；移出后同一边的其他倾向重新分摊该组并重放，使 I25 的「每一边之和为 1」继续成立；已成为性格的不移出，候选移完仍超容量时为显式故障 `CapacityExhausted` | 否 | 重建，待用户确认 |
| V2-44 | v2.0 第 2.2 节、第 5.2 节 | 数值的规范写法：组内均值先在 binary64 里算各成员的 a(e) − b，再精确求和、除以成员数后正确舍入（与次序无关，成员整体重复任意次逐位不变）；内积按通道次序累加；B 的递推写作 B ← c₀ + ρ(B − c₀) + W\|S\|，使 n₀ + κ 不衰减；ĥ 的下限给出时取 max；存下的 −0 归一为 +0；NaN、Inf、计数溢出是显式故障；舍入使 \|A\| > B 或 π 落到 ±1 时是显式故障 `PrecisionExhausted`，在那一步报告；M、U 分别舍入，U = 0 而 M ≠ 0 合法，r_k = 0 | 否；数学上与参考形式相同 | 重建，待用户确认 |
| V2-45 | v2.0 第 4 节 EV7、第 5.2 节 | 教导组：只分给被教的倾向，S = ±1（μ 视为 1），强度 w_teach，可信取处理该组之前的信任 (1 + π)/2，没有信任倾向时为 ½；教导组不进入 r_k 的加权平均（只让 k 的 M、U 乘 ρ，与其他旧证据一样衰减）。S = 0 的组照常记入（旧证据乘 ρ，组数加一，符号不变），与参考核验一致 | 否 | 重建，待用户确认 |
| V2-46 | ADR 0014 第 4 条；v2.0 第 7 节 | 别名：候选概念的规范代表与已有倾向或本步先建的候选相同时不建立新倾向，本步的吻合项改指那个倾向，组里已有它的吻合项时保留原项。候选的倾向 ID 取其记录的 record_id | 否 | 重建，待用户确认 |
| V2-47 | v2.0 第 2.5 节 | L 条目：本步中值有任何变化的倾向各一条 TendencyChange（π 前值、后值、本步处理过的证据组），按倾向 ID 的规范次序；entry_id、event_id 由外层事务填写（P5-29） | 否 | 重建，待用户确认 |
| V2-48 | V2-34；v2.0 第 5.2 节「典型幅度」「计算次序」；I25 | 契约缺口：V2-34 的存储字段不够 G 用。（1）r_k 的加权平均需要分子 Σω·μ·p_c 与分母 Σω·μ，`Tendency` 只存截断后的值；（2）回溯分摊、后来的成员、容量移出后的重新分摊都要改旧分配，需要按序号重放，`EvidenceAllocation` 缺序号、μ、p_c。建议另开契约变更卡补这 5 个字段（`reports/P5-23/BLOCKED.md`、`tasks/requests/P5-23-law-state-idl.md`）。本卡不改 `core/contracts/` | 否；需要契约变更 | 用户 2026-10-22 同意方案 a：由 P5-32（ADR 0016、V2-50）补齐。P5-32 已合并（#52），本卡已换成生成的类型，临时结构已删去（P5-23.md D12） |
| V2-49 | v2.0 第 5.1 节 LP1、LP7；第 17.2 节 AC2-01、AC2-07；推论 4「不计新近权重」 | 有新近权重时，一组证据到来也让旧证据乘 ρ（LP4），所以 LP1、LP7 按两种意义检验：不计新近权重（ρ = 1，同推论 4），以及这一组相对于同样衰减之后的作用。在参考形式下，强的旧支持之后来一组弱的支持，π 可以下降，因为旧证据按 ρ 让位（反过来对相反证据同理）。测试 `ac2_01_reference_form_lets_old_support_give_way` 记下了这一点；这是参考形式的行为，不是实现的偏差 | 否；验收的读法，不放宽 | 待用户确认 |

## 2026-10-22 追加：P5-32 契约变更

来源：ADR 0016 与任务卡 P5-32（用户 2026-10-22 同意方案 a）。V2-48 在 P5-23（PR #49）中登记。

| ID | 原文位置 | 重建或读法 | 冻结边界 | 状态 |
| --- | --- | --- | --- | --- |
| V2-50 | V2-34、V2-48；v2.0 第 5.2 节「典型幅度」「计算次序」；I23、I25 | R 的存储补五个字段：倾向存典型幅度递推的 M = Σω·μ·p_c 与 U = Σω·μ；每个分配存序号、μ_{c,k} 与 p_c（教导组缺省）。以后继消息 TendencyV2、EvidenceAllocationV2、EvidenceGroupV2、RSnapshotV2 实现，旧消息标为取代。改旧分配时，G 只用存下的 S、强度、份额、可信、可核实，以及序号、μ、p_c 重放该倾向的递推，不需要历史的 b、σ̂ 或其他倾向的 π 历史；与整段重算逐位一致。解决 V2-48 | 否；只增，不改已发布字节 | 待用户批准，ADR 0016 |
