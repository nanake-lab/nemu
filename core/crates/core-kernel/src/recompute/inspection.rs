use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Supplied by an independent service; Missing/Failed/TimedOut are observations
/// to record, not deadlines to measure again during replay.
#[derive(Clone, Debug)]
pub struct IndependentResult<S> {
    pub binding: Binding,
    pub implementation: String,
    pub result: std::result::Result<S, Unavailable>,
}

/// Compare complete states under the *same* law and H. Identity checks cannot
/// prove implementation independence; that is an explicit caller obligation.
pub fn verify_candidate<E, K: Kernel<E>>(
    kernel: &K,
    binding: &Binding,
    primary_implementation: &str,
    candidate: &K::State,
    independent: Option<&IndependentResult<K::State>>,
) -> Result<CheckOutcome> {
    let Some(other) = independent else {
        return Ok(CheckOutcome::Unavailable(Unavailable::Missing));
    };
    if primary_implementation.is_empty() {
        return Err(Fault::InvalidInput(
            "primary implementation identity".into(),
        ));
    }
    if &other.binding != binding
        || other.implementation.is_empty()
        || other.implementation == primary_implementation
    {
        // A stale or non-independent response cannot prove disagreement. Treat
        // it as failed verification, which still applies and alarms (V2-97).
        return Ok(CheckOutcome::Unavailable(Unavailable::Failed));
    }
    Ok(match &other.result {
        Ok(state) if kernel.same_bits(candidate, state) => CheckOutcome::Consistent,
        Ok(_) => CheckOutcome::Inconsistent,
        Err(reason) => CheckOutcome::Unavailable(*reason),
    })
}

/// Neutral probes: one positive and one negative input per nature channel.
/// Amplitudes and thresholds come from a preregistered law calibration resource.
/// No personality scenario or default numerical value is chosen here.
#[derive(Clone, Debug)]
pub struct ProbeSet {
    version: String,
    law: String,
    amplitudes: [f64; 4],
    coefficient_limit: f64,
    body_limit: f64,
}
impl ProbeSet {
    pub fn new(
        version: String,
        law: String,
        amplitudes: [f64; 4],
        coefficient_limit: f64,
        body_limit: f64,
    ) -> Result<Self> {
        if version.is_empty()
            || law.is_empty()
            || amplitudes
                .iter()
                .any(|a| !a.is_finite() || *a <= 0.0 || *a > 1.0)
            || [coefficient_limit, body_limit]
                .iter()
                .any(|x| !x.is_finite() || *x <= 0.0)
        {
            return Err(Fault::InvalidInput("probe resource".into()));
        }
        Ok(Self {
            version,
            law,
            amplitudes,
            coefficient_limit,
            body_limit,
        })
    }
    pub fn vectors(&self) -> [[f64; 4]; 8] {
        std::array::from_fn(|i| {
            let mut vector = [0.0; 4];
            vector[i / 2] = if i % 2 == 0 {
                self.amplitudes[i / 2]
            } else {
                -self.amplitudes[i / 2]
            };
            vector
        })
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn evaluate(
        &self,
        mut compile: impl FnMut([f64; 4]) -> Result<crate::modulation::NumericCompilation>,
    ) -> Result<[ProbeReadout; 8]> {
        let mut rows = Vec::with_capacity(8);
        for vector in self.vectors() {
            rows.push(ProbeReadout {
                compilation: compile(vector)?,
                vector,
                calibration: self.version.clone(),
            });
        }
        rows.try_into()
            .map_err(|_| Fault::InvalidInput("probe count".into()))
    }
}

/// Numeric J output at one probe, supplied by the F/J projection owner. The
/// complete eight rows must use identical carrier/readout resources on both
/// sides; missing learned coefficients mean zero, not positional matching.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeReadout {
    pub compilation: crate::modulation::NumericCompilation,
    vector: [f64; 4],
    calibration: String,
}

/// Strictly greater triggers. Equality does not trigger; no clamp or epsilon
/// can hide a nonfinite result. This does not check storage/clock/carrier health.
pub fn compare_probes(
    probes: &ProbeSet,
    scope: &Scope,
    before: &[ProbeReadout; 8],
    after: &[ProbeReadout; 8],
) -> Result<bool> {
    if probes.law != scope.law {
        return Err(Fault::Binding("probe law"));
    }
    let mut alarm = false;
    let resources = &before[0].compilation.receipt.resources;
    let body_version = &before[0].compilation.body.version;
    let validation = &before[0].compilation.receipt.combination_validation;
    for ((a, b), vector) in before.iter().zip(after).zip(probes.vectors()) {
        if a.calibration != probes.version
            || b.calibration != probes.version
            || a.vector.map(f64::to_bits) != vector.map(f64::to_bits)
            || b.vector.map(f64::to_bits) != vector.map(f64::to_bits)
            || &a.compilation.receipt.resources != resources
            || &a.compilation.body.version != body_version
            || &a.compilation.receipt.combination_validation != validation
        {
            return Err(Fault::Binding("probe order/calibration/resources"));
        }
        let a = &a.compilation;
        let b = &b.compilation;
        if !a.receipt.excluded.is_empty()
            || !b.receipt.excluded.is_empty()
            || a.receipt.mode != crate::modulation::CompilationMode::Connected
            || b.receipt.mode != crate::modulation::CompilationMode::Connected
        {
            return Err(Fault::MissingInput("incomplete probe compilation".into()));
        }
        if a.receipt.resources != b.receipt.resources
            || a.receipt.combination_validation != b.receipt.combination_validation
            || a.receipt.body_readout_version != b.receipt.body_readout_version
            || a.body.version != b.body.version
            || a.receipt.mode != b.receipt.mode
        {
            return Err(Fault::Binding("probe resource mismatch"));
        }
        let aa = coefficients(a)?;
        let bb = coefficients(b)?;
        let ids: BTreeSet<_> = aa.keys().chain(bb.keys()).collect();
        for id in ids {
            alarm |= exceeds(
                *aa.get(id).unwrap_or(&0.0),
                *bb.get(id).unwrap_or(&0.0),
                probes.coefficient_limit,
            )?;
        }
        for (x, y) in [a.body.facing, a.body.tension, a.body.energy]
            .into_iter()
            .zip([b.body.facing, b.body.tension, b.body.energy])
        {
            alarm |= exceeds(x, y, probes.body_limit)?;
        }
    }
    Ok(alarm)
}

fn coefficients(c: &crate::modulation::NumericCompilation) -> Result<BTreeMap<(u8, String), f64>> {
    let n = &c.nature_coefficients;
    let mut out = BTreeMap::new();
    for (i, value) in [n.connection, n.empathy, n.curiosity, n.achievement]
        .into_iter()
        .enumerate()
    {
        out.insert((0, i.to_string()), value);
    }
    for learned in &c.learned_coefficients {
        if out
            .insert((1, learned.tendency_id.clone()), learned.coefficient)
            .is_some()
        {
            return Err(Fault::InvalidInput("duplicate probe coefficient".into()));
        }
    }
    Ok(out)
}

fn exceeds(a: f64, b: f64, limit: f64) -> Result<bool> {
    let difference = b - a;
    if !a.is_finite() || !b.is_finite() || !difference.is_finite() {
        return Err(Fault::Computation("nonfinite probe readout".into()));
    }
    Ok(difference.abs() > limit)
}
