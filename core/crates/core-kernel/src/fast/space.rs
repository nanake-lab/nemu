//! The box space X, its time projection Φ_x and the commit-side maps.

use std::cmp::Ordering;

use core_types::WireMessage;
use core_types::contract::Limits;
use core_types::generated::{FastStateSnapshotV2, LearnedComponent, NatureVector};
use core_types::scalar::DecimalU64;
use core_types::semantic::tendency_order;

use super::types::{Drive, FastError, FastState, LearnedValue};
use crate::effects::EffectiveEffects;

/// Version of the representation written into every snapshot.
pub const REPRESENTATION_VERSION: &str = "echo-core-v2.0/fast-state/1";

const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_items: 4096,
    max_depth: 32,
};

/// Clip to [−1, 1] and normalize −0 (ADR 0019 §8: no negative zero is stored).
fn clip(v: f64) -> f64 {
    let c = v.clamp(-1.0, 1.0);
    if c == 0.0 { 0.0 } else { c }
}

/// x is a valid point of X: finite, in [−1, 1], learned IDs non-empty, unique
/// and in canonical order.
pub fn validate(x: &FastState) -> Result<(), FastError> {
    let finite =
        x.nature.iter().all(|v| v.is_finite()) && x.learned.iter().all(|l| l.value.is_finite());
    if !finite {
        return Err(FastError::NonFinite("x"));
    }
    let in_box = x.nature.iter().all(|v| (-1.0..=1.0).contains(v))
        && x.learned.iter().all(|l| (-1.0..=1.0).contains(&l.value));
    if !in_box {
        return Err(FastError::OutOfRange("x"));
    }
    if x.learned.iter().any(|l| l.tendency_id.is_empty()) {
        return Err(FastError::Misaligned);
    }
    let ordered = x
        .learned
        .windows(2)
        .all(|w| tendency_order(&w[0].tendency_id, &w[1].tendency_id) == Ordering::Less);
    if !ordered {
        return Err(FastError::Misaligned);
    }
    Ok(())
}

/// The learned components of x must be exactly the tendencies of R⁺ in E_eff,
/// in the same order. F_out (`f_out`) establishes this after every change of R⁺.
fn check_aligned(x: &FastState, effects: &EffectiveEffects) -> Result<(), FastError> {
    validate(x)?;
    let same = x.learned.len() == effects.effects.len()
        && x.learned
            .iter()
            .zip(&effects.effects)
            .all(|(l, e)| l.tendency_id == e.tendency_id);
    if same {
        Ok(())
    } else {
        Err(FastError::Misaligned)
    }
}

/// λ_k = Σ_i |d_{k,i}| λ_i / Σ_i |d_{k,i}| (v2.0 §6 reference), summed in
/// the fixed channel order.
fn learned_rate(d: &[f64; 4], lambda: &[f64; 4]) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..4 {
        let w = d[i].abs();
        num += w * lambda[i];
        den += w;
    }
    num / den
}

/// One coordinate of Φ_x: x + (−expm1(−λΔ)) · (rest − x), then clipped.
/// For Δ = 0 the factor is exactly 0, so the result equals x bit for bit.
fn relax(x: f64, rest: f64, rate: f64, delta_s: f64) -> f64 {
    let pull = -libm::expm1(-(rate * delta_s));
    clip(x + pull * (rest - x))
}

/// Φ_x(Δ; x) with x_rest = (b, 0) and Λ from θ and the directions of R⁺
/// (v2.0 §6). `delta_ns` is the committed time between the anchor of x and
/// the processing time, in nanoseconds; it is converted to seconds because λ
/// is per second (V2-31). Pure: projecting twice from the same anchor gives
/// the same result as projecting once (no accumulated decay).
pub fn project_time(
    x: &FastState,
    effects: &EffectiveEffects,
    delta_ns: u64,
) -> Result<FastState, FastError> {
    check_aligned(x, effects)?;
    let theta = &effects.theta;
    if theta.lambda.iter().any(|l| !(l.is_finite() && *l > 0.0)) {
        return Err(FastError::OutOfRange("lambda"));
    }
    let delta_s = delta_ns as f64 / 1e9;
    let nature: [f64; 4] =
        std::array::from_fn(|i| relax(x.nature[i], theta.b[i], theta.lambda[i], delta_s));
    let learned = x
        .learned
        .iter()
        .zip(&effects.effects)
        .map(|(l, e)| LearnedValue {
            tendency_id: l.tendency_id.clone(),
            value: relax(l.value, 0.0, learned_rate(&e.d, &theta.lambda), delta_s),
        })
        .collect();
    Ok(FastState { nature, learned })
}

