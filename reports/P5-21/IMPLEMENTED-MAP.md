# P5-21 implemented message map

Generated from `core/contracts/generated/schema.json`. New messages first, then the messages this card marks superseded. Field-level layout and constraints are in the referenced `.proto` and the generated schema.

## New messages (42)

| Message | Specification | IDL | Frozen type tag | Supersedes | Fields |
| --- | --- | --- | --- | --- | --- |
| NatureVector | v2.0 §1 a(e) ∈ [−1,1]⁴, §3, §6 x_i, §8 b ∈ [−1,1]⁴ | core/contracts/base.proto | `echo.nature-vector` | — | schema_version, connection, empathy, curiosity, achievement |
| NatureScale | v2.0 §8 g, λ, σ̂, τ̂ per nature channel | core/contracts/base.proto | `echo.nature-scale` | — | schema_version, connection, empathy, curiosity, achievement |
| NatureReal | v2.0 §9.1 β_i·x_i coefficients | core/contracts/base.proto | `echo.nature-real` | — | schema_version, connection, empathy, curiosity, achievement |
| TrilingualText | v2.0 §7 concept in three languages; E11 | core/contracts/base.proto | `echo.trilingual-text` | — | schema_version, zh, en, ja |
| PersonRelation | v2.0 §7, §10 relations are tendencies about a person; V2-13 | core/contracts/base.proto | `echo.person-relation` | — | schema_version, person_id, kind |
| VersionedEnvironmentV2 | v2.0 §1 Ω, §2.4, §9; O 2.1,16.1; S 5 | core/contracts/base.proto | `echo.environment-v2` | VersionedEnvironment; reserved 4 | schema_version, registry, irl, carrier, model, tokenizer, config, implementations, nature_appraiser, situation_matcher, direction_set, law_version |
| LineageHeaderV2 | v2.0 §13, §14; ADR 0005; ADR 0010 decision 2 | core/contracts/base.proto | `echo.lineage-v2` | LineageHeader | schema_version, lineage, created_at_ns, genesis_manifest_digest, world_id, world_seed |
| EvidenceAllocation | v2.0 §5.2 S_{c,k}, weight factors, shares; LP2, I25 | core/contracts/law.proto | `echo.evidence-allocation` | — | schema_version, tendency_id, support, intensity, share |
| EvidenceGroup | v2.0 §1 c, §4 EV1, EV2, EV7, EV9; §5.2 W = intensity × credibility × verifiability × share | core/contracts/law.proto | `echo.evidence-group` | — | schema_version, group_id, members, source_key, credibility, verifiability, allocations |
| Tendency | v2.0 §5.2, §5.3 corollary 7, §7, §10 | core/contracts/law.proto | `echo.tendency` | — | schema_version, tendency_id, concept, predicate, direction, evidence_a, evidence_b, h_hat, group_count, flip_count, last_sign, pi, typical_magnitude, relation, attachment_prior |
| RSnapshot | v2.0 §2.1, §2.2, §7 | core/contracts/law.proto | `echo.r-snapshot` | — | schema_version, revision, law_version, tendencies |
| ThetaSnapshot | v2.0 §8 θ and the statistics of C | core/contracts/law.proto | `echo.theta-snapshot` | — | schema_version, revision, law_version, values, sigma, tau_s, weight_sum, h_hat_theta, estimator_state |
| GrowthCommand | v2.0 §5.4, §10; ADR 0010 decision 5; permissions P5-05 | core/contracts/law.proto | `echo.growth-command` | — | schema_version, command_id, kind, verdict, review_record |
| LawParameters | v2.0 §5.2, §6, §8, §16 | core/contracts/law.proto | `echo.law-parameters` | — | schema_version, n0, kappa1, w_max, w_teach, v_u, r_min, r_max, h0, n_h, h_hat_min, h0_theta, h_hat_theta_min, n_b, n_sigma, g_min, g_max, recovery_min_per_s, recovery_max_per_s, w_min, t_x_s, epsilon_x, k_max |
| LawVersion | v2.0 §1 𝔏_v, §5.4, §16 | core/contracts/law.proto | `echo.law-version` | — | schema_version, version, reference_form, parameters |
| FuseConfig | v2.0 §11, §12, §16 | core/contracts/law.proto | `echo.fuse-config` | — | schema_version, r_capacity, delta_theta_max, jump_threshold, jump_probes, slow_drift_run, slow_drift_margin, contact_limit, message_limit, network_window_ns |
| GenesisManifestV2 | v2.0 §2.3, §13, §16; I28; ADR 0010 decision 6 | core/contracts/law.proto | `echo.genesis-v2` | — | schema_version, law_version, nature_structure, initial_gain, initial_baseline, sigma0, tau0_s, attachment_prior, caretaker_id, carrier, direction_set, direction_validation, body_readout, proactivity_readout, fuses, boundaries, protocol |
| ThetaValues | v2.0 §8 θ = (b, g, λ) | core/contracts/ledger.proto | `echo.theta-values` | — | schema_version, baseline, gain, recovery_per_s |
| TendencyChange | v2.0 §2.5 R change: tendency, π before and after, evidence groups | core/contracts/ledger.proto | `echo.tendency-change` | — | schema_version, tendency_id, pi_before, pi_after, evidence_groups |
| ThetaChange | v2.0 §2.5 θ change: before and after values, referenced events | core/contracts/ledger.proto | `echo.theta-change` | — | schema_version, before, after, events |
| RecomputeRecord | v2.0 §2.5, §5.4 recompute: law version and difference summary | core/contracts/ledger.proto | `echo.recompute-record` | — | schema_version, cause, law_version, previous_law_version, r_revision_before, r_revision_after, theta_revision_before, theta_revision_after, changed_tendencies, theta_changed |
| MigrationRecord | v2.0 §2.5, §9.5 model change is a migration, not growth | core/contracts/ledger.proto | `echo.migration-record` | — | schema_version, previous_environment, environment, validation |
| LedgerEntry | v2.0 §1 L, §2.5, §5.4, §12; V2-10, V2-11 | core/contracts/ledger.proto | `echo.ledger-entry` | — | schema_version, entry_id, event_id, kind, tendency_change, theta_change, recompute, migration, review_trigger, review_verdict, review_record |
| NatureAppraisalRecord | v2.0 §1 a(e), §3, EV6, EV10 | core/contracts/records.proto | `echo.nature-appraisal` | — | schema_version, record_id, event_id, appraisal, persons, binding, producer, producer_version |
| TendencyMatch | v2.0 §1 μ_k(e) ∈ [0,1] | core/contracts/records.proto | `echo.tendency-match` | — | schema_version, tendency_id, mu |
| SituationMatchRecord | v2.0 §1 μ, §2.4 M, EV5, EV6 | core/contracts/records.proto | `echo.situation-match` | — | schema_version, record_id, event_id, matches, binding, producer, producer_version |
| TendencyConcept | v2.0 §7 concept: name, definition, examples, three languages | core/contracts/records.proto | `echo.tendency-concept` | — | schema_version, name, definition, positive_examples, negative_examples |
| TendencyCandidateRecord | v2.0 §7 candidates, EV3, EV5; §5.2 κ, d_k; V2-13 | core/contracts/records.proto | `echo.tendency-candidate` | — | schema_version, record_id, event_id, concept, predicate, direction, relation, binding, producer, producer_version |
| TeachingAnnotation | v2.0 EV7, §10 | core/contracts/records.proto | `echo.teaching-annotation` | — | schema_version, record_id, event_id, speaker, tendency_id, supports, binding, producer, producer_version |
| LieAnnotation | v2.0 EV4, §10; I27 | core/contracts/records.proto | `echo.lie-annotation` | — | schema_version, record_id, event_id, statement, contradicts, binding, producer, producer_version |
| SameMatterAnnotation | v2.0 EV2 | core/contracts/records.proto | `echo.same-matter-annotation` | — | schema_version, record_id, event_id, same_as, binding, producer, producer_version |
| ReinterpretationLink | v2.0 EV5, EV6, §5.4 | core/contracts/records.proto | `echo.reinterpretation` | — | schema_version, record_id, event_id, new_record, prior_records, interpreter_version, binding, producer, producer_version |
| EventAttributesRecord | v2.0 §1 exogenous, EV2, EV3, EV9; P5-06 rule table | core/contracts/records.proto | `echo.event-attributes` | — | schema_version, event_id, rule_table_version, exogenous, self_narrative, verifiable, source_key |
| ProposalBindingV2 | D I 3; O 16.1; v2.0 §1 Ω, §2.4 | core/contracts/state.proto | `echo.proposal-binding-v2` | ProposalBinding; reserved 16 | schema_version, branch_id, event_id, raw_log_position, base_commit, process_time_ns, anchor_time_ns, projected_state_digest, accepted_z_digest, c_core_digest, x_revision, r_revision, theta_revision, validation_snapshot, source_validity_version, relevant_effect_set_digest, proposal_payload_digest, read_set, environment |
| SemanticCandidateV2 | D I 4.2,7.1; v2.0 §2.4 | core/contracts/state.proto | `echo.semantic-candidate-v2` | SemanticCandidate; reserved 2 | schema_version, candidate_graph, producer, producer_version, actual_output, binding |
| ConceptProposalV2 | F 3.4; D I 4.4; v2.0 §7 | core/contracts/state.proto | `echo.concept-proposal-v2` | ConceptProposal; reserved 2 | schema_version, mention, signature, grounding, relations, source_refs, binding |
| LearnedComponent | v2.0 §6 x_k for k ∈ R⁺ | core/contracts/state.proto | `echo.learned-component` | — | schema_version, tendency_id, value |
| FastStateSnapshotV2 | v2.0 §6 X = [−1,1]^{4+|R⁺|}; D I 5.1 | core/contracts/state.proto | `echo.fast-state-v2` | FastStateSnapshot; reserved 4, 5 | schema_version, revision, representation_version, anchor_time_ns, nature, learned |
| CoreTransactionV2 | D I 3,7; O 16.2; S 8; v2.0 §2.4, §2.5 | core/contracts/state.proto | `echo.transaction-v2` | CoreTransaction; reserved 2 | schema_version, staged_h, staged_x, staged_r, staged_theta, staged_l, staged_q, staged_u, processed_position, anchor_time_ns, staged_concepts, accepted_results, audit_refs, binding, staged_ledger |
| CoreCommitResultV2 | D I 6.2; O 16.2; v2.0 §2.4, §2.5 | core/contracts/state.proto | `echo.commit-result-v2` | CoreCommitResult | schema_version, branch_id, event_id, commit_id, committed, snapshot, status, ledger_entries |
| LearnedCoefficient | v2.0 §9.1 β_k·x_k for k ∈ R⁺, §9.2 | core/contracts/terminal.proto | `echo.learned-coefficient` | — | schema_version, tendency_id, coefficient |
| ModulationEnvelopeV2 | v2.0 §9.1, §9.2; O 13.2; S 19; ADR 0008 §3 | core/contracts/terminal.proto | `echo.modulation-v2` | ModulationEnvelope; reserved 10 | schema_version, snapshot_id, projected_at_ns, x_digest, r_digest, theta_digest, carrier, model, tokenizer, total_bound, effect_receipt_digest, direction_set, nature_coefficients, learned_coefficients |

