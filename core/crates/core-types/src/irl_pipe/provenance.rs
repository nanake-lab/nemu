//! ADR 0051. Only a correlated candidate returned by an active core call can
//! be compared with the independent transaction and producer observations.
use super::{Direction, PipeCall};
use crate::contract::{Limits, Result, require};
use crate::generated::*;
use crate::scalar::DecimalU64;
use crate::semantic::{
    SemanticReadView, StageExecution, check_graph_registry, check_graph_sources,
    check_stage_execution, resolve_location, resolved_text_range,
};
use crate::{WireMessage, codec_schema};

/// Application payloads from the trusted bridge record exclude transport headers
/// and keys. The local adapter supplies only its independently observed record.
pub struct IrlProducerObservation<'a> {
    pub record: &'a IrlProducer,
    pub remote_exchange: Option<(&'a [u8], &'a [u8])>,
}

pub struct ReceivedCandidate<'a> {
    returned: IrlCandidateReturn,
    binding: &'a ProposalBindingV2,
    event: RecordVersion,
}

impl<'a> PipeCall<'a> {
    /// `observed` is the host's independent record of the authenticated
    /// delegate execution, including the actual output, parameters and disclosure.
    pub fn receive_candidate(
        &mut self,
        bytes: &[u8],
        observed: &IrlProducerObservation<'_>,
        limits: Limits,
    ) -> Result<ReceivedCandidate<'a>> {
        require(bytes.len() <= self.frame_bytes, "IRL return budget")?;
        let binding = self.binding.ok_or("candidate outside core call")?;
        let event = self
            .opening
            .dispatch
            .as_ref()
            .ok_or("missing dispatch")?
            .target
            .clone();
        let returned = IrlCandidateReturn::from_protobuf(bytes, limits)?;
        let p = &returned.producer;
        let d = &p.delegation;
        let f = &returned.frame;
        require(
            p == observed.record
                && p.link == d.link
                && d.parent == self.opening.call
                && f.operation == IrlPipeOperation::IRL_CANDIDATE
                && f.output.as_ref().is_some_and(|m| m.object == p.output)
                && d.inputs
                    .iter()
                    .all(|m| binding.read_set.contains(&m.object)),
            "IRL candidate differs from producer execution",
        )?;
        for input in &d.inputs {
            super::verify_material(input, limits)?;
        }
        match p.link {
            IrlPipeLink::IRL_WORKER_ADAPTER => require(
                observed.remote_exchange.is_none()
                    && p.executed_model == d.local_model
                    && p.isolation
                        .as_ref()
                        .is_none_or(|r| Some(r) == d.isolation.as_ref()),
                "IRL adapter execution mismatch",
            )?,
            IrlPipeLink::IRL_WORKER_BRIDGE => {
                let r = p.remote.as_ref().ok_or("missing remote attempt")?;
                let (sent, received) = observed.remote_exchange.ok_or("missing remote payloads")?;
                require(
                    matches!(
                        r.status,
                        RemoteAttemptStatus::REMOTE_ATTEMPT_COMPLETED
                            | RemoteAttemptStatus::REMOTE_ATTEMPT_TRUNCATED
                    ),
                    "remote outcome has no candidate",
                )?;
                super::verify_raw(
                    r.sent_request.as_ref().ok_or("missing sent request")?,
                    sent,
                    limits,
                )?;
                super::verify_raw(
                    &r.response.as_ref().ok_or("missing service response")?.raw,
                    received,
                    limits,
                )?;
            }
            _ => return Err("not a delegated producer".into()),
        }
        self.receive(
            Direction::ToCaller,
            &f.protobuf_bytes(limits)?,
            limits,
            |_| Ok(&[]),
        )?;
        Ok(ReceivedCandidate {
            returned,
            binding,
            event,
        })
    }
}

fn check_object<T: WireMessage>(reference: &ObjectRef, value: &T, limits: Limits) -> Result<()> {
    let json = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let schema = codec_schema(T::NAME)?;
    crate::semantic::bound_object(schema, T::NAME, &json, reference, limits)
}

/// Resolve the exact excerpt taken from the saved model output against the
/// original field. A repeated excerpt requires an independently known range.
/// Model offsets and source aliases are never used as location authorities.
pub fn locate_excerpt(
    view: &impl SemanticReadView,
    base: &str,
    transaction: Option<&str>,
    field: &SourceLocation,
    excerpt: &str,
    trusted_range: Option<&Utf8Range>,
    limits: Limits,
) -> Result<SourceLocation> {
    require(!excerpt.is_empty(), "empty IRL source excerpt")?;
    let mut location = field.clone();
    location.text = None;
    let value = resolve_location(view, base, transaction, &location, limits)?;
    let text = value.as_str().ok_or("IRL source requires text")?;
    let range = if let Some(r) = trusted_range {
        require(
            resolved_text_range(text, r)? == excerpt,
            "different IRL excerpt",
        )?;
        r.clone()
    } else {
        let (field_text, offset) = match &field.text {
            Some(r) => (resolved_text_range(text, r)?, r.start.0),
            None => (text, 0),
        };
        let start = field_text.find(excerpt).ok_or("unmatched IRL excerpt")?;
        // rfind also detects overlapping repetitions.
        require(
            field_text.rfind(excerpt) == Some(start),
            "ambiguous IRL excerpt",
        )?;
        Utf8Range {
            schema_version: 1,
            start: DecimalU64(offset + start as u64),
            end: DecimalU64(offset + (start + excerpt.len()) as u64),
        }
    };
    require(
        field
            .text
            .as_ref()
            .is_none_or(|r| r.start.0 <= range.start.0 && range.end.0 <= r.end.0),
        "excerpt outside bound source",
    )?;
    location.text = Some(range);
    Ok(location)
}

