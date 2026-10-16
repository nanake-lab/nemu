# Specification baseline — P0-01 template

P0-06 fills this template; hashes have not been verified here.

| Original file | Document title | Version | SHA-256 | Priority |
| ------------- | -------------- | ------- | ------- | -------- |

Record missing Review_and_Remediation sections and impact explicitly. See docs/echo-core/README.md for precedence.

## P0-06 verified baseline — 2026-10-14

This appended record supersedes the unverified template above. Originals were opened read-only as DOCX ZIP/XML and hashed with `Get-FileHash -Algorithm SHA256`. No original was changed. Paths below are relative to `docs/echo-core/original/`.

| Alias | Original file | Internal title / version | SHA-256 |
| --- | --- | --- | --- |
| F | Echo_Core_v0.6.4_Frozen_Architecture.docx | Echo Core 可塑状态转移核心；架构与形式化规格 v0.6.4 修订版 | D56D48E60A445C926B5A8C974BEF11B790A9ED552CABB8AF50EB72CE55663F2A |
| D | Echo_Core_v0.6.4_Integrated_Executable_Implementation_Delivery.docx | Echo Core v0.6.4；可执行架构与可塑机制施工合同；Integrated Implementation Delivery v1.0 | 8C57EB1EEAE9AB3AF9B7DE599C3DC553FAAFBA85EF92A7BEE5DCFE0A3BF6B83E |
| R | Echo_Core_v0.6.4_R_G_Plasticity_Implementation_Spec_Final_Audit.docx | Echo Core 可塑状态转移核心；R / G 可塑机制实现规格；Final Audit Draft / Implementation Spec v1.0 | CAB94D3F834E707EECE4D563EB17B53D6A895700AAB2294E61E9F4329E696D1D |
| O | Ada Echo Core Math define.docx | Echo Core Ada 核心算子与整体动力学规格；版本 1.0 冻结候选（数学闭合修订），2026-09-23 | D735D030C5EE708D824D26C3F3993DA0FFBABD3D1139C92F89D3F617FF18E638 |
| S | Echo Core Implementation Plan.docx | Echo Core Ada 全系统施工总方案；v1.0，2026-09-23 | 7F7174078BA4E26F43FA0BE66501660A19BE6CC0D2B18568A39C074FF6113174 |

The aliases identify original files, not replacement specifications. Readable counterparts are respectively `v0.6.4-frozen-architecture.md`, `v0.6.4-integrated-implementation.md`, `v0.6.4-rg-plasticity-spec.md`, `ada-core-operators-spec-v1.0.md`, and `echo-core-system-plan-v1.0.md`, under `docs/echo-core/`. DOCX wins over conversion differences.

### Precedence (unchanged from Echo Core README §1)

1. Frozen ontology/invariants in F and D Part I §12.1.
2. Explicitly closed contracts in O v1.0, including I01–I22.
3. Unfrozen engineering choices in D and R.
4. Engineering choices in S v1.0.
5. User-approved Nemu ADRs.
6. Nemu ARCHITECTURE.md and CONTRACTS.md.

This does not silently settle disagreements between a frozen clause and a newer closure. W00-C01 in [diff-register.md](diff-register.md) remains pending user confirmation; [ADR 0006](../adr/0006-single-root-spec-conflict.md) is proposed, not approved.

### Missing upstream material

S §1 and O's upstream references rely on `Review_and_Remediation`: F §§20–27 and D Appendix B. The supplied F ends at §19; supplied D has Appendix A, not B. E12 remains **待用户提供** for those missing sections. Do not reconstruct their θ design. W09/W10 and growth are v1.x; Nemu v1.0 uses genesis b/Λ/B₀, empty R and audited identity G/C. Missing additions do not alone block v1.0 IDL/transaction work. Future growth must re-baseline newly supplied originals and resolve W00-C01 before activation.

**追加状态说明（2026-10-14 决定）：用户确认不可得（2026-10-14 决定）**。此状态取代上述“待用户提供”的当前含义；原文保留作历史说明。范围为 Review_and_Remediation 增补 F 第 20–27 节、D 附录 B。不重建缺失内容；C01 保持未决、仅阻塞 W09/W10，ADR 0006 保持 PROPOSED，v1.0 保持 NO-GROWTH。

Review scope: all five documents' headings and §1; S §§1,2,26,31; targeted original clauses for E1–E14, conflict verification, acceptance and work-package mapping. This is not a claim that all future implementation details have already been audited.

Reproduce: `pwsh -File reports/P0-06/verify-baseline.ps1` from the repository root. Evidence is written to a new timestamped directory, never over prior evidence.
