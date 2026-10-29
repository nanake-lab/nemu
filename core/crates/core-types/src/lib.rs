#![forbid(unsafe_code)]
//! Echo Core validated value types. No state transitions or storage access.

mod canonical;
pub mod contract;
pub mod numeric;
mod protobuf;
pub mod scalar;
pub mod semantic;
pub mod terminal_ingress;
#[rustfmt::skip]
pub mod generated;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CodecKey {
    tag: String,
    schema_version: u32,
    generated_schema_sha256: [u8; 32],
}
struct BuiltinCodec {
    schema: std::sync::Arc<contract::Schema>,
    hash: [u8; 32],
    descriptors: std::collections::BTreeMap<CodecKey, std::sync::Arc<contract::Schema>>,
}
impl BuiltinCodec {
    fn build() -> contract::Result<Self> {
        use sha2::{Digest, Sha256};
        let hash =
            Sha256::digest(include_bytes!("../../../contracts/generated/schema.json")).into();
        let schema = std::sync::Arc::new(contract::Schema::builtin()?);
        // A descriptor needs the complete immutable graph for nested types.
        // Every key shares that graph; no objects or validation results are cached.
        // Version 1 is the supported schema version, still checked by normalize.
        let descriptors = schema
            .messages
            .values()
            .map(|message| {
                (
                    CodecKey {
                        tag: message.tag.clone(),
                        schema_version: 1,
                        generated_schema_sha256: hash,
                    },
                    std::sync::Arc::clone(&schema),
                )
            })
            .collect();
        Ok(Self {
            schema,
            hash,
            descriptors,
        })
    }
    fn schema_for(&self, name: &str) -> &contract::Schema {
        let descriptor = self.schema.messages.get(name).and_then(|message| {
            self.descriptors.get(&CodecKey {
                tag: message.tag.clone(),
                schema_version: 1,
                generated_schema_sha256: self.hash,
            })
        });
        // Unknown names still go through the original codec, preserving its
        // validation order and exact error rather than adding a cache error.
        descriptor.unwrap_or(&self.schema)
    }
}
fn codec_schema_in<'a>(
    cache: &'a std::sync::OnceLock<contract::Result<BuiltinCodec>>,
    name: &str,
) -> contract::Result<&'a contract::Schema> {
    cache
        .get_or_init(BuiltinCodec::build)
        .as_ref()
        .map(|codec| codec.schema_for(name))
        .map_err(Clone::clone)
}
fn codec_schema(name: &str) -> contract::Result<&'static contract::Schema> {
    static BUILTIN: std::sync::OnceLock<contract::Result<BuiltinCodec>> =
        std::sync::OnceLock::new();
    codec_schema_in(&BUILTIN, name)
}

fn verify_typed<T: WireMessage>(
    value: &serde_json::Value,
    limits: contract::Limits,
) -> Result<(), String> {
    let typed: T = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    let canonical = typed.canonical_bytes(limits)?;
    let protobuf = typed.protobuf_bytes(limits)?;
    contract::require(
        T::from_canonical(&canonical, limits)?.canonical_bytes(limits)? == canonical,
        "typed canonical roundtrip",
    )?;
    contract::require(
        T::from_protobuf(&protobuf, limits)?.canonical_bytes(limits)? == canonical,
        "typed protobuf roundtrip",
    )
}

