use super::{
    Fault, aliases::Aliases, attributes::*, graph::SourceFacts, irl::IrlReadView, tests::*,
};
use core_types::{
    WireMessage, contract,
    generated::*,
    semantic::{SemanticReadView, VisibleContent},
};

/// Two distinct source records. Related reports can point back to the same
/// observed event without counting the report itself as another unit.
struct Input {
    at: RecordVersion,
    envelope: EventEnvelope,
    bytes: Vec<u8>,
}
impl Input {
    fn new<T: WireMessage>(id: &str, kind: EventKind, payload: T) -> Self {
        let content = reference(&payload);
        let mut envelope: EventEnvelope = fixture();
        envelope.id = id.into();
        envelope.kind = kind;
        envelope.payload_digest = content.digest.clone();
        envelope.payload_ref = content.clone();
        let at = RecordVersion {
            schema_version: 1,
            branch_id: "b".into(),
            record_id: id.into(),
            content,
        };
        Self {
            at,
            envelope,
            bytes: payload.canonical_bytes(limits()).unwrap(),
        }
    }
}

use AttributeSource::*;
use EventKind::*;
use EvidenceAccess::*;
struct View {
    event: Input,
    unit: Input,
    inputs: Vec<RecordVersion>,
    table: AttributeRuleTable,
    pinned: ObjectRef,
    origin: AttributeSource,
    aliases: Aliases,
    account: String,
    metadata: SourceMetadataV2,
    occurred: Vec<ScopedVerification>,
    truth: Vec<ScopedVerification>,
    subjective: Vec<ScopedVerification>,
    authorized: bool,
    epistemic: EpistemicType,
}
impl View {
    fn new(event: Input, origin: AttributeSource) -> Self {
        let unit = Input::new("episode", OBSERVATION, fixture::<TouchObservationV2>());
        let inputs = vec![event.at.clone(), unit.at.clone()];
        let (pinned, table) = initial_attribute_rules(limits()).unwrap();
        let aliases = Aliases::build(
            &["alice".into(), "account-1".into(), "account-2".into()],
            &[
                ("account-1".into(), "alice".into()),
                ("account-2".into(), "alice".into()),
            ],
            limits(),
        )
        .unwrap();
        let whole = SourceLocation {
            schema_version: 1,
            source: event.at.clone(),
            path: vec![],
            text: None,
        };
        let mut loc = whole.clone();
        let human = origin == ATTR_SOURCE_HUMAN_RESPONSE;
        let epistemic = if human {
            EpistemicType::USER_ASSERTION
        } else {
            EpistemicType::SYSTEM_RECORD
        };
        let speaker = if human {
            let range = Utf8Range {
                schema_version: 1,
                start: core_types::scalar::DecimalU64(0),
                end: core_types::scalar::DecimalU64(5),
            };
            loc.path.push(SourceField {
                schema_version: 1,
                field_number: 2,
                element: None,
            });
            loc.text = Some(range.clone());
            Some(SpeakerAttribution {
                schema_version: 1,
                governance: event.envelope.provenance.clone(),
                layers: vec![QuoteLayer {
                    schema_version: 1,
                    source: event.at.clone(),
                    visible: range,
                    principal_id: "alice".into(),
                    identity_evidence: event.envelope.provenance.clone(),
                }],
            })
        } else {
            None
        };
        let metadata = SourceMetadataV2 {
            schema_version: 1,
            locations: vec![loc.clone()],
            roots: vec![event.at.clone()],
            transformations: vec![],
            representation_version: "v1".into(),
            derivation: Derivation::VERBATIM,
            epistemic_type: epistemic,
            resolution_state: ResolutionState::EXISTING,
            producer: event.envelope.provenance.clone(),
            validity: event.envelope.provenance.clone(),
            speaker,
            conversation: None,
        };
        let observed = ScopedVerification {
            schema_version: 1,
            scope: whole,
            value: VerificationValue::VERIFY_VALUE_TRUE,
            evidence: vec![event.at.clone()],
            reason: None,
            experiencer: None,
        };
        let unknown = ScopedVerification {
            scope: loc.clone(),
            value: VerificationValue::VERIFY_VALUE_UNKNOWN,
            evidence: vec![],
            reason: Some("unverified content".into()),
            ..observed.clone()
        };
        let subjective = if human {
            ScopedVerification {
                scope: loc,
                experiencer: Some("alice".into()),
                ..observed.clone()
            }
        } else {
            unknown.clone()
        };
        Self {
            pinned,
            event,
            unit,
            inputs,
            table,
            origin,
            aliases,
            account: "account-1".into(),
            metadata,
            occurred: vec![observed],
            truth: vec![unknown],
            subjective: vec![subjective],
            authorized: true,
            epistemic,
        }
    }
    fn run(&self) -> crate::Result<EventAttributesRecordV2> {
        stage_attributes(self, &self.event.at, limits())
    }
}
impl SemanticReadView for View {
    fn branch(&self) -> &str {
        "b"
    }
    fn base_commit(&self) -> &str {
        "prefix"
    }
    fn transaction(&self) -> Option<&str> {
        Some("tx")
    }
    fn authorize_provenance(&self, m: &SourceMetadataV2) -> contract::Result<()> {
        if self.authorized && m == &self.metadata {
            Ok(())
        } else {
            Err("revoked".into())
        }
    }
    fn content(&self, id: &RecordVersion) -> contract::Result<VisibleContent<'_>> {
        let v = [&self.event, &self.unit]
            .into_iter()
            .find(|v| &v.at == id)
            .ok_or("missing")?;
        Ok(VisibleContent::Event(&v.envelope, &v.bytes))
    }
}
impl IrlReadView for View {
    fn source_facts(&self, _: &RecordVersion) -> contract::Result<SourceFacts> {
        Ok(SourceFacts {
            epistemic: self.epistemic,
            self_narrative: self.origin == ATTR_SOURCE_INTERNAL,
            speaker: None,
            verifiable: false,
        })
    }
}
impl AttributeView for View {
    fn attribute_facts(&self, event: &RecordVersion) -> contract::Result<AttributeFacts<'_>> {
        if event != &self.event.at {
            return Err("different event".into());
        }
        let person = matches!(
            self.origin,
            ATTR_SOURCE_HUMAN_RESPONSE | ATTR_SOURCE_DIRECTED_TOUCH
        );
        Ok(AttributeFacts {
            source: self.origin,
            identity: if person {
                SourceIdentity::Account {
                    account: &self.account,
                    aliases: &self.aliases,
                    mapping: &self.metadata.validity,
                }
            } else {
                SourceIdentity::Other("source")
            },
            metadata: &self.metadata,
            table: &self.table,
            pinned_table: &self.pinned,
            governance: &self.metadata.validity,
            inputs: &self.inputs,
            unit: match self.origin {
                ATTR_SOURCE_HUMAN_RESPONSE => Some(&self.event.at),
                ATTR_SOURCE_DIRECTED_TOUCH => Some(&self.unit.at),
                _ => None,
            },
            directed_to_n: person.then_some(&self.event.at),
            cause: (self.origin == ATTR_SOURCE_SELF_RESULT).then_some(&self.unit.at),
            episode_phase: (self.origin == ATTR_SOURCE_DIRECTED_TOUCH)
                .then_some(&self.metadata.validity),
            duration: None,
            occurred: &self.occurred,
            truth: &self.truth,
            subjective: &self.subjective,
            targets: &[],
        })
    }
}
fn reply() -> View {
    View::new(
        Input::new(
            "reply",
            OBSERVATION,
            UserText {
                schema_version: 1,
                text: "hello".into(),
            },
        ),
        ATTR_SOURCE_HUMAN_RESPONSE,
    )
}
#[test]
fn g05_rule_selects_the_authenticated_source_kind_and_payload() {
    for (input, source, external, narrative, access) in [
        (
            reply().event,
            ATTR_SOURCE_HUMAN_RESPONSE,
            true,
            false,
            EVIDENCE_ACCESS_TARGET_DEPENDENT,
        ),
        (
            Input::new(
                "voice",
                OBSERVATION,
                UserUtterance {
                    text: "hello".into(),
                    ..fixture()
                },
            ),
            ATTR_SOURCE_HUMAN_RESPONSE,
            true,
            false,
            EVIDENCE_ACCESS_TARGET_DEPENDENT,
        ),
        (
            Input::new("report", OBSERVATION, fixture::<TouchObservationV2>()),
            ATTR_SOURCE_DIRECTED_TOUCH,
            true,
            false,
            EVIDENCE_ACCESS_TARGET_DEPENDENT,
        ),
        (
            Input::new("desktop", OBSERVATION, fixture::<DesktopObservation>()),
            ATTR_SOURCE_DESKTOP,
            false,
            false,
            EVIDENCE_ACCESS_TARGET_DEPENDENT,
        ),
        (
            Input::new("result", EXECUTION_RESULT, fixture::<BodyActionResult>()),
            ATTR_SOURCE_SELF_RESULT,
            false,
            false,
            EVIDENCE_ACCESS_COMPARISON_REQUIRED,
        ),
        (
            Input::new("delivery", EXECUTION_RESULT, fixture::<SpeechDelivery>()),
            ATTR_SOURCE_SELF_RESULT,
            false,
            false,
            EVIDENCE_ACCESS_COMPARISON_REQUIRED,
        ),
        (
            Input::new(
                "delivery-v2",
                EXECUTION_RESULT,
                fixture::<SpeechDeliveryV2>(),
            ),
            ATTR_SOURCE_SELF_RESULT,
            false,
            false,
            EVIDENCE_ACCESS_COMPARISON_REQUIRED,
        ),
        (
            Input::new(
                "historical-thought",
                OBSERVATION,
                fixture::<InternalActivityRecord>(),
            ),
            ATTR_SOURCE_INTERNAL,
            false,
            true,
            EVIDENCE_ACCESS_NONE,
        ),
        (
            Input::new(
                "thought",
                OBSERVATION,
                fixture::<InternalActivityRecordV2>(),
            ),
            ATTR_SOURCE_INTERNAL,
            false,
            true,
            EVIDENCE_ACCESS_NONE,
        ),
        (
            Input::new("maintenance", MAINTENANCE, fixture::<TerminalMaintenance>()),
            ATTR_SOURCE_MAINTENANCE,
            false,
            false,
            EVIDENCE_ACCESS_NONE,
        ),
        (
            Input::new(
                "maintenance-record",
                MAINTENANCE,
                fixture::<SystemMaintenanceRecord>(),
            ),
            ATTR_SOURCE_MAINTENANCE,
            false,
            false,
            EVIDENCE_ACCESS_NONE,
        ),
    ] {
        let mut v = View::new(input, source);
        let a = v.run().unwrap();
        assert_eq!(
            (a.exogenous, a.self_narrative, a.applied.growth_access),
            (external, narrative, access)
        );
        assert_eq!(a.rule_table, v.pinned);
        assert_eq!(a.c_unit.is_some(), external);
        let first = a.canonical_bytes(limits()).unwrap();
        v.table.rules.reverse();
        assert_eq!(first, v.run().unwrap().canonical_bytes(limits()).unwrap());
        v.origin = if source == ATTR_SOURCE_DESKTOP {
            ATTR_SOURCE_ABSENCE
        } else {
            ATTR_SOURCE_DESKTOP
        };
        assert_eq!(v.run(), Err(Fault::UnsupportedSource));
    }
    let mut v = reply();
    for kind in [
        STARTUP,
        ACTION_REQUEST,
        EXECUTION_RESULT,
        CONSEQUENCE,
        CANCEL,
        CORRECTION,
        MAINTENANCE,
    ] {
        v.event.envelope.kind = kind;
        assert_eq!(v.run(), Err(Fault::UnsupportedSource));
    }
}
#[test]
fn g05_preserves_independent_scopes_aliases_and_shared_episode_units() {
    let mut v = reply();
    let a = v.run().unwrap();
    assert_eq!(a.occurred[0].value, VerificationValue::VERIFY_VALUE_TRUE);
    assert_ne!(a.occurred[0].scope, a.truth[0].scope);
    assert_eq!(a.truth[0].value, VerificationValue::VERIFY_VALUE_UNKNOWN);
    assert!(a.truth[0].reason.is_some());
    assert_eq!(a.subjective[0].experiencer.as_deref(), Some("alice"));
    v.account = "account-2".into();
    assert_eq!(a, v.run().unwrap());
    let touch = |id| {
        View::new(
            Input::new(id, OBSERVATION, fixture::<TouchObservationV2>()),
            ATTR_SOURCE_DIRECTED_TOUCH,
        )
        .run()
        .unwrap()
    };
    let (a, b) = (touch("first"), touch("second"));
    assert_ne!(a.event, b.event);
    assert_eq!(a.c_unit, b.c_unit);
}
#[test]
fn g05_rejects_unbound_rules_invalid_scopes_and_revoked_or_promoted_sources() {
    for change in [
        |v: &mut View| v.authorized = false,
        |v: &mut View| v.pinned.digest.0[0] ^= 1,
        |v: &mut View| v.table.rules.push(v.table.rules[0].clone()),
        |v: &mut View| v.truth[0].reason = None,
        |v: &mut View| v.truth[0].scope.text.as_mut().unwrap().end.0 = 6,
        |v: &mut View| v.subjective[0].experiencer = None,
        |v: &mut View| v.epistemic = EpistemicType::MODEL_INFERENCE,
        |v: &mut View| v.account = "unmapped".into(),
        |v: &mut View| v.event.bytes.push(0),
    ] {
        let mut v = reply();
        change(&mut v);
        assert!(v.run().is_err());
    }
}
