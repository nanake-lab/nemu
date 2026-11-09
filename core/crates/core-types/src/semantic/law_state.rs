//! ADR 0035 §4 / P5-68. Explicit boundary; H/resource resolution belongs to P5-75.
use crate::contract::{Limits, Result, Schema, require};
use serde_json::Value;

fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().expect("normalized list")
}
fn aligned(a: &Value, b: &Value) -> bool {
    list(a, "learned")
        .iter()
        .map(|c| &c["tendency_id"])
        .eq(list(b, "learned").iter().map(|c| &c["tendency_id"]))
}
fn slot(v: &Value) -> Result<()> {
    require(aligned(&v["current"], &v["target"]), "slot coordinates")
}

/// Call before consuming a new fast-state message. Does not compare h to the
/// current target: ADR 0035 §4.6 B6' explicitly permits revised/merged states.
pub fn check_law_state_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "FastBasisIdentity"
            | "LawLearnedCoordinate"
            | "LawStateVector"
            | "FastBasisSlot"
            | "FastStateSnapshotV3"
            | "FastEffectRecord"
            | "FastStateMigrationRecord"
    ) {
        return Ok(());
    }
    let v = Schema::builtin()?.normalize(name, value, limits, 0)?;
    match name {
        "FastBasisSlot" => slot(&v)?,
        "FastStateSnapshotV3" => {
            for s in list(&v, "slots") {
                slot(s)?;
                require(aligned(&s["current"], &v), "snapshot coordinates")?;
            }
            require(
                v["law_version"]["object_type"] == "echo.law-version",
                "snapshot law reference",
            )?;
        }
        "FastEffectRecord" => {
            require(
                !list(&v, "interpretation_refs").is_empty()
                    && !list(&v, "governance_refs").is_empty(),
                "effect references",
            )?;
            require(
                v["theta_before"]["object_type"] == "echo.theta-snapshot-v3",
                "effect theta reference",
            )?;
            require(
                v["law_version"]["object_type"] == "echo.law-version",
                "effect law reference",
            )?;
            require(v["role"] != 4 || v["basis"]["kind"] == 2, "internal role")?;
            require(v["role"] != 1 || v["basis"]["kind"] == 1, "new basis role")?;
            // ADR 0035 §4.2–4.3: internal targets cannot be revisited.
            // §5.2 still permits a target-update application for this basis.
            require(
                v["basis"]["kind"] != 2 || v["role"] != 3,
                "internal revisit",
            )?;
        }
        "FastStateMigrationRecord" => {
            require(
                v["prior_decomposition_unknown"] == true,
                "unknown prior decomposition",
            )?;
            require(
                v["previous_snapshot"]["object_type"] == "echo.fast-state-v2",
                "migration predecessor",
            )?;
            require(
                v["migrated_snapshot"]["object_type"] == "echo.fast-state-v3",
                "migration successor",
            )?;
        }
        _ => {}
    }
    Ok(())
}
