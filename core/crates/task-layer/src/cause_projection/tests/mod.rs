//! Test codec and fixture builders for cause-projection unit and acceptance tests.

use std::collections::BTreeMap;

use super::{
    AssemblyInput, BasisId, Binding, Budget, DItem, Delivered, Eligibility, HPos, Member, Outcome,
    PresentedD, PresentedE, Record, RecordId, RecordTable, Relation, RelationKind, RequestCodec,
    RequestPlan, SentNote, Slot, assemble,
};

mod accept_3_2;
mod closure;
mod determinism;
mod golden_2_3;
mod rank;
mod step0_decisions;

/// Deterministic human-readable codec writing sections in fixed `P -> D -> E -> o` order
/// (Card §6; ADR 0042 §2.1).
pub(super) struct TestCodec;

impl RequestCodec for TestCodec {
    fn encode(&self, plan: &RequestPlan, records: &RecordTable) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"strict=");
        out.extend_from_slice(if plan.strict { b"true\n" } else { b"false\n" });

        out.extend_from_slice(b"[P]\n");
        out.extend_from_slice(&plan.protocol);
        out.push(b'\n');

        out.extend_from_slice(b"[D]\n");
        for item in &plan.dialogue {
            let body = records
                .get(&item.id)
                .map_or(&[][..], |r| match item.delivered {
                    Delivered::Full => r.body.as_slice(),
                    Delivered::Partial { start, end } => {
                        let s = usize::try_from(start).unwrap_or(r.body.len());
                        let e = usize::try_from(end).unwrap_or(r.body.len());
                        r.body.get(s..e).unwrap_or(&[])
                    }
                });
            write_framed_record(&mut out, &item.id, body, &item.continuation_refs);
        }

        out.extend_from_slice(b"[E]\n");
        for item in &plan.experience {
            let body = records.get(&item.id).map_or(&[][..], |r| r.body.as_slice());
            write_framed_record(&mut out, &item.id, body, &item.continuation_refs);
        }

        out.extend_from_slice(b"[o]\n");
        let latest_body = records
            .get(&plan.latest)
            .map_or(&[][..], |r| r.body.as_slice());
        write_framed_record(
            &mut out,
            &plan.latest,
            latest_body,
            &plan.latest_continuation_refs,
        );

        out
    }

    fn measure(&self, bytes: &[u8]) -> u64 {
        u64::try_from(bytes.len()).unwrap_or(u64::MAX)
    }
}

/// Appends one deterministic line with id, presented body bytes, and continuation refs.
fn write_framed_record(out: &mut Vec<u8>, id: &RecordId, body: &[u8], refs: &[RecordId]) {
    out.extend_from_slice(b"id=");
    out.extend_from_slice(&id.0);
    out.extend_from_slice(b"|body=");
    out.extend_from_slice(body);
    out.extend_from_slice(b"|refs=[");
    for (idx, r) in refs.iter().enumerate() {
        if idx > 0 {
            out.push(b',');
        }
        out.extend_from_slice(&r.0);
    }
    out.extend_from_slice(b"]\n");
}

/// Builds a `RecordId` from an ASCII/UTF-8 label.
pub(super) fn rid(s: &str) -> RecordId {
    RecordId(s.as_bytes().to_vec())
}

/// Builds a `BasisId` from an ASCII/UTF-8 label.
pub(super) fn bid(s: &str) -> BasisId {
    BasisId(s.as_bytes().to_vec())
}

/// Builds an `Eligible` record with default content version `v1`.
pub(super) fn rec(id: &str, h_pos: u64, body: &str) -> Record {
    Record {
        id: rid(id),
        content_version: b"v1".to_vec(),
        h_pos: HPos(h_pos),
        body: body.as_bytes().to_vec(),
        eligibility: Eligibility::Eligible,
    }
}

/// Builds a record with custom `Eligibility`.
pub(super) fn rec_with(id: &str, h_pos: u64, body: &str, eligibility: Eligibility) -> Record {
    Record {
        id: rid(id),
        content_version: b"v1".to_vec(),
        h_pos: HPos(h_pos),
        body: body.as_bytes().to_vec(),
        eligibility,
    }
}

/// Builds an `Ineligible` record with `reason`.
pub(super) fn rec_inelig(id: &str, h_pos: u64, body: &str, reason: &str) -> Record {
    rec_with(
        id,
        h_pos,
        body,
        Eligibility::Ineligible {
            reason: reason.to_string(),
        },
    )
}

/// Builds a `Relation` edge.
pub(super) fn rel(from: &str, to: &str, kind: RelationKind) -> Relation {
    Relation {
        from: rid(from),
        to: rid(to),
        kind,
    }
}