## Superseded messages (32)

| Message | Frozen type tag | Successor | Reason |
| --- | --- | --- | --- |
| VersionedEnvironment | `echo.environment` | VersionedEnvironmentV2 | P5-21; v2.0 §15: scope_interpreter is replaced; v2 adds A, M, directions and law version |
| LineageHeader | `echo.lineage` | LineageHeaderV2 | P5-21; ADR 0010 decision 2: EXPERIMENT needs world identity and seed |
| EffectContract | `echo.effect-contract` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| QualificationCertificate | `echo.qualification` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ConsolidationSlice | `echo.consolidation-slice` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ThetaModule | `echo.theta-module` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ResponsibilityBinding | `echo.responsibility-binding` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| TrialResult | `echo.trial-result` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| DistinctionRevision | `echo.distinction` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| CouplingRevision | `echo.coupling` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| PlasticEvidenceLink | `echo.plastic-evidence` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| MechanismContext | `echo.mechanism-context` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| DescendantReviewMarker | `echo.descendant-review` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| PlasticTraceRevision | `echo.plastic-trace` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| TraceHead | `echo.trace-head` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| RepresentationInadequacyWitness | `echo.inadequacy-witness` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| FrozenPrediction | `echo.frozen-prediction` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ShadowTrial | `echo.shadow-trial` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| EffectBudgetLedger | `echo.effect-budget-ledger` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| SingleRootLedger | `echo.single-root-ledger` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| EffectiveCausalProjection | `echo.effective-causal-projection` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ConsolidationBinding | `echo.consolidation-binding` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ThetaModuleSourceBinding | `echo.theta-source-binding` | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data and for tests |
| ProposalBinding | `echo.proposal-binding` | ProposalBindingV2 | P5-21; binds VersionedEnvironmentV2 (v2.0 §1 Ω) |
| SemanticCandidate | `echo.semantic-candidate` | SemanticCandidateV2 | P5-21; carries ProposalBindingV2 |
| ConceptProposal | `echo.concept-proposal` | ConceptProposalV2 | P5-21; carries ProposalBindingV2 |
| FastStateSnapshot | `echo.fast-state` | FastStateSnapshotV2 | P5-21; v2.0 §6: nature channels plus learned components by tendency ID |
| FastDriveProposal | `echo.fast-drive` | NatureAppraisalRecord | P5-21; v2.0 §2.4, §3, §6: drive is assembled by F from the recorded a(e) |
| CoreTransaction | `echo.transaction` | CoreTransactionV2 | P5-21; adds staged L entries for R and θ changes (v2.0 §2.4) |
| CoreCommitResult | `echo.commit-result` | CoreCommitResultV2 | P5-21; reports published L entries (v2.0 §2.4) |
| HookCoefficients | `echo.hook-coefficients` | — | P5-21; v2.0 §9.1: per-direction coefficients are carried by ModulationEnvelopeV2 |
| ModulationEnvelope | `echo.modulation` | ModulationEnvelopeV2 | P5-21; v2.0 §9.1: direction set, nature and learned coefficients |

