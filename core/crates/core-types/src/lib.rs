#![forbid(unsafe_code)]
//! Echo Core validated value types. No state transitions or storage access.

mod canonical;
pub mod contract;
pub mod numeric;
mod protobuf;
pub mod scalar;
pub mod semantic;
#[rustfmt::skip]
pub mod generated;

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
    fn canonical_bytes(&self, limits: contract::Limits) -> contract::Result<Vec<u8>> {
        let schema = contract::Schema::builtin()?;
        schema.encode_canonical(
            Self::NAME,
            &serde_json::to_value(self).map_err(|e| e.to_string())?,
            limits,
        )
    }
    fn protobuf_bytes(&self, limits: contract::Limits) -> contract::Result<Vec<u8>> {
        contract::Schema::builtin()?.encode_protobuf(
            Self::NAME,
            &serde_json::to_value(self).map_err(|e| e.to_string())?,
            limits,
        )
    }
    fn from_protobuf(bytes: &[u8], limits: contract::Limits) -> contract::Result<Self> {
        let value = contract::Schema::builtin()?.decode_protobuf(Self::NAME, bytes, limits, 0)?;
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
    fn from_canonical(bytes: &[u8], limits: contract::Limits) -> contract::Result<Self> {
        let value = contract::Schema::builtin()?.decode_canonical(Self::NAME, bytes, limits, 0)?;
        serde_json::from_value(value).map_err(|e| e.to_string())
    }
}
