# P5-01 implemented message mapping

This supplements the historical SOURCE-MAP.md checklist. F/D/O/S retain the baseline document identities. Each row maps the emitted message to its source annotation and frozen tag. Field-level layout and constraints are in the referenced proto and generated schema.json.

| Message | Specification | IDL | Frozen type tag | Fields |
| --- | --- | --- | --- | --- |
| ObjectRef | S 5,6,7; D I 7.1 | core/contracts/base.proto | `echo.object-ref` | schema_version, digest, object_type, resource_version, byte_length |
| ConceptRef | F 3.4; D I 4.4 | core/contracts/base.proto | `echo.concept-ref` | schema_version, concept_id, registry_version |
| VersionedEnvironment | O 2.1,16.1; S 5 | core/contracts/base.proto | `echo.environment` | schema_version, registry, irl, scope_interpreter, carrier, model, tokenizer, config, implementations |
| FailureReason | D I 6.2; S 21 | core/contracts/base.proto | `echo.failure-reason` | schema_version, code, detail_ref |
| DegradedStatus | D I 4.5,5.4; S 21; E13 | core/contracts/base.proto | `echo.degraded-status` | schema_version, states, reasons |
| LineageHeader | ADR 0005 | core/contracts/base.proto | `echo.lineage` | schema_version, lineage, created_at_ns, genesis_manifest_digest |
| AuditExtension | S 6; D I 7.1 | core/contracts/base.proto | `echo.audit-extension` | schema_version, audit_schema_tag, audit_schema_version, payload |
| EffectContract | O 3.1; S 5 | core/contracts/growth.proto | `echo.effect-contract` | schema_version, responsibility_id, causal_revision, owner, endpoint, scope, carrier, parameters, amplitude_bound, qualifications, ancestors, dependencies |
| QualificationCertificate | S 5; O 6,8,9 | core/contracts/growth.proto | `echo.qualification` | schema_version, test_domain, root_blocks, ancestry, exposure, assumptions, opportunity_id, criteria, raw_results, dependencies, outcome |
| ConsolidationSlice | D II 5.2; O 9 | core/contracts/growth.proto | `echo.consolidation-slice` | schema_version, source_trace_revision, source_coupling_revision, responsibility, scope, functional_contract, current_q, candidate_envelope, supporting_roots, qualifications, ancestry, dependencies, budget_claim, promote_interval_ns, validity_contract |
| ThetaModule | D II 5.2; O 9,10 | core/contracts/growth.proto | `echo.theta-module` | schema_version, module_id, revision, source_slice_id, carrier, parameters, composition_order, enabled, scope, mechanism_effect, validity_contract |
| ResponsibilityBinding | D II 5.3; O 3,10.1 | core/contracts/growth.proto | `echo.responsibility-binding` | schema_version, responsibility_id, causal_revision, owner, endpoint, scope, source_slice_id, module_id, disposition |
| TrialResult | O 6,8; S 15 | core/contracts/growth.proto | `echo.trial-result` | schema_version, trial_id, settlement_id, plan, candidate_revision, baseline, outcome, actual_results, root_blocks, dependencies |
| DistinctionRevision | D II 1.3,1.4 | core/contracts/growth.proto | `echo.distinction` | schema_version, distinction_id, revision, form, status, anchor_roots, scope, representation |
| CouplingRevision | D II 1.5; O 7 | core/contracts/growth.proto | `echo.coupling` | schema_version, coupling_id, revision, phase, carrier, condition_scope, target, parameters, uncertainty, qualification_scope, effect_bounds, evidence_refs, trial_refs, conflict_set, metaplastic_flag |
| PlasticEvidenceLink | D II 1.8,2.2 | core/contracts/growth.proto | `echo.plastic-evidence` | schema_version, event_id, root_id, distinction_id, coupling_revision, relation |
| MechanismContext | D 0.4; D II 1.8; O 7.1 | core/contracts/growth.proto | `echo.mechanism-context` | schema_version, theta_modules, metaplastic_couplings, previous_commit |
| DescendantReviewMarker | D 0.4; D II 5.6; O 8 | core/contracts/growth.proto | `echo.descendant-review` | schema_version, ancestor, affected_responsibilities, dependency_snapshot, needs_review |
| PlasticTraceRevision | D 0.4; D II 1.3 | core/contracts/growth.proto | `echo.plastic-trace` | schema_version, trace_id, revision, budget_lineage_id, source_ids, root_ids, time_range, evidence_links, distinction, coupling_hypotheses, validation_state, effect_control, consolidation_bindings, mechanism_context, lifecycle |
| TraceHead | D 0.4; D II 1.8 | core/contracts/growth.proto | `echo.trace-head` | schema_version, trace_id, revision |
| RepresentationInadequacyWitness | D II 2.3; O 6.1 | core/contracts/growth.proto | `echo.inadequacy-witness` | schema_version, equivalence_class, controlled_variables, scope, observed_difference, alternative_checks, statistical_evidence, non_identifiable, reference_contract |
| FrozenPrediction | D II 3.4; O 6,8 | core/contracts/growth.proto | `echo.frozen-prediction` | schema_version, candidate_revision, baseline, scope, allowed_information, forbidden_information, target, time_window, scorer, criteria, validator_version |
| ShadowTrial | D 0.4; D II 1.8,3.4 | core/contracts/growth.proto | `echo.shadow-trial` | schema_version, trial_id, frozen_prediction, actual_results, settlement |
| EffectBudgetLedger | D II 4.4; O 11 | core/contracts/growth.proto | `echo.effect-budget-ledger` | schema_version, root_id, budget_lineage_id, endpoint, window, total_allocation, r_allocation, theta_allocation, consumed, remaining |
| SingleRootLedger | F 7.1,8; user C01 | core/contracts/growth.proto | `echo.single-root-ledger` | schema_version, root_id, branch, content_version, support_version, dwell_start_ns, revalidation_records, impact_metrics, cumulative_single_root_path, total_root_ledger |
| EffectiveCausalProjection | D II 1.2; O 3.2 | core/contracts/growth.proto | `echo.effective-causal-projection` | schema_version, snapshot_id, projected_at_ns, effects |
| ConsolidationBinding | D 0.4; D II 5.3 | core/contracts/growth.proto | `echo.consolidation-binding` | schema_version, source_slice_id, theta_module_id, responsibility |
| ThetaModuleSourceBinding | D 0.4; D II 5.2 | core/contracts/growth.proto | `echo.theta-source-binding` | schema_version, module_id, source_slice_id, source_trace_revision, source_coupling_revision |
| EventEnvelope | F 4; S 9 | core/contracts/state.proto | `echo.event` | schema_version, id, kind, event_time_ns, ingest_time_ns, raw_log_position, parents, payload_ref, provenance, device, session, stable_source_key, payload_digest, action_id, trial_id, correction_of, derivation |
| FactItem | F 3; S 11 | core/contracts/state.proto | `echo.fact-item` | schema_version, content, source_id, availability, correction_chain |
| FactProjection | F 3; S 11 | core/contracts/state.proto | `echo.fact-projection` | schema_version, history_prefix, query_version, status, items, index_version, truncated |
| SourceMetadata | F 3.2; S 10 | core/contracts/state.proto | `echo.source-metadata` | schema_version, source_refs, source_span, derivation, epistemic_type, resolution_state, representation_version |
| Ref | F 3.2 | core/contracts/state.proto | `echo.semantic-ref` | schema_version, id, metadata, referent |
| RoleBinding | F 3.2,3.4 | core/contracts/state.proto | `echo.role-binding` | schema_version, role, node_ref |
| Claim | F 3.2,3.4 | core/contracts/state.proto | `echo.semantic-claim` | schema_version, id, metadata, predicate, roles, value, polarity, modality, time |
| Act | F 3.2 | core/contracts/state.proto | `echo.semantic-act` | schema_version, id, metadata, interaction_kind |
| Edge | F 3.2 | core/contracts/state.proto | `echo.semantic-edge` | schema_version, relation, from_node, to_node |
| SemanticGraph | F 3.2; D I 4.3 | core/contracts/state.proto | `echo.semantic-graph` | schema_version, id, refs, claims, acts, edges |
| ProposalBinding | D I 3; O 16.1 | core/contracts/state.proto | `echo.proposal-binding` | schema_version, branch_id, event_id, raw_log_position, base_commit, process_time_ns, anchor_time_ns, projected_state_digest, accepted_z_digest, c_core_digest, x_revision, r_revision, theta_revision, validation_snapshot, source_validity_version, environment, relevant_effect_set_digest, proposal_payload_digest, read_set |
| SemanticCandidate | D I 4.2,7.1 | core/contracts/state.proto | `echo.semantic-candidate` | schema_version, binding, candidate_graph, producer, producer_version, actual_output |
| ConceptProposal | F 3.4; D I 4.4 | core/contracts/state.proto | `echo.concept-proposal` | schema_version, binding, mention, signature, grounding, relations, source_refs |
| FastStateSnapshot | D I 5.1; O 5.1 | core/contracts/state.proto | `echo.fast-state` | schema_version, revision, representation_version, dimension, coordinates, anchor_time_ns |
| FastDriveProposal | D I 5.3; O 5.2 | core/contracts/state.proto | `echo.fast-drive` | schema_version, binding, semantic_graph_id, state_space_version, drive, source_node_refs, producer, producer_version, proposal_hash |
| CoreTransaction | D I 3,7; O 16.2; S 8 | core/contracts/state.proto | `echo.transaction` | schema_version, binding, staged_h, staged_x, staged_r, staged_theta, staged_l, staged_q, staged_u, processed_position, anchor_time_ns, staged_concepts, accepted_results, audit_refs |
| CoreCommitResult | D I 6.2; O 16.2 | core/contracts/state.proto | `echo.commit-result` | schema_version, branch_id, event_id, commit_id, committed, snapshot, status |
| UserUtterance | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.user-utterance` | schema_version, text, lang, asr_confidence, duration_ms, audio_ref |
| UserText | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.user-text` | schema_version, text |
| UserRetraction | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.user-retraction` | schema_version, target_refs, note |
| TouchObservation | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.touch` | schema_version, region, kind, speed |
| DesktopObservation | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.desktop-observation` | schema_version, kind, process_name, title, dnd, idle_ms |
| CodexSessionObservation | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.codex-session` | schema_version, event, session_id, cwd_label, summary |
| SpeechDelivery | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.speech-delivery` | schema_version, action_id, output_id, delivered_chunks, played_chars, status |
| BodyActionResult | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.body-action-result` | schema_version, action_id, status, detail |
| TerminalMaintenance | CONTRACTS 4.2 | core/contracts/terminal.proto | `echo.terminal-maintenance` | schema_version, kind, detail |
| Receipt | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.receipt` | schema_version, stable_source_key, stage, event_id, reason |
| OutputChunk | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.output-chunk` | schema_version, output_id, action_id, chunk_index, text, lang, snapshot_id, speech_mode, strict |
| OutputEnd | CONTRACTS 4.3; ADR 0007 hand example | core/contracts/terminal.proto | `echo.output-end` | schema_version, output_id, reason |
| Anchor | CONTRACTS 2.1 | core/contracts/terminal.proto | `echo.anchor` | schema_version, kind, x, y, hwnd, offset_x, side, along |
| GazeTarget | CONTRACTS 2.1 | core/contracts/terminal.proto | `echo.gaze-target` | schema_version, kind, x, y |
| BodyAction | CONTRACTS 2.1,4.3 | core/contracts/terminal.proto | `echo.body-action` | schema_version, kind, motion_id, anchor, platform_id, side, target, hold_ms |
| ActionRequest | CONTRACTS 4.3; O 16.3 | core/contracts/terminal.proto | `echo.action-request` | schema_version, action_id, credential, action, expires_at_ns |
| ApprovalRequest | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.approval-request` | schema_version, approval_id, action_id, tier, title, detail, expires_at_ns |
| RunState | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.run-state` | schema_version, states, speech_mode, strict, lineage, born_at, autonomy, reasons |
| ExpressionWeight | CONTRACTS 7 | core/contracts/terminal.proto | `echo.expression-weight` | schema_version, emotion, weight |
| BodyReadout | CONTRACTS 7 | core/contracts/terminal.proto | `echo.body-readout` | schema_version, version, expression_baseline, idle_tempo, ear_pose, tail_amplitude, gaze_sociability, posture |
| Readout | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.readout` | schema_version, readout, snapshot_id, projected_at_ns |
| QueryFilter | CONTRACTS 4.1 | core/contracts/terminal.proto | `echo.query-filter` | schema_version, text, from, to, kinds, include_retracted |
| QueryItem | CONTRACTS 4.1 | core/contracts/terminal.proto | `echo.query-item` | schema_version, ref, at, kind, summary, source, retracted |
| Query | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.query` | schema_version, query_id, kind, filter, limit |
| QueryResult | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.query-result` | schema_version, query_id, status, items |
| ApprovalDecision | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.approval-decision` | schema_version, approval_id, decision |
| OutputCancel | CONTRACTS 4.3 | core/contracts/terminal.proto | `echo.output-cancel` | schema_version, output_id |
| HookCoefficients | O 13.1,13.2 | core/contracts/terminal.proto | `echo.hook-coefficients` | schema_version, hook, coefficients, shape, dtype, conversion_tolerance |
| ModulationEnvelope | O 13.2; S 19 | core/contracts/terminal.proto | `echo.modulation` | schema_version, snapshot_id, projected_at_ns, x_digest, r_digest, theta_digest, carrier, model, tokenizer, hooks, total_bound, effect_receipt_digest |
| TaskInput | CONTRACTS 5.1 | core/contracts/terminal.proto | `echo.task-input` | schema_version, current_input, protocol_version, protocol_content, c_task, snapshot_id, max_tokens, max_duration_ns |
| ActionPlan | S 11,20 | core/contracts/terminal.proto | `echo.action-plan` | schema_version, plan_id, snapshot_id, task_version, protocol_version, output_or_tool_parameters, expected_scope, dependencies, deadline_ns |
| ActionLifecycle | S 20; O 16.3 | core/contracts/terminal.proto | `echo.action-lifecycle` | schema_version, action_id, status, plan, request, actual_result |
| PlanAndCarrierReceipt | CONTRACTS 5.1 | core/contracts/terminal.proto | `echo.plan-carrier-receipt` | schema_version, output_chunks, action_plan, speech_mode, strict, model_hash, tokenizer_hash, carrier_receipt, projection_version, intermediate_results, usage |
