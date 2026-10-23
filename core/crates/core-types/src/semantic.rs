//! Semantic checks for Echo Core v2.0 values beyond the codec bounds (P5-21).
//!
//! The codec rejects unknown schema, nonfinite numbers, overflow and values outside
//! the annotated ranges (SYS21). These pure functions add the properties the codec
//! cannot express: unit directions d_k, strictly positive situation matches, the
//! open interval of π, the counting invariants of a tendency and the inequalities
//! v2.0 states between law parameters. They read nothing but their argument.
use crate::contract::{Result, require};
use crate::generated::{
    EvidenceAllocationV3, EvidenceGroupV2, EvidenceGroupV3, EvidenceSign, GenesisManifestV2,
    LawParameters, NatureScale, NatureVector, PersonRelation, RelationKind, SituationMatchRecord,
    Tendency, TendencyCandidateRecord, TendencyChange, TendencyV2, TendencyV3, ThetaSnapshot,
    ThetaValues,
};
use std::cmp::Ordering;

/// Allowed |‖d‖₂ − 1| for a direction d_k ∈ ℝ⁴. A few binary64 ulps; producers
/// normalize in binary64 before proposing (reading registered as V2-32).
pub const UNIT_NORM_TOLERANCE: f64 = 1e-12;

/// The four nature channels in the fixed order of v2.0 §3: connection, empathy,
/// curiosity, achievement.
pub fn nature(v: &NatureVector) -> [f64; 4] {
    [v.connection, v.empathy, v.curiosity, v.achievement]
}

/// Same fixed order for non-negative per-channel scales (g, λ, σ̂, τ̂).
pub fn scale(v: &NatureScale) -> [f64; 4] {
    [v.connection, v.empathy, v.curiosity, v.achievement]
}

/// d_k is a unit vector in ℝ⁴ (v2.0 §5.2, §7). Components are summed in fixed order.
pub fn check_direction(d: &NatureVector) -> Result<()> {
    let v = nature(d);
    require(v.iter().all(|x| x.is_finite()), "nonfinite direction")?;
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    require(
        (norm - 1.0).abs() <= UNIT_NORM_TOLERANCE,
        "direction is not a unit vector",
    )
}

/// Order of tendency IDs used by every `set=tendency_id` collection: the CanonicalV1
/// set order of their string keys (ADR 0007 §6), i.e. lexicographic on the u32
/// little-endian byte length followed by the UTF-8 bytes. F (P5-10) orders the
/// learned components of x the same way.
pub fn tendency_order(a: &str, b: &str) -> Ordering {
    let key = |s: &str| {
        let mut k = u32::try_from(s.len())
            .unwrap_or(u32::MAX)
            .to_le_bytes()
            .to_vec();
        k.extend_from_slice(s.as_bytes());
        k
    };
    key(a).cmp(&key(b))
}

/// Only tendencies with μ > 0 are listed (P5-21 card; v2.0 §1 μ ∈ [0, 1]).
pub fn check_situation_match(r: &SituationMatchRecord) -> Result<()> {
    require(
        r.matches.iter().all(|m| m.mu > 0.0 && m.mu <= 1.0),
        "listed match must have 0 < mu <= 1",
    )
}

pub fn check_candidate(c: &TendencyCandidateRecord) -> Result<()> {
    check_direction(&c.direction)
}

/// The stored counts of one tendency, shared by `Tendency` and `TendencyV2`.
struct Counts<'a> {
    direction: &'a NatureVector,
    pi: f64,
    a: f64,
    b: f64,
    groups: u64,
    flips: u64,
    last_sign: Option<EvidenceSign>,
    prior: Option<f64>,
    relation: Option<&'a PersonRelation>,
}

fn check_counts(t: &Counts<'_>) -> Result<()> {
    check_direction(t.direction)?;
    require(t.pi > -1.0 && t.pi < 1.0, "pi must lie in (-1, 1)")?;
    // B_k = n₀ + κ_k + Σ ω·W·|S| > 0 and |A_k| = |Σ ω·W·S| ≤ Σ ω·W·|S| < B_k.
    require(t.b > 0.0, "B must be positive")?;
    require(t.a.abs() <= t.b, "|A| must not exceed B")?;
    require(
        t.flips <= t.groups.saturating_sub(1),
        "flip count exceeds adjacent group pairs",
    )?;
    require(
        t.groups > 0 || t.last_sign.is_none(),
        "no group, so no last sign",
    )?;
    if t.prior.is_some() {
        require(
            t.relation
                .is_some_and(|r| r.kind == RelationKind::CLOSENESS),
            "attachment prior belongs to a closeness tendency",
        )?;
    }
    Ok(())
}

