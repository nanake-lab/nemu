#![allow(dead_code)]
pub mod adapters;
use core_runtime::{binding::digest, clock::*, pipeline::Input, stages::*};
use core_types::{WireMessage, contract::Limits, generated::*, scalar::DecimalU64};
pub fn limits() -> Limits {
    Limits {
        max_bytes: 4 << 20,
        max_items: 4096,
        max_depth: 64,
    }
}
pub fn theta_bits(t: &ThetaSnapshot) -> Vec<u64> {
    let b = &t.values.baseline;
    let mut out = vec![
        b.connection.to_bits(),
        b.empathy.to_bits(),
        b.curiosity.to_bits(),
        b.achievement.to_bits(),
        t.weight_sum.to_bits(),
        t.h_hat_theta.to_bits(),
    ];
    for s in [&t.values.gain, &t.values.recovery_per_s, &t.sigma, &t.tau_s] {
        out.extend([s.connection, s.empathy, s.curiosity, s.achievement].map(f64::to_bits));
    }
    out
}
pub fn r_bits(r: &RSnapshotV3) -> Vec<u64> {
    let mut out = vec![];
    for t in &r.tendencies {
        let d = &t.direction;
        out.extend(
            [
                d.connection,
                d.empathy,
                d.curiosity,
                d.achievement,
                t.evidence_a,
                t.evidence_b,
                t.h_hat,
                t.pi,
                t.typical_magnitude,
                t.magnitude_sum,
                t.magnitude_weight,
                t.evidence_absolute,
            ]
            .map(f64::to_bits),
        );
        out.extend(t.attachment_prior.map(f64::to_bits));
    }
    out
}
pub fn fixture<T: WireMessage>() -> T {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{}.hex", T::NAME));
    let hex = std::fs::read_to_string(path).unwrap();
    T::from_canonical(&core_types::scalar::unhex(hex.trim()).unwrap(), limits()).unwrap()
}
// Structural storage fixtures only. Generated canonical data is changed through
// the schema codec; these synthetic resources are never production L/Q/U roots.
pub fn with_refs<T: WireMessage>(reference: &ObjectRef) -> T {
    let schema = core_types::contract::Schema::builtin().unwrap();
    let mut value = schema
        .decode_canonical(
            T::NAME,
            &fixture::<T>().canonical_bytes(limits()).unwrap(),
            limits(),
            0,
        )
        .unwrap();
    let leaf = schema
        .decode_canonical(
            ObjectRef::NAME,
            &reference.canonical_bytes(limits()).unwrap(),
            limits(),
            0,
        )
        .unwrap();
    let mut pending = vec![&mut value];
    while let Some(v) = pending.pop() {
        if v.get("object_type").is_some() && v.get("digest").is_some() {
            *v = leaf.clone();
        } else if v.is_object() {
            pending.extend(v.as_object_mut().unwrap().values_mut());
        } else if v.is_array() {
            pending.extend(v.as_array_mut().unwrap().iter_mut());
        }
    }
    T::from_canonical(
        &schema.encode_canonical(T::NAME, &value, limits()).unwrap(),
        limits(),
    )
    .unwrap()
}
pub fn bounds() -> Bounds {
    Bounds {
        max_interval_ns: 10_000,
        max_wall_skew_ns: 2,
        max_uncertainty_ns: 1,
    }
}
pub fn sample(n: u64) -> Sample {
    Sample {
        boot: "boot".into(),
        epoch: "epoch".into(),
        monotonic_ns: n,
        wall_ns: 100 + n as i64,
        uncertainty_ns: Some(0),
    }
}
pub struct Source(pub Sample, pub usize);
impl Provider for Source {
    fn sample(&mut self) -> Sample {
        self.1 += 1;
        self.0.clone()
    }
}
pub fn time(ns: u64) -> ProcessTime {
    Clock::start(sample(0), 0, bounds())
        .unwrap()
        .capture(&mut Source(sample(ns), 0))
        .unwrap()
}
pub struct Case {
    pub event: EventEnvelope,
    pub attributes: EventAttributesRecord,
    pub base: Snapshot,
    pub facts: FactProjection,
    pub binding: ProposalBindingV2,
}
impl Case {
    pub fn new() -> Self {
        let event: EventEnvelope = fixture();
        let mut attributes: EventAttributesRecord = fixture();
        attributes.event_id = event.id.clone();
        attributes.exogenous = true;
        attributes.self_narrative = false;
        let mut base = Snapshot {
            x: fixture(),
            r: fixture(),
            theta: fixture(),
        };
        base.x.anchor_time_ns = DecimalU64(10);
        base.x.nature.connection = 0.1;
        base.x.learned.clear();
        base.r.tendencies.clear();
        base.theta.values.baseline.connection = 0.0;
        let mut binding: ProposalBindingV2 = fixture();
        binding.event_id = event.id.clone();
        binding.raw_log_position = event.raw_log_position;
        binding.anchor_time_ns = base.x.anchor_time_ns;
        binding.process_time_ns = DecimalU64(30);
        binding.x_revision = base.x.revision.clone();
        binding.r_revision = base.r.revision.clone();
        binding.theta_revision = base.theta.revision.clone();
        let resource = binding.environment.model.clone();
        binding.environment.nature_appraiser = Some(resource.clone());
        binding.environment.situation_matcher = Some(resource.clone());
        binding.environment.direction_set = Some(resource.clone());
        binding.environment.law_version = Some(resource);
        let mut facts: FactProjection = fixture();
        facts.history_prefix = binding.base_commit.clone();
        binding.c_core_digest = digest(&facts, limits()).unwrap();
        let mut projected = base.x.clone();
        projected.anchor_time_ns = DecimalU64(30);
        binding.projected_state_digest = digest(&projected, limits()).unwrap();
        binding.accepted_z_digest = digest(&fixture::<SemanticGraph>(), limits()).unwrap();
        Self {
            event,
            attributes,
            base,
            facts,
            binding,
        }
    }
    pub fn input(&self, paused: bool) -> Input<'_> {
        Input {
            event: self.event.clone(),
            attributes: &self.attributes,
            base: self.base.clone(),
            facts: &self.facts,
            binding: self.binding.clone(),
            time: time(30),
            growth_paused: paused,
        }
    }
}