/// Builds a `Member` for a slot.
pub(super) fn member(basis: &str, sources: &[&str]) -> Member {
    Member {
        basis: bid(basis),
        sources: sources.iter().map(|s| rid(s)).collect(),
    }
}

/// Builds a single-member `Slot` with `merge_undecided = false`.
pub(super) fn slot(basis: &str, h: Vec<f64>, first_f_pos: u64, sources: &[&str]) -> Slot {
    Slot {
        basis: bid(basis),
        h,
        first_f_pos: HPos(first_f_pos),
        members: vec![member(basis, sources)],
        merge_undecided: false,
    }
}

/// Builds an `AssemblyInput` with unrelated base records `x1` (D, Full, pos 100) and `x2` (o, pos 101).
pub(super) fn input_with_unrelated_base(
    extra_records: Vec<Record>,
    relations: Vec<Relation>,
    slots: Vec<Slot>,
) -> AssemblyInput {
    let mut records = vec![rec("x1", 100, "base_d"), rec("x2", 101, "base_o")];
    records.extend(extra_records);
    AssemblyInput {
        protocol: b"P:facts".to_vec(),
        protocol_cited: Vec::new(),
        dialogue: vec![DItem {
            id: rid("x1"),
            delivered: Delivered::Full,
        }],
        latest: rid("x2"),
        required_facts: Vec::new(),
        records,
        relations,
        frontier_complete: true,
        slots,
        decomposition_unknown: false,
        budget: Budget {
            capacity: 1_000_000,
            output_reserve: 0,
        },
    }
}

/// Builds a custom `AssemblyInput` with explicit `protocol`, `dialogue`, `latest`, and high budget.
pub(super) fn custom_input(
    protocol: &[u8],
    dialogue: Vec<DItem>,
    latest: &str,
    records: Vec<Record>,
    relations: Vec<Relation>,
    slots: Vec<Slot>,
) -> AssemblyInput {
    AssemblyInput {
        protocol: protocol.to_vec(),
        protocol_cited: Vec::new(),
        dialogue,
        latest: rid(latest),
        required_facts: Vec::new(),
        records,
        relations,
        frontier_complete: true,
        slots,
        decomposition_unknown: false,
        budget: Budget {
            capacity: 1_000_000,
            output_reserve: 0,
        },
    }
}

/// Runs `assemble` and unwraps `Outcome::Sent` for tests expecting a valid request.
pub(super) fn expect_sent(input: &AssemblyInput) -> (RequestPlan, Vec<u8>, Binding, SentNote) {
    match assemble(input, &TestCodec).expect("expected valid input") {
        Outcome::Sent {
            plan,
            bytes,
            binding,
            note,
        } => (plan, bytes, binding, note),
        other => panic!("expected Outcome::Sent, got {other:?}"),
    }
}

/// Extracts `RecordId`s from `plan.experience`.
pub(super) fn exp_ids(plan: &RequestPlan) -> Vec<RecordId> {
    plan.experience.iter().map(|e| e.id.clone()).collect()
}

/// Computes `TestCodec` byte length of the default unrelated base (`D=[x1], E=[], o=x2`).
pub(super) fn unrelated_base_len(input: &AssemblyInput) -> u64 {
    measure_plan(
        &input.protocol,
        vec![("x1", Delivered::Full, &[])],
        vec![],
        ("x2", &[]),
        &input.records,
    )
}

/// Computes exact `TestCodec` byte length for a candidate plan over `records` (no magic budgets).
pub(super) fn measure_plan(
    protocol: &[u8],
    dialogue: Vec<(&str, Delivered, &[&str])>,
    experience: Vec<(&str, &[&str])>,
    latest: (&str, &[&str]),
    records: &[Record],
) -> u64 {
    let mut map = BTreeMap::new();
    for r in records {
        map.insert(r.id.clone(), r.clone());
    }
    let table = RecordTable::new(map);
    let plan = RequestPlan {
        protocol: protocol.to_vec(),
        dialogue: dialogue
            .into_iter()
            .map(|(id, delivered, refs)| PresentedD {
                id: rid(id),
                delivered,
                continuation_refs: refs.iter().map(|s| rid(s)).collect(),
            })
            .collect(),
        experience: experience
            .into_iter()
            .map(|(id, refs)| PresentedE {
                id: rid(id),
                continuation_refs: refs.iter().map(|s| rid(s)).collect(),
            })
            .collect(),
        latest: rid(latest.0),
        latest_continuation_refs: latest.1.iter().map(|s| rid(s)).collect(),
        strict: false,
    };
    let bytes = TestCodec.encode(&plan, &table);
    TestCodec.measure(&bytes)
}