/// Counting invariants of a tendency under the v2.0 §5.2 reference form.
pub fn check_tendency(t: &Tendency) -> Result<()> {
    check_counts(&Counts {
        direction: &t.direction,
        pi: t.pi,
        a: t.evidence_a,
        b: t.evidence_b,
        groups: t.group_count.0,
        flips: t.flip_count.0,
        last_sign: t.last_sign,
        prior: t.attachment_prior,
        relation: t.relation.as_ref(),
    })
}

/// `check_tendency` for the successor (ADR 0016 §3, as corrected after review), plus two
/// exact consequences of the recurrence: before the first group the typical-magnitude sums
/// are M = U = 0, and while U = 0 the typical magnitude is r_k = 0.
///
/// There is deliberately no relation between M and U beyond these. They are rounded
/// separately, so at the subnormal edge a legal recurrence can leave U = 0 with M ≠ 0
/// (review of PR #52: two groups with μ = 2^-1074 reach M = 2^-1074, U = 0); such a state
/// is valid and r_k is 0.
pub fn check_tendency_v2(t: &TendencyV2) -> Result<()> {
    check_counts(&Counts {
        direction: &t.direction,
        pi: t.pi,
        a: t.evidence_a,
        b: t.evidence_b,
        groups: t.group_count.0,
        flips: t.flip_count.0,
        last_sign: t.last_sign,
        prior: t.attachment_prior,
        relation: t.relation.as_ref(),
    })?;
    require(
        t.group_count.0 > 0 || (t.magnitude_sum == 0.0 && t.magnitude_weight == 0.0),
        "magnitude sums before the first group",
    )?;
    require(
        t.magnitude_weight != 0.0 || t.typical_magnitude == 0.0,
        "typical magnitude without magnitude weight",
    )
}

/// The allocations of one tendency across R's evidence groups carry the ordinals
/// 0, 1, …, N_k − 1, each exactly once (ADR 0016): the order G replays them in.
pub fn check_allocation_ordinals(t: &TendencyV2, groups: &[EvidenceGroupV2]) -> Result<()> {
    let mut ordinals: Vec<u64> = groups
        .iter()
        .flat_map(|g| g.allocations.iter())
        .filter(|a| a.tendency_id == t.tendency_id)
        .map(|a| a.ordinal.0)
        .collect();
    ordinals.sort_unstable();
    require(
        u64::try_from(ordinals.len()).is_ok_and(|n| n == t.group_count.0),
        "allocation count differs from the group count",
    )?;
    require(
        ordinals.iter().zip(0u64..).all(|(o, i)| *o == i),
        "allocation ordinals are not 0..N-1 exactly once",
    )
}

/// P5-34 / ADR 0021: q is the historical multiplier, never inferred from current support.
pub fn check_allocation_v3(a: &EvidenceAllocationV3) -> Result<()> {
    require(
        a.actual_retention.is_finite() && a.actual_retention > 0.0 && a.actual_retention <= 1.0,
        "actual retention must lie in (0, 1]",
    )
}

