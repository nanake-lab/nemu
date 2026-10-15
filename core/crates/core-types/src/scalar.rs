//! Lossless JSON bridge for golden fixtures; JSON is never the authority hash.
use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! decimal {
    ($name:ident, $ty:ty) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(pub $ty);
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.0.to_string())
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = String::deserialize(d)?;
                let n = s.parse::<$ty>().map_err(serde::de::Error::custom)?;
                if n.to_string() != s {
                    return Err(serde::de::Error::custom("noncanonical integer"));
                }
                Ok(Self(n))
            }
        }
    };
}
decimal!(DecimalU64, u64);
decimal!(DecimalI64, i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HexBytes(pub Vec<u8>);
impl Serialize for HexBytes {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex(&self.0))
    }
}
impl<'de> Deserialize<'de> for HexBytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        unhex(&String::deserialize(d)?)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid hex".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
pub fn finite<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    if !value.is_finite() {
        return Err(serde::ser::Error::custom("nonfinite"));
    }
    serializer.serialize_f64(if *value == 0.0 { 0.0 } else { *value })
}
pub fn finite_option<S: serde::Serializer>(
    value: &Option<f64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(v) => finite(v, serializer),
        None => serializer.serialize_none(),
    }
}
pub fn finite_vec<S: serde::Serializer>(value: &[f64], serializer: S) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    if value.iter().any(|v| !v.is_finite()) {
        return Err(serde::ser::Error::custom("nonfinite"));
    }
    value
        .iter()
        .map(|v| if *v == 0.0 { 0.0 } else { *v })
        .collect::<Vec<_>>()
        .serialize(serializer)
}