/// A copied phrase parsed from the retained output; the field and optional
/// disambiguation come from the core's source mapping for this call.
pub struct IrlExcerpt<'a> {
    pub field: &'a SourceLocation,
    pub quote: &'a str,
    pub known_range: Option<&'a Utf8Range>,
}

/// Facts resolved from the active transaction and the recorded parse of the
/// producer's real output. The host checks EV6 on the neutral input/read set
/// and isolation evidence before supplying the execution's isolation reference.
pub struct IrlAdoptionEvidence<'a> {
    pub execution: StageExecution<'a>,
    pub parsed_graph: &'a SemanticGraphV2,
    pub excerpts: &'a [IrlExcerpt<'a>],
    pub registry: &'a ConceptRegistry,
    pub time_facts: &'a [ObjectRef],
    pub producer: &'a str,
    pub producer_version: &'a str,
    pub verified_neutral_input: &'a ObjectRef,
}

impl ReceivedCandidate<'_> {
    /// Check the immutable link before staging it alongside the IRL stage.
    /// Only the outer transaction can publish the linked records.
    pub fn check_adoption(
        &self,
        link: &IrlAdoptionLink,
        candidate: &SemanticCandidateV2,
        stage: &StageAdoption,
        evidence: &IrlAdoptionEvidence<'_>,
        view: &impl SemanticReadView,
        limits: Limits,
    ) -> Result<()> {
        let p = &self.returned.producer;
        let seen = &evidence.execution;
        let graph = evidence.parsed_graph;
        link.canonical_bytes(limits)?;
        require(
            seen.binding == self.binding
                && seen.event == &self.event
                && seen.stage == InterpretationStage::INTERPRET_STAGE_IRL
                && seen.candidate == Some(&link.candidate)
                && seen.actual_output == Some(&p.output)
                && stage.outcome != StageOutcome::STAGE_OUTCOME_FAILED
                && p.isolation.as_ref() == Some(seen.isolation)
                && p.neutral_input.as_ref() == Some(evidence.verified_neutral_input)
                && p.remote
                    .as_ref()
                    .is_none_or(|r| r.status == RemoteAttemptStatus::REMOTE_ATTEMPT_COMPLETED)
                && &candidate.binding == self.binding
                && candidate.actual_output == p.output
                && candidate.producer == evidence.producer
                && candidate.producer_version == evidence.producer_version
                && self.returned.frame.graph.as_ref() == Some(&candidate.candidate_graph)
                && graph.registry == self.binding.environment.registry
                && view.transaction() == Some(seen.transaction_id)
                && view.branch() == self.binding.branch_id
                && view.base_commit() == self.binding.base_commit,
            "unbound IRL adoption",
        )?;
        check_object(&link.candidate_return, &self.returned, limits)?;
        check_object(&link.candidate, candidate, limits)?;
        check_object(&link.stage, stage, limits)?;
        check_object(&candidate.candidate_graph, graph, limits)?;
        require(
            evidence.excerpts.len() <= limits.max_items,
            "IRL excerpt count",
        )?;
        let locations = evidence
            .excerpts
            .iter()
            .map(|item| {
                locate_excerpt(
                    view,
                    &self.binding.base_commit,
                    Some(seen.transaction_id),
                    item.field,
                    item.quote,
                    item.known_range,
                    limits,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        for meta in graph
            .references
            .iter()
            .map(|n| &n.source)
            .chain(graph.statements.iter().map(|n| &n.source))
            .chain(graph.actions.iter().map(|n| &n.source))
        {
            require(
                meta.locations
                    .iter()
                    .filter(|l| l.text.is_some())
                    .all(|l| locations.contains(l)),
                "graph text differs from excerpts in actual output",
            )?;
            require(
                meta.roots
                    .iter()
                    .all(|r| r == &self.event || self.binding.read_set.contains(&r.content)),
                "source outside IRL dispatch",
            )?;
        }
        check_graph_registry(graph, evidence.registry, limits)?;
        check_graph_sources(
            graph,
            view,
            &self.binding.base_commit,
            Some(seen.transaction_id),
            evidence.time_facts,
            limits,
        )?;
        check_stage_execution(stage, seen, limits)
    }
}
