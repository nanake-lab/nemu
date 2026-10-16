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