pub trait WireMessage: serde::Serialize + serde::de::DeserializeOwned {
    const NAME: &'static str;
    /// True for types superseded by a later contract (P5-21). They stay
    /// decodable for old data and tests; no runtime path may produce them.
    const SUPERSEDED: bool = false;
    /// Transient credentials must never become authoritative objects.
    const PERSISTABLE: bool = true;
    fn canonical_bytes(&self, limits: contract::Limits) -> contract::Result<Vec<u8>> {
        let schema = codec_schema(Self::NAME)?;
        schema.encode_canonical(
            Self::NAME,
            &serde_json::to_value(self).map_err(|e| e.to_string())?,
            limits,
        )
    }
    fn protobuf_bytes(&self, limits: contract::Limits) -> contract::Result<Vec<u8>> {
        codec_schema(Self::NAME)?.encode_protobuf(
            Self::NAME,
            &serde_json::to_value(self).map_err(|e| e.to_string())?,
            limits,
        )
    }
    fn from_protobuf(bytes: &[u8], limits: contract::Limits) -> contract::Result<Self> {
        let value = codec_schema(Self::NAME)?.decode_protobuf(Self::NAME, bytes, limits, 0)?;
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
    fn from_canonical(bytes: &[u8], limits: contract::Limits) -> contract::Result<Self> {
        let value = codec_schema(Self::NAME)?.decode_canonical(Self::NAME, bytes, limits, 0)?;
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod codec_cache_tests {
    use super::*;
    use contract::{Limits, Schema};
    use scalar::{hex, unhex};
    use serde_json::{Value, json};
    use std::sync::OnceLock;

    fn probe(schema: &Schema, request: &Value) -> contract::Result<Value> {
        let l = &request["limits"];
        let limits = Limits {
            max_bytes: l["max_bytes"].as_u64().unwrap() as usize,
            max_items: l["max_items"].as_u64().unwrap() as usize,
            max_depth: l["max_depth"].as_u64().unwrap() as usize,
        };
        let name = request["name"].as_str().unwrap();
        let value = match request["op"].as_str().unwrap() {
            "encode" => schema.normalize(name, &request["value"], limits, 0)?,
            "canonical" => schema.decode_canonical(
                name,
                &unhex(request["hex"].as_str().unwrap())?,
                limits,
                0,
            )?,
            "protobuf" => schema.decode_protobuf(
                name,
                &unhex(request["hex"].as_str().unwrap())?,
                limits,
                0,
            )?,
            _ => return Err("operation".into()),
        };
        generated::verify_typed(name, &value, limits)?;
        Ok(
            json!({"ok":true,"value":value,"canonical":hex(&schema.encode_canonical(name,&value,limits)?),"protobuf":hex(&schema.encode_protobuf(name,&value,limits)?),"digest":schema.digest(name,&value,limits)?}),
        )
    }

    #[test]
    fn all_legal_and_illegal_golden_vectors_are_equal_cold_hot_and_uncached() {
        for (legal, bytes) in [
            (true, include_str!("../../../contracts/golden/legal.json")),
            (
                false,
                include_str!("../../../contracts/golden/illegal.json"),
            ),
        ] {
            let fixtures: Vec<Value> = serde_json::from_str(bytes).unwrap();
            for (index, fixture) in fixtures.iter().enumerate() {
                let request = &fixture["request"];
                let name = request["name"].as_str().unwrap();
                let uncached = probe(&Schema::builtin().unwrap(), request);
                // A fresh cell per fixture exercises real initialization, then
                // repeats through the exact production cache accessor hot.
                let cache = OnceLock::new();
                let cold = probe(codec_schema_in(&cache, name).unwrap(), request);
                let hot = probe(codec_schema_in(&cache, name).unwrap(), request);
                assert_eq!(cold, uncached, "cold fixture {legal}/{index}");
                assert_eq!(hot, uncached, "hot fixture {legal}/{index}");
                assert_eq!(cold.is_ok(), legal, "fixture {legal}/{index}");
                if legal {
                    assert_eq!(
                        cold.unwrap(),
                        fixture["expected"],
                        "frozen bytes/digest/value {index}"
                    );
                }
            }
            println!(
                "CODEC_GOLDEN legal={legal} vectors={} cold/hot/uncached equal",
                fixtures.len()
            );
        }
    }

    #[test]
    fn codec_keys_bind_tag_schema_version_hash_and_share_only_immutable_graph() {
        use sha2::{Digest, Sha256};
        let codec = BuiltinCodec::build().unwrap();
        let expected: [u8; 32] =
            Sha256::digest(include_bytes!("../../../contracts/generated/schema.json")).into();
        assert_eq!(codec.hash, expected);
        assert!(codec.descriptors.len() > 100);
        for (key, schema) in &codec.descriptors {
            assert_eq!(key.schema_version, 1);
            assert_eq!(key.generated_schema_sha256, expected);
            assert!(schema.messages.values().any(|m| m.tag == key.tag));
            assert!(std::sync::Arc::ptr_eq(schema, &codec.schema));
        }
        let cache = OnceLock::new();
        std::thread::scope(|scope| {
            let expected = codec_schema_in(&cache, "UserText").unwrap();
            for _ in 0..8 {
                scope.spawn(|| {
                    assert!(std::ptr::eq(
                        expected,
                        codec_schema_in(&cache, "UserText").unwrap()
                    ))
                });
            }
        });
    }
}