/// F_in: x̃ = Proj_X(x̄ + D(e)). The projection is a coordinate-wise clip, so
/// the map is 1-Lipschitz in the ∞-norm (ADR 0008 §1).
pub fn apply_drive(x: &FastState, drive: &Drive) -> Result<FastState, FastError> {
    validate(x)?;
    let aligned = x.learned.len() == drive.learned.len()
        && x.learned
            .iter()
            .zip(&drive.learned)
            .all(|(a, b)| a.tendency_id == b.tendency_id);
    if !aligned {
        return Err(FastError::Misaligned);
    }
    if drive.nature.iter().any(|v| !v.is_finite())
        || drive.learned.iter().any(|l| !l.value.is_finite())
    {
        return Err(FastError::NonFinite("drive"));
    }
    let nature: [f64; 4] = std::array::from_fn(|i| clip(x.nature[i] + drive.nature[i]));
    let learned = x
        .learned
        .iter()
        .zip(&drive.learned)
        .map(|(l, d)| LearnedValue {
            tendency_id: l.tendency_id.clone(),
            value: clip(l.value + d.value),
        })
        .collect();
    Ok(FastState { nature, learned })
}

/// F_out: align x with the R⁺ of the next snapshot. A tendency that entered
/// R⁺ starts at 0, one that left is removed, all other coordinates are kept
/// unchanged (v2.0 §6; O §5.3). x is never reset to the new b.
pub fn f_out(x: &FastState, next: &EffectiveEffects) -> Result<FastState, FastError> {
    validate(x)?;
    let learned = next
        .effects
        .iter()
        .map(|e| LearnedValue {
            tendency_id: e.tendency_id.clone(),
            value: x
                .learned
                .iter()
                .find(|l| l.tendency_id == e.tendency_id)
                .map_or(0.0, |l| l.value),
        })
        .collect();
    Ok(FastState {
        nature: x.nature,
        learned,
    })
}

/// ‖x − y‖∞ over the same coordinates.
pub fn distance(x: &FastState, y: &FastState) -> Result<f64, FastError> {
    validate(x)?;
    validate(y)?;
    let same_ids = x.learned.len() == y.learned.len()
        && x.learned
            .iter()
            .zip(&y.learned)
            .all(|(a, b)| a.tendency_id == b.tendency_id);
    if !same_ids {
        return Err(FastError::Misaligned);
    }
    let nature = x.nature.iter().zip(&y.nature).map(|(a, b)| (a - b).abs());
    let learned = x
        .learned
        .iter()
        .zip(&y.learned)
        .map(|(a, b)| (a.value - b.value).abs());
    Ok(nature.chain(learned).fold(0.0, f64::max))
}

/// Encode x as a `FastStateSnapshotV2` in CanonicalV1 bytes.
pub fn serialize(x: &FastState, revision: &str, anchor_time_ns: u64) -> Result<Vec<u8>, FastError> {
    validate(x)?;
    let msg = FastStateSnapshotV2 {
        schema_version: 1,
        revision: revision.to_owned(),
        representation_version: REPRESENTATION_VERSION.to_owned(),
        anchor_time_ns: DecimalU64(anchor_time_ns),
        nature: NatureVector {
            schema_version: 1,
            connection: x.nature[0],
            empathy: x.nature[1],
            curiosity: x.nature[2],
            achievement: x.nature[3],
        },
        learned: x
            .learned
            .iter()
            .map(|l| LearnedComponent {
                schema_version: 1,
                tendency_id: l.tendency_id.clone(),
                value: l.value,
            })
            .collect(),
    };
    msg.canonical_bytes(LIMITS).map_err(FastError::Codec)
}

/// Decode CanonicalV1 bytes of a `FastStateSnapshotV2`; returns x, the
/// revision and the anchor time. Rejects other representation versions.
pub fn deserialize(bytes: &[u8]) -> Result<(FastState, String, u64), FastError> {
    let msg = FastStateSnapshotV2::from_canonical(bytes, LIMITS).map_err(FastError::Codec)?;
    if msg.representation_version != REPRESENTATION_VERSION {
        return Err(FastError::RepresentationVersion(msg.representation_version));
    }
    let n = &msg.nature;
    let x = FastState {
        nature: [n.connection, n.empathy, n.curiosity, n.achievement],
        learned: msg
            .learned
            .into_iter()
            .map(|l| LearnedValue {
                tendency_id: l.tendency_id,
                value: l.value,
            })
            .collect(),
    };
    validate(&x)?;
    Ok((x, msg.revision, msg.anchor_time_ns.0))
}
