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
