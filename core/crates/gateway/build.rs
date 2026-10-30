//! Dispatch only to IDL-generated Rust messages; this generates no wire types.
fn main() {
    let source = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../contracts/generated/schema.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let schema: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source).unwrap()).unwrap();
    let mut code = String::from(
        "fn prepare<L: core_store::LineageKind>(store: &core_store::Store<L>, name: &str, bytes: &[u8], version: &str, limits: core_types::contract::Limits) -> crate::Result<core_store::PreparedObject<L>> { use core_types::WireMessage; match name {\n",
    );
    for (name, message) in schema["messages"].as_object().unwrap() {
        if message["transient"].as_bool() == Some(true)
            || !message["superseded"].is_null()
            || message["file"].as_str() == Some("terminal_ingress.proto")
        {
            continue;
        }
        let call = if name == "RSnapshotV3" {
            "store.prepare_r(&value)"
        } else {
            "store.prepare(&value, version)"
        };
        code.push_str(&format!("\"{name}\" => {{ let value = core_types::generated::{name}::from_canonical(bytes, limits).map_err(|_| crate::Error::Contract)?; Ok({call}?) }},\n"));
    }
    code.push_str("_ => Err(crate::Error::Contract) } }\n");
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("upload_dispatch.rs"),
        code,
    )
    .unwrap();
}