## New enums (7) and added values

| Enum | Values |
| --- | --- |
| Lineage | added EXPERIMENT=3 |
| RunStatus | added GROWTH_PAUSED=10, GROWTH_REVIEW_PENDING=11, A_DEGRADED=12, M_DEGRADED=13 |
| FailureCode | added GROWTH_OFF_REHEARSAL=14 |
| RelationKind (new) | CLOSENESS=1, TRUST=2 |
| EvidenceSign (new) | SIGN_NEGATIVE=1, SIGN_POSITIVE=2 |
| GrowthCommandKind (new) | GROWTH_PAUSE=1, GROWTH_RESUME=2, GROWTH_REVIEW_CONCLUSION=3 |
| LedgerKind (new) | LEDGER_TENDENCY_CHANGED=1, LEDGER_THETA_CHANGED=2, LEDGER_RECOMPUTED=3, LEDGER_MIGRATED=4, LEDGER_GROWTH_PAUSED=5, LEDGER_GROWTH_RESUMED=6, LEDGER_REVIEW_HELD=7, LEDGER_REVIEW_CONCLUDED=8 |
| RecomputeCause (new) | CAUSE_LAW_VERSION=1, CAUSE_SOURCE_CORRECTION=2, CAUSE_REINTERPRETATION=3 |
| ReviewTrigger (new) | TRIGGER_JUMP=1, TRIGGER_THETA_STEP=2, TRIGGER_SLOW_DRIFT=3 |
| ReviewVerdict (new) | LAW_CORRECT=1, LAW_INCORRECT=2 |

