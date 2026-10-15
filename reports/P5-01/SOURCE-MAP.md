# P5-01 source inventory — implementation checklist

The full original filenames and SHA-256 digests are fixed by
`docs/traceability/baseline.md`. F = Frozen Architecture; D = Integrated
Implementation; O = Operators; S = System Plan. These are source pointers,
not a claim that the corresponding message has been implemented.

| Required message / object | Source and fields to preserve | Status |
| --- | --- | --- |
| EventEnvelope | F §4; S §9: id/kind/event and ingest time/parents/payload/provenance; raw location, device/session, stable key, digest, action/trial/correction/derivation | Pending IDL |
| FactProjection | F §3; S §11: bounded content, source, availability, correction chain, index version, truncation and distinct failure results | Pending IDL |
| SemanticCandidate | D I §§4.2,6.2: proposal distinct from accepted graph; input/producer binding | Pending IDL |
| SemanticGraph, Ref, Claim, Act, Edge | F §§3.2–3.4; S §10: ConceptRef predicate, roles/value/polarity/modality/time, source metadata, typed edges | Pending IDL |
| ConceptRef, ConceptProposal | F §3.4; D I §4.4: immutable identity, versioned Registry, signature/grounding/relations/status; no worker-issued valid identity | Pending IDL |
| FastStateSnapshot | D I §5.1; O §5.1: representation version, finite bounded coordinates, explicit dimension | Pending IDL |
| FastDriveProposal | D I §5.3; O §5.2: event/accepted z/input versions, drive, source refs, producer, digest; worker input narrowed by O | Pending IDL |
| ProposalBinding | D I §§3.1–3.7; O §16.1: branch/event/raw position/head/process and anchor times/projected digest/z/fact/versions/validity/read set/payload digest | Pending IDL |
| CoreTransaction, CoreCommitResult | D I §§3,6,7; O §16.2; S §8: staged results, atomic published versions, all required state and runtime records | Pending IDL |
| DegradedStatus, FailureReason | D I §§4.5,5.4,6.2; S §21; README E13/E14: typed reasons and capability intersection; no silent substitute drive | Pending IDL |
| EffectContract | O §3.1; S §5: responsibility, causal revision, owner, endpoint, scope, carrier/parameters/bounds, qualifications/ancestors/dependencies | Pending IDL |
| QualificationCertificate | S §5; O §§6,8,9: domain, root blocks/ancestry/exposure, assumptions, opportunity ID, criteria, result refs, dependencies | Pending IDL |
| ConsolidationSlice | D II §5.2; O §9: exact source revisions, current q/function, scope/endpoint/dependencies, roots/budget, T_promote/ValidityContract | Types only, pending |
| ThetaModule, ResponsibilityBinding | D II §§5.2–5.3; O §§3,9–11: source slice, carrier/parameters/order/scope, owner/disposition; do not invent missing θ mechanisms | Types only, pending |
| TrialResult | O §§6,8; S §§15,20: frozen plan binding, actual outcome references, causally grouped data, explicit failure/unknown | Types only, pending |
| ModulationEnvelope | O §13.2; S §19: snapshot/projection/state digests/carrier/model/tokenizer/hook/coefficients/bounds/receipt | Pending IDL |
| VersionedEnvironment | O §2.1; S §5: explicit Registry/IRL/scope/carrier/model/tokenizer/config and implementation versions | Pending IDL |
| PlasticTraceRevision, trace head | D §0.4, II §1.3: identity/revision/budget lineage/provenance/distinction/couplings/validation/effect control/bindings/context/lifecycle | Types only, pending |
| DistinctionRevision, CouplingRevision | D II §§1.4–1.5: source-backed distinction; separate causal revision, phase/carrier/scope/channel/parameters/bounds/evidence/trials/conflicts/meta | Types only, pending |
| Plastic evidence link, inadequacy witness | D §0.4, II §§2.3,3; O §6: source-backed, candidate-free witness; no qualification from candidate's own effect | Types only, pending |
| Shadow trial, frozen prediction | D §0.4, II §§3.3–3.7; O §6: preregistered prediction, data split/version, actual recorded outcomes | Types only, pending |
| Effect budget ledger | D §0.4, II §4.4; F §§7.1,8: total-root allocation/consumption and branch-specific single-root path; no refund/recharge by branch switch | Types only, pending |
| Qualification branch / single-root ledger | F §§7,7.1,8; user C01 instruction: multi/single discriminator, dwell start, content/support versions, revalidation/impact/cumulative path | Types only; never enables growth |
| EffectiveCausalProjection | D §0.4, II §1.2; O §3.2: minimal authorized causal view; no candidate or audit text | Types only, pending |
| Consolidation binding, theta source binding | D §0.4, II §5: exact source revisions and responsibility suppression | Types only, pending |
| Mechanism context, descendant-review marker | D §0.4, II §§4.8,5.6; O §§7–8: generation ancestors and explicit review dependency | Types only, pending |
| Action plan/request/result/lifecycle | S §§11,20; O §16.3: eight action states, identity/bindings, credentials, actual outcomes and UNKNOWN | Pending IDL |
| Terminal observations/results/maintenance | CONTRACTS §4.2: all nine rows including user_retraction; stable source key | Pending IDL |
| Receipt/output/action/approval/run state/readout/query result | CONTRACTS §4.3; all fields and distinct query statuses; BodyAction/BodyReadout from §§2,7 | Three review candidate messages only |
| Approval decision, output cancel, query | CONTRACTS §4.3; QueryKind/Filter/Item/Status in §4.1 | Pending IDL |
| TaskInput, PlanAndCarrierReceipt, SpeechMode | CONTRACTS §5; O §13.2: explicit task/protocol/facts/snapshot/budgets; A/B/C receipts and strictness | Pending IDL |
| LineageHeader | ADR 0005: REHEARSAL/LIFE, creation time, genesis digest | Review candidate only |
| Finite binary64, bounded nanoseconds/counts | S §6; O §19: reject nonfinite and overflow, normalize -0, explicit units/caps | Rust helper implemented; no wire schema |

Every future field must have a source mapping. A named object is not permission
to invent its undefined semantics. Unspecified semantic extensions require an
ADR before implementation. Byte-layout choices in ADR 0007 are proposed
engineering decisions, not quotations from the original specifications.

No golden message vector or cross-language result has yet been certified.
All remaining P5-01 acceptance items remain open. SYS01–SYS25 are preserved.
