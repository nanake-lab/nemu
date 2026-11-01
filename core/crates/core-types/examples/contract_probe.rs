//! Test-only JSON-lines adapter. Never opens authority storage.
mod ingress_probe;
use core_types::contract::{Limits, Schema};
use core_types::scalar::{hex, unhex};
use serde_json::{Value, json};
use std::io::{self, BufRead};

fn probe(schema: &Schema, request: &Value) -> Result<Value, String> {
    let l = request.get("limits").ok_or("limits")?;
    let limit = |name: &str| -> Result<usize, String> {
        usize::try_from(l[name].as_u64().ok_or("limit")?).map_err(|e| e.to_string())
    };
    let limits = Limits {
        max_bytes: limit("max_bytes")?,
        max_items: limit("max_items")?,
        max_depth: limit("max_depth")?,
    };
    if request["op"] == "ingress" {
        return ingress_probe::check(request, limits);
    }
    let name = request["name"].as_str().ok_or("name")?;
    let value = match request["op"].as_str().ok_or("op")? {
        "encode" => schema.normalize(name, &request["value"], limits, 0)?,
        "canonical" => schema.decode_canonical(
            name,
            &unhex(request["hex"].as_str().ok_or("hex")?)?,
            limits,
            0,
        )?,
        "protobuf" => schema.decode_protobuf(
            name,
            &unhex(request["hex"].as_str().ok_or("hex")?)?,
            limits,
            0,
        )?,
        _ => return Err("operation".into()),
    };
    core_types::generated::verify_typed(name, &value, limits)?;
    core_types::semantic::check_theta_message(name, &value, limits)?;
    Ok(
        json!({"ok":true,"value":value,"canonical":hex(&schema.encode_canonical(name,&value,limits)?),
        "protobuf":hex(&schema.encode_protobuf(name,&value,limits)?),"digest":schema.digest(name,&value,limits)?}),
    )
}
fn main() {
    let schema = Schema::builtin().expect("built-in descriptor");
    for line in io::stdin().lock().lines() {
        let result = line
            .map_err(|e| e.to_string())
            .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
            .and_then(|v| probe(&schema, &v));
        println!(
            "{}",
            result.unwrap_or_else(|error| json!({"ok":false,"error":error}))
        );
    }
}