## Superseded enums and values

| Enum | Value | Successor | Reason |
| --- | --- | --- | --- |
| QualificationBranch | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| EffectEndpoint | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| EffectOwner | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| CouplingPhase | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| TrialOutcome | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| DistinctionForm | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| DistinctionStatus | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| TraceLifecycle | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| EvidenceRelation | (whole enum) | — | P5-21; v2.0 §15: old R/G mechanism, kept only to read old data |
| RunStatus | CORE_QUARANTINED | — | P5-21; v2.0 §12: no whole-core quarantine, evidence is removed and recomputed |
| RunStatus | AX_DEGRADED | A_DEGRADED | P5-21; V2-17: A_x is now nature appraisal A |
| FailureCode | GROWTH_NOT_ENABLED_V1 | GROWTH_OFF_REHEARSAL | P5-21; ADR 0010 decision 1: v1.0 grows from birth |
| FailureCode | AX_UNAVAILABLE | — | P5-21; V2-17: A failures use A_DEGRADED with a generic reason |
| FailureCode | EVIDENCE_INCOMPLETE | — | P5-21; v2.0 §15: C consolidation query plans are replaced |
| FailureCode | UNSUPPORTED_SLICE | — | P5-21; v2.0 §15: consolidation slices are replaced |
| FailureCode | UNKNOWN_OVERLAP | — | P5-21; v2.0 §15: scope DSL overlap checks are replaced |