/// Local V3 invariants. Context-dependent B/pi/r consistency is checked separately
/// with the bound law parameters, never by recovering c from rounded B-V.
pub fn check_tendency_v3(t: &TendencyV3) -> Result<()> {
    require(
        [
            t.evidence_a,
            t.evidence_b,
            t.evidence_absolute,
            t.h_hat,
            t.pi,
            t.typical_magnitude,
            t.magnitude_sum,
            t.magnitude_weight,
        ]
        .iter()
        .all(|v| v.is_finite())
            && t.attachment_prior.is_none_or(|p| p.is_finite() && p >= 0.0),
        "nonfinite or invalid tendency scalar",
    )?;
    check_counts(&Counts {
        direction: &t.direction,
        pi: t.pi,
        a: t.evidence_a,
        b: t.evidence_b,
        groups: t.group_count.0,
        flips: t.flip_count.0,
        last_sign: t.last_sign,
        prior: t.attachment_prior,
        relation: t.relation.as_ref(),
    })?;
    require(
        t.evidence_absolute >= t.evidence_a.abs(),
        "V must not be below |A|",
    )?;
    require((0.0..1.0).contains(&t.h_hat), "h must lie in [0, 1)")?;
    require(
        t.group_count.0 < (1u64 << 53) && t.flip_count.0 < (1u64 << 53),
        "count precision exhausted",
    )?;
    require(
        t.magnitude_weight >= 0.0 && t.typical_magnitude >= 0.0,
        "negative magnitude weight or magnitude",
    )?;
    require(
        t.group_count.0 > 0 || (t.magnitude_sum == 0.0 && t.magnitude_weight == 0.0),
        "magnitude sums before the first group",
    )?;
    require(
        t.magnitude_weight != 0.0 || t.typical_magnitude == 0.0,
        "typical magnitude without magnitude weight",
    )
}

/// ADR 0019 §8: exact binary64 grouping; P+B is not a substitute for (P+c)+V.
/// The caller must bind p to the tendency's law resource before calling this pure check.
pub fn check_tendency_v3_with_parameters(t: &TendencyV3, p: &LawParameters) -> Result<()> {
    check_tendency_v3(t)?;
    check_law_parameters(p)?;
    require(
        p.kappa1.is_finite() && p.kappa1 >= 0.0 && p.r_max.is_finite() && p.r_max >= 0.0,
        "invalid contextual law parameter",
    )?;
    let conditions = u32::try_from(t.predicate.len()).map_err(|_| "predicate count overflow")?;
    let kappa = p.kappa1 * f64::from(conditions);
    let c = p.n0 + kappa;
    let prior = t.attachment_prior.unwrap_or(0.0);
    let anchor = prior + c;
    let denominator = anchor + t.evidence_absolute;
    let numerator = prior + t.evidence_a;
    let b = c + t.evidence_absolute;
    require(
        [kappa, c, anchor, denominator, numerator, b]
            .iter()
            .all(|v| v.is_finite())
            && c > 0.0,
        "nonfinite contextual evidence",
    )?;
    require(t.evidence_b == b, "B differs from c+V")?;
    require(
        t.pi == numerator / denominator,
        "pi differs from (P+A)/((P+c)+V)",
    )?;
    let r = if t.magnitude_weight == 0.0 {
        0.0
    } else {
        let mean = t.magnitude_sum / t.magnitude_weight;
        require(mean.is_finite(), "nonfinite magnitude mean")?;
        mean.clamp(0.0, p.r_max)
    };
    require(
        t.typical_magnitude == r,
        "typical magnitude differs from clipped M/U",
    )
}

/// V3 replay order is still exactly 0..N-1, retaining P5-32's check for old bytes.
pub fn check_allocation_ordinals_v3(t: &TendencyV3, groups: &[EvidenceGroupV3]) -> Result<()> {
    let mut ordinals = Vec::new();
    for a in groups
        .iter()
        .flat_map(|g| &g.allocations)
        .filter(|a| a.tendency_id == t.tendency_id)
    {
        check_allocation_v3(a)?;
        ordinals.push(a.ordinal.0);
    }
    ordinals.sort_unstable();
    require(
        u64::try_from(ordinals.len()).is_ok_and(|n| n == t.group_count.0),
        "allocation count differs from the group count",
    )?;
    require(
        ordinals.iter().zip(0u64..).all(|(o, i)| *o == i),
        "allocation ordinals are not 0..N-1 exactly once",
    )
}

/// A ledger R change names a tendency entering, changing or leaving R.
pub fn check_tendency_change(c: &TendencyChange) -> Result<()> {
    require(
        c.pi_before.is_some() || c.pi_after.is_some(),
        "change needs a before or an after value",
    )?;
    for pi in [c.pi_before, c.pi_after].into_iter().flatten() {
        require(pi > -1.0 && pi < 1.0, "pi must lie in (-1, 1)")?;
    }
    Ok(())
}

