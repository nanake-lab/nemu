//! ADR 0030. Explicit boundary after structural decoding; no authority, clocks,
//! source resolution, real carrier verification or production Q comparison.
use crate::contract::{Limits, Result, Schema, require};
use serde_json::Value;

fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().expect("normalized list")
}
fn has(v: &Value, key: &str) -> bool {
    v.get(key).is_some()
}
fn tag(v: &Value, key: &str, expected: &str) -> Result<()> {
    require(
        !has(v, key) || v[key]["object_type"] == expected,
        "cognition reference type",
    )
}
fn number(v: &Value, key: &str) -> u64 {
    v[key]
        .as_str()
        .expect("normalized uint64")
        .parse()
        .expect("normalized uint64")
}

pub fn check_cognition_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "CognitionBinding"
            | "CognitiveMaterial"
            | "CognitiveReturnedResult"
            | "CognitiveInput"
            | "CognitiveResultAndReceipt"
            | "InternalActivityRecord"
            | "ActivityState"
            | "CognitiveInflightAction"
            | "CognitionState"
            | "InternalActivityCommit"
            | "CoreTransactionV3"
            | "SystemMaintenanceRecord"
    ) {
        return Ok(());
    }
    let schema = Schema::builtin()?;
    let v = schema.normalize(name, value, limits, 0)?;
    match name {
        "CognitiveReturnedResult" => {
            require(
                v["status"] != 2 || has(&v, "result"),
                "successful result missing",
            )?;
            require(v["status"] != 1 || !has(&v, "result"), "waiting has result")?;
        }
        "CognitiveInput" => {
            for item in list(&v, "results") {
                check_cognition_message("CognitiveReturnedResult", item, limits)?;
            }
        }
        "CognitiveResultAndReceipt" => {
            tag(&v, "input", "echo.cognitive-input")?;
            tag(&v, "modulation", "echo.modulation-v2")?;
            let b = &v["binding"];
            require(
                v["model_hash"] == b["environment"]["model"]["digest"],
                "model binding",
            )?;
            require(
                v["tokenizer_hash"] == b["environment"]["tokenizer"]["digest"],
                "tokenizer binding",
            )?;
            require(
                !(v["completion"] == 1 || v["completion"] == 2) || has(&v, "carrier_receipt"),
                "carrier receipt missing",
            )?;
            if v["completion"] != 1 {
                require(has(&v, "reason"), "incomplete reason missing")?;
                require(
                    !has(&v, "next_focus")
                        && !has(&v, "action_plan")
                        && list(&v, "queries").is_empty(),
                    "incomplete successor",
                )?;
            }
            if let Some(p) = v.get("action_plan") {
                require(p["snapshot_id"] == b["snapshot_id"], "action snapshot")?;
                require(p["task_version"] == b["q_revision"], "action Q revision")?;
                require(
                    p["protocol_version"] == b["protocol"]["resource_version"],
                    "action protocol",
                )?;
            }
        }
        "InternalActivityRecord" => {
            tag(&v, "previous", "echo.internal-activity-record")?;
            require(
                number(&v, "finished_at_ns") >= number(&v, "started_at_ns"),
                "request execution interval",
            )?;
            check_cognition_message("CognitiveResultAndReceipt", &v["result"], limits)?;
        }
        "ActivityState" => {
            tag(&v, "latest_step", "echo.internal-activity-record")?;
            tag(&v, "successor", "echo.cognitive-input")?;
        }
        "CognitiveInflightAction" => tag(&v, "origin_record", "echo.internal-activity-record")?,
        "CognitionState" => {
            tag(&v, "active_request", "echo.cognitive-input")?;
            tag(&v, "runnable_successor", "echo.cognitive-input")?;
            tag(&v, "transition", "echo.system-maintenance-record")?;
            let count =
                u8::from(has(&v, "active_request")) + u8::from(has(&v, "runnable_successor"));
            require(
                if v["mode"] == 1 {
                    count == 1
                } else {
                    count == 0 && has(&v, "reason")
                },
                "global continuation",
            )?;
            for item in list(&v, "activities") {
                check_cognition_message("ActivityState", item, limits)?;
            }
            for item in list(&v, "inflight") {
                check_cognition_message("CognitiveInflightAction", item, limits)?;
            }
        }
        "InternalActivityCommit" => {
            check_cognition_message("InternalActivityRecord", &v["record"], limits)?;
            require(
                !list(&v, "interpretations").is_empty(),
                "accepted interpretation outcomes missing",
            )?;
            require(
                schema.encode_canonical("CognitionBinding", &v["binding"], limits)?
                    == schema.encode_canonical(
                        "CognitionBinding",
                        &v["record"]["result"]["binding"],
                        limits,
                    )?,
                "internal binding",
            )?;
        }
        "CoreTransactionV3" => {
            if let Some(c) = v.get("internal_commit") {
                check_cognition_message("InternalActivityCommit", c, limits)?;
            }
        }
        _ => {}
    }
    Ok(())
}
