//! P5-36: explicit semantic boundary, in addition to the structural wire codec.
use crate::contract::{Limits, Result, Schema, require};
use crate::generated::{ThetaEstimatorParameters, ThetaSnapshotV2};
use crate::scalar::{HexBytes, unhex};
use serde_json::Value;

pub const THETA_NUMERICS_ID: &str = "P5-25-v2-93-98-libm-0.2.16";
/// CanonicalV1 domain-separated digest of the complete C resource view.
pub fn theta_parameters_digest(
    parameters: &ThetaEstimatorParameters,
    limits: Limits,
) -> Result<HexBytes> {
    let value = serde_json::to_value(parameters).map_err(|e| e.to_string())?;
    check_theta_message("ThetaEstimatorParameters", &value, limits)?;
    Ok(HexBytes(unhex(&Schema::builtin()?.digest(
        "ThetaEstimatorParameters",
        &value,
        limits,
    )?)?))
}
const EXACT: u64 = 1_u64 << 53;

fn number(v: &Value, key: &str) -> f64 {
    v[key].as_f64().expect("normalized double")
}
fn integer(v: &Value) -> u64 {
    v.as_str()
        .expect("normalized uint64")
        .parse()
        .expect("uint64")
}
fn signed(v: &Value) -> i64 {
    v.as_str()
        .expect("normalized sint64")
        .parse()
        .expect("sint64")
}
fn array<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().expect("normalized array")
}
fn scale(v: &Value) -> [f64; 4] {
    ["connection", "empathy", "curiosity", "achievement"].map(|k| number(v, k))
}
fn parameters(p: &Value) -> Result<()> {
    require(
        p["numerics_id"] == THETA_NUMERICS_ID,
        "theta numerics version",
    )?;
    for key in [
        "baseline_weight",
        "variance_weight",
        "duration_weight",
        "change_prior_weight",
        "gain_min",
        "gain_max",
        "recovery_min",
        "recovery_max",
        "detector_shift_sigma",
        "detector_threshold",
    ] {
        require(
            number(p, key) > 0.0 && number(p, key) < EXACT as f64,
            "theta positive exact parameter",
        )?;
    }
    for key in ["change_prior", "change_min"] {
        let h = number(p, key);
        require(
            h > 0.0 && h < 1.0 && 1.0 - h != 1.0,
            "theta representable retention",
        )?;
    }
    for key in ["gain_prior", "sigma_prior", "tau_prior_s"] {
        require(
            scale(&p[key]).iter().all(|x| *x > 0.0),
            "theta positive prior",
        )?;
    }
    require(
        number(p, "gain_min") <= number(p, "gain_max")
            && number(p, "recovery_min") <= number(p, "recovery_max"),
        "theta ordered bounds",
    )?;
    require(
        scale(&p["gain_prior"])
            .iter()
            .all(|x| *x >= number(p, "gain_min") && *x <= number(p, "gain_max")),
        "theta gain prior bounds",
    )?;
    require(
        scale(&p["sigma_prior"])
            .iter()
            .all(|x| (x * x).is_finite() && x * x > 0.0),
        "theta prior variance",
    )?;
    require(
        number(p, "jump_notice") > 0.0
            && integer(&p["drift_run"]) > 0
            && integer(&p["drift_run"]) < EXACT,
        "theta calibration",
    )
}
fn running(t: &Value) -> Result<()> {
    let elapsed = integer(&t["event_ns"]).checked_sub(integer(&t["origin_ns"]));
    require(
        elapsed.and_then(|x| x.checked_sub(integer(&t["stopped_ns"])))
            == Some(integer(&t["running_ns"])),
        "theta running time identity",
    )
}
fn state(s: &Value) -> Result<()> {
    for key in [
        "reaction_sum",
        "square_sum",
        "variance",
        "duration_log_sum",
        "duration_mass",
        "duration_log_mean",
        "zero_duration_count",
        "run_sign",
        "run_start_ns",
        "detector_mean",
        "detector_positive",
        "detector_negative",
    ] {
        require(array(s, key).len() == 4, "theta four channels")?;
    }
    for key in ["drift_sign", "drift_count"] {
        require(array(s, key).len() == 12, "theta twelve components")?;
    }
    let n = integer(&s["event_count"]);
    require(n < EXACT, "theta exact event count")?;
    let h = number(s, "h");
    require(
        h > 0.0 && h < 1.0 && 1.0 - h != 1.0,
        "theta representable h",
    )?;
    require(
        scale(&s["sigma"])
            .iter()
            .chain(scale(&s["tau_s"]).iter())
            .all(|v| *v > 0.0)
            && array(s, "variance")
                .iter()
                .all(|v| v.as_f64().expect("double") > 0.0),
        "theta positive estimates",
    )?;
    for key in ["detector_segment_count", "detector_changes"] {
        require(integer(&s[key]) <= n, "theta detector count")?;
    }
    for key in ["zero_duration_count", "drift_count"] {
        require(
            array(s, key).iter().all(|v| integer(v) <= n),
            "theta count exceeds events",
        )?;
    }
    let has_event = s.get("last_event").is_some();
    require(
        has_event == (n > 0) && s.get("last_running_time").is_some() == has_event,
        "theta last event presence",
    )?;
    if has_event {
        running(&s["last_running_time"])?;
        require(
            s["last_event"]["time_ns"] == s["last_running_time"]["event_ns"],
            "theta last time mismatch",
        )?;
        require(
            integer(&s["detector_segment_count"]) > 0 && number(s, "weight") > 0.0,
            "theta nonempty statistics",
        )?;
    } else {
        require(number(s, "weight") == 0.0, "theta genesis weight")?;
        for key in [
            "reaction_sum",
            "square_sum",
            "duration_log_sum",
            "duration_mass",
            "detector_mean",
            "detector_positive",
            "detector_negative",
        ] {
            require(
                array(s, key).iter().all(|v| v.as_f64() == Some(0.0)),
                "theta genesis statistics",
            )?;
        }
    }
    let now = if has_event {
        integer(&s["last_running_time"]["running_ns"])
    } else {
        0
    };
    for i in 0..4 {
        let sign = signed(&array(s, "run_sign")[i]);
        let start = integer(&array(s, "run_start_ns")[i]);
        require(
            (-1..=1).contains(&sign)
                && start <= now
                && (sign != 0 || start == 0)
                && (has_event || sign == 0),
            "theta open run",
        )?;
        require(
            array(s, "duration_mass")[i].as_f64() != Some(0.0)
                || array(s, "duration_log_sum")[i].as_f64() == Some(0.0),
            "theta empty duration sum",
        )?;
    }
    for i in 0..12 {
        let sign = signed(&array(s, "drift_sign")[i]);
        let count = integer(&array(s, "drift_count")[i]);
        require(
            (-1..=1).contains(&sign) && (sign == 0) == (count == 0),
            "theta drift sign/count",
        )?;
    }
    Ok(())
}
/// Call after decoding and before treating a message as estimator state. The
/// structural codec is deliberately separate, as for other semantic validators.
/// Even a valid snapshot still needs a trusted caller's resource/H-prefix binding.
/// Use the same caller-owned limits as the structural codec, including digesting.
pub fn check_theta_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "ThetaEstimatorParameters"
            | "ThetaRunningTime"
            | "ThetaLastEvent"
            | "ThetaEstimatorState"
            | "ThetaSnapshotV2"
    ) {
        return Ok(());
    }
    let schema = Schema::builtin()?;
    let v = schema.normalize(name, value, limits, 0)?;
    match name {
        "ThetaEstimatorParameters" => parameters(&v),
        "ThetaRunningTime" => running(&v),
        "ThetaLastEvent" => Ok(()),
        "ThetaEstimatorState" => state(&v),
        _ => {
            let p = &v["parameters"];
            let s = &v["estimator"];
            parameters(p)?;
            state(s)?;
            require(
                v["law_version"]["object_type"] == "echo.law-version"
                    && v["law_version"]["resource_version"] == p["law_id"],
                "theta law binding",
            )?;
            require(
                v["parameters_digest"].as_str()
                    == Some(
                        schema
                            .digest("ThetaEstimatorParameters", p, limits)?
                            .as_str(),
                    ),
                "theta parameter digest",
            )?;
            require(number(s, "h") >= number(p, "change_min"), "theta h floor")?;
            if integer(&s["event_count"]) == 0 {
                require(
                    number(s, "h").to_bits()
                        == number(p, "change_prior")
                            .max(number(p, "change_min"))
                            .to_bits(),
                    "theta initial h",
                )?;
            }
            for (field, lo, hi) in [
                ("gain", "gain_min", "gain_max"),
                ("recovery_per_s", "recovery_min", "recovery_max"),
            ] {
                require(
                    scale(&v["values"][field])
                        .iter()
                        .all(|x| *x >= number(p, lo) && *x <= number(p, hi)),
                    "theta value bounds",
                )?;
            }
            Ok(())
        }
    }
}
pub fn check_theta_snapshot_v2(snapshot: &ThetaSnapshotV2, limits: Limits) -> Result<()> {
    check_theta_message(
        "ThetaSnapshotV2",
        &serde_json::to_value(snapshot).map_err(|e| e.to_string())?,
        limits,
    )
}