/// θ statistics: λ ≥ λ_min > 0 (v2.0 §6, §8); σ̂ and τ̂ are divisors in §8, so positive.
pub fn check_theta(t: &ThetaSnapshot) -> Result<()> {
    let positive = |v: &NatureScale| scale(v).iter().all(|x| *x > 0.0);
    require(
        positive(&t.values.recovery_per_s),
        "recovery must be positive",
    )?;
    require(positive(&t.sigma), "sigma must be positive")?;
    require(positive(&t.tau_s), "tau must be positive")
}

/// θ inside the law's ranges: g ∈ [g_min, g_max], λ ∈ [λ_min, λ_max] (v2.0 §8).
pub fn check_theta_within(t: &ThetaValues, p: &LawParameters) -> Result<()> {
    require(
        scale(&t.gain)
            .iter()
            .all(|g| (p.g_min..=p.g_max).contains(g)),
        "gain outside [g_min, g_max]",
    )?;
    require(
        scale(&t.recovery_per_s)
            .iter()
            .all(|l| (p.recovery_min_per_s..=p.recovery_max_per_s).contains(l)),
        "recovery outside [lambda_min, lambda_max]",
    )
}

/// Inequalities v2.0 states between law parameters.
pub fn check_law_parameters(p: &LawParameters) -> Result<()> {
    // n₀ > 0 keeps π inside the open interval (v2.0 §1, §5.2).
    require(p.n0 > 0.0, "n0 must be positive")?;
    require(p.w_teach <= p.w_max, "w_teach must not exceed w_max")?; // v2.0 §5.2
    require(p.v_u < 1.0, "v_u must be below 1")?; // v2.0 §5.2
    require(p.r_min <= p.r_max, "r_min must not exceed r_max")?;
    require(p.g_min <= p.g_max, "g_min must not exceed g_max")?; // v2.0 §8
    require(p.recovery_min_per_s > 0.0, "lambda_min must be positive")?; // v2.0 §6, §8
    require(
        p.recovery_min_per_s <= p.recovery_max_per_s,
        "lambda_min must not exceed lambda_max",
    )
}

/// Genesis values consistent with its own law version (v2.0 §3, §8, §13).
pub fn check_genesis(g: &GenesisManifestV2) -> Result<()> {
    let p = &g.law_version.parameters;
    check_law_parameters(p)?;
    let gain = scale(&g.initial_gain);
    require(
        gain.iter().all(|x| (p.g_min..=p.g_max).contains(x)),
        "initial gain outside [g_min, g_max]",
    )?;
    // v2.0 §3: achievement starts with the weakest gain of the four.
    require(
        gain[3] < gain[0] && gain[3] < gain[1] && gain[3] < gain[2],
        "achievement must have the weakest initial gain",
    )?;
    require(g.sigma0 > 0.0, "sigma0 must be positive")?;
    require(g.tau0_s > 0.0, "tau0 must be positive")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vector(c: f64, e: f64, q: f64, a: f64) -> NatureVector {
        NatureVector {
            schema_version: 1,
            connection: c,
            empathy: e,
            curiosity: q,
            achievement: a,
        }
    }

    #[test]
    fn unit_directions_pass_and_others_fail() {
        let h = std::f64::consts::FRAC_1_SQRT_2;
        assert!(check_direction(&vector(1.0, 0.0, 0.0, 0.0)).is_ok());
        assert!(check_direction(&vector(h, 0.0, -h, 0.0)).is_ok());
        assert!(check_direction(&vector(0.5, 0.5, 0.5, 0.5)).is_ok());
        assert!(check_direction(&vector(0.0, 0.0, 0.0, 0.0)).is_err());
        assert!(check_direction(&vector(1.0, 1.0, 0.0, 0.0)).is_err());
        assert!(check_direction(&vector(0.6, 0.6, 0.0, 0.0)).is_err());
        assert!(check_direction(&vector(f64::NAN, 0.0, 0.0, 0.0)).is_err());
    }

    #[test]
    fn tendency_order_is_canonical_set_order() {
        // Shorter keys sort first by their little-endian length bytes.
        assert_eq!(tendency_order("b", "aa"), Ordering::Less);
        assert_eq!(tendency_order("a", "b"), Ordering::Less);
        assert_eq!(tendency_order("t", "t"), Ordering::Equal);
    }
}
