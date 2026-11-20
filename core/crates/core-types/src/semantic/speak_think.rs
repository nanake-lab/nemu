//! ADR 0046. Check decoded facts before consuming them at a trusted boundary.
use super::check_cognition_message;
use crate::contract::{Limits, Result, Schema, require};
use crate::generated::Utf8Range;
use serde_json::Value;
use std::collections::BTreeSet;

fn number(v: &Value, key: &str) -> u64 {
    v[key]
        .as_str()
        .expect("normalized uint64")
        .parse()
        .expect("uint64")
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().expect("normalized vector")
}

fn interval(v: &Value, length: u64) -> Result<(u64, u64)> {
    let (start, end) = (number(v, "start"), number(v, "end"));
    require(
        start < end && end <= length,
        "empty or out-of-bounds UTF-8 range",
    )?;
    Ok((start, end))
}

/// Apply a checked range to the bytes retrieved for its immutable content reference.
/// Byte bounds alone cannot detect the middle of a UTF-8 scalar.
pub fn resolved_text_range<'a>(text: &'a str, range: &Utf8Range) -> Result<&'a str> {
    require(
        range.r#schema_version == 1 && range.r#start.0 < range.r#end.0,
        "invalid text range",
    )?;
    let start = usize::try_from(range.r#start.0).map_err(|e| e.to_string())?;
    let end = usize::try_from(range.r#end.0).map_err(|e| e.to_string())?;
    text.get(start..end)
        .ok_or_else(|| "invalid UTF-8 slice".into())
}

fn remote(v: &Value) -> Result<()> {
    for key in ["model_sha256", "tokenizer_sha256", "request_sha256"] {
        if let Some(hex) = v[key].as_str() {
            require(
                hex.bytes().any(|b| b != b'0'),
                "unavailable digest encoded as zero",
            )?;
        }
    }
    let sent = v["status"] != 6;
    if sent {
        require(
            number(v, "output_reserve") <= number(v, "capacity"),
            "output reserve exceeds capacity",
        )?;
    }
    let has = |k: &str| v.get(k).is_some();
    require(
        !has("returned_text") || has("raw_response"),
        "text without response",
    )?;
    require(
        !has("returned_model") || has("raw_response"),
        "model without response",
    )?;
    let adoption = v["adoption"].as_u64().expect("normalized enum");
    let accepted = matches!(adoption, 1 | 2);
    require(
        has("product") == accepted && has("accepted_range") == accepted,
        "adoption payload",
    )?;
    require(adoption == 5 || has("review"), "missing adoption decision")?;
    let length = if has("returned_text") {
        number(&v["returned_text"], "byte_length")
    } else {
        0
    };
    if accepted {
        let (start, end) = interval(&v["accepted_range"], length)?;
        require(
            adoption != 1 || (start == 0 && end == length),
            "incomplete full adoption",
        )?;
        let product = &v["product"]["content"];
        require(
            number(product, "byte_length") == end - start,
            "product length",
        )?;
        require(
            start != 0 || end != length || product == &v["returned_text"],
            "full text identity",
        )?;
    }
    require(adoption != 5 || length == 0, "unaccounted returned text")
}

fn check(schema: &Schema, name: &str, v: &Value, limits: Limits) -> Result<()> {
    match name {
        "Utf8Range" => {
            interval(v, u64::MAX)?;
        }
        "QuoteLayer" => {
            interval(
                &v["visible"],
                number(&v["source"]["content"], "byte_length"),
            )?;
        }
        "SpeakerAttribution" => {
            let layers = items(v, "layers");
            require(!layers.is_empty(), "missing speaker layers")?;
            for layer in layers {
                require(
                    layer["source"]["branch_id"] == layers[0]["source"]["branch_id"],
                    "quote branch",
                )?;
                check(schema, "QuoteLayer", layer, limits)?;
            }
        }
        "RecordRelation" => require(
            v["origin"]["branch_id"] == v["target"]["branch_id"],
            "relation branch",
        )?,
        "RelationSet" => {
            let mut seen = BTreeSet::new();
            for edge in items(v, "relations") {
                check(schema, "RecordRelation", edge, limits)?;
                require(
                    edge["purpose"] == v["purpose"]
                        && (edge["origin"] == v["endpoint"] || edge["target"] == v["endpoint"]),
                    "relation query scope",
                )?;
                require(
                    seen.insert(schema.encode_canonical("RecordRelation", edge, limits)?),
                    "duplicate relation",
                )?;
            }
        }
        "RelationQueryReceipt" => {
            let set = &v["result"];
            check(schema, "RelationSet", set, limits)?;
            require(
                number(v, "expected_count") == items(set, "relations").len() as u64,
                "incomplete relation count",
            )?;
            let manifest = &v["manifest"];
            require(
                manifest["object_type"] == "echo.relation-set"
                    && manifest["digest"] == schema.digest("RelationSet", set, limits)?
                    && number(manifest, "byte_length")
                        == schema.encode_canonical("RelationSet", set, limits)?.len() as u64,
                "relation manifest",
            )?;
        }
        "RemoteCognitiveReceipt" => remote(v)?,
        "InternalActivityRecordV2" => {
            require(
                number(v, "finished_at_ns") >= number(v, "started_at_ns"),
                "execution interval",
            )?;
            if let Some(prev) = v.get("previous") {
                require(
                    matches!(
                        prev["object_type"].as_str(),
                        Some("echo.internal-activity-record" | "echo.internal-activity-record-v2")
                    ),
                    "previous record type",
                )?;
            }
            if let Some(local) = v.get("local") {
                check_cognition_message("CognitiveResultAndReceipt", local, limits)?;
            } else {
                remote(&v["remote"])?;
            }
        }
        "InternalActivityCommitV2" => {
            let activity = &v["activity"];
            check(schema, "InternalActivityRecordV2", activity, limits)?;
            let result = activity.get("local").unwrap_or(&activity["remote"]);
            require(
                result["binding"] == v["binding"] && !items(v, "interpretations").is_empty(),
                "commit binding or interpretations",
            )?;
        }
        "CoreTransactionV4" => {
            if let Some(commit) = v.get("activity_commit") {
                check(schema, "InternalActivityCommitV2", commit, limits)?;
            }
        }
        "SpeechDeliveryV2" => {
            let length = number(&v["presentation"]["content"], "byte_length");
            let ranges = items(v, "delivered");
            let mut previous = 0;
            for (i, range) in ranges.iter().enumerate() {
                let (start, end) = interval(range, length)?;
                require(
                    i == 0 || previous < start,
                    "delivery ranges overlap or touch",
                )?;
                previous = end;
            }
            let full = ranges.len() == 1 && number(&ranges[0], "start") == 0 && previous == length;
            match v["status"].as_u64() {
                Some(1) => require(
                    full || (length == 0 && ranges.is_empty()),
                    "full delivery range",
                )?,
                Some(2) => require(!ranges.is_empty() && !full, "partial delivery range")?,
                Some(4) => require(ranges.is_empty(), "unknown delivery claims text")?,
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn check_speak_think_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "Utf8Range"
            | "RecordVersion"
            | "QuoteLayer"
            | "SpeakerAttribution"
            | "RecordRelation"
            | "RelationSet"
            | "RelationQueryReceipt"
            | "RemoteCognitiveReceipt"
            | "InternalActivityRecordV2"
            | "InternalActivityCommitV2"
            | "CoreTransactionV4"
            | "SpeechDeliveryV2"
    ) {
        return Ok(());
    }
    let schema = Schema::builtin()?;
    let v = schema.normalize(name, value, limits, 0)?;
    check(&schema, name, &v, limits)
}
