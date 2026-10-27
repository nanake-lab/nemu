use core_types::{contract::Limits, terminal_ingress as c};
use serde::de::DeserializeOwned;
use serde_json::Value;
fn field<T: DeserializeOwned>(q: &Value, key: &str) -> Result<T, String> {
    serde_json::from_value(q[key].clone()).map_err(|_| "invalid ingress value".into())
}
pub fn check(q: &Value, l: Limits) -> Result<Value, String> {
    match q["check"].as_str().ok_or("ingress check")? {
        "auth" => c::check_auth_pair(&field(q, "request")?, &field(q, "response")?, l)?,
        "request" => c::check_request(
            &field(q, "request")?,
            field::<Option<core_types::generated::TerminalSession>>(q, "session")?.as_ref(),
            l,
        )?,
        "response" => c::check_response(
            &field(q, "request")?,
            &field(q, "response")?,
            &field(q, "session")?,
            l,
        )?,
        "window" => c::check_upload_window(
            &field(q, "chunk")?,
            &field(q, "budget")?,
            &c::UploadWindow {
                upload_id: q
                    .get("declared_id")
                    .unwrap_or(&q["chunk"]["upload_id"])
                    .as_str()
                    .ok_or("upload id")?
                    .to_owned(),
                object: serde_json::from_value(
                    q.get("declared_object")
                        .unwrap_or(&q["chunk"]["object"])
                        .clone(),
                )
                .map_err(|_| "declared object")?,
                next_offset: q["next_offset"].as_u64().ok_or("offset")?,
                active_uploads: u32::try_from(q["active"].as_u64().ok_or("active")?)
                    .map_err(|_| "active overflow")?,
                reserved_bytes: q["reserved"].as_u64().ok_or("reserved")?,
            },
            l,
        )?,
        "chunk" => c::check_chunk(&field(q, "chunk")?, &field(q, "budget")?, l)?,
        "object" => c::check_uploaded_object(
            &field(q, "reference")?,
            &core_types::scalar::unhex(q["hex"].as_str().ok_or("hex")?)?,
            &field(q, "budget")?,
            l,
        )?,
        _ => return Err("unknown ingress check".into()),
    }
    Ok(serde_json::json!({"ok":true}))
}
