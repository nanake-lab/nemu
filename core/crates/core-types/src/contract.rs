//! Descriptor-based validation shared by the two distinct encodings.
use crate::scalar::{hex, unhex};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_items: usize,
    pub max_depth: usize,
}
#[derive(Deserialize)]
pub struct Field {
    pub name: String,
    pub r#type: String,
    pub number: u32,
    pub repeated: bool,
    pub required: bool,
    pub nonempty: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub size: Option<usize>,
    pub set: Option<String>,
    pub oneof: Option<String>,
}
#[derive(Deserialize)]
pub struct Message {
    pub tag: String,
    pub file: String,
    #[serde(default)]
    pub transient: bool,
    pub superseded: Option<serde_json::Value>,
    pub fields: Vec<Field>,
    pub rules: Rules,
}
#[derive(Deserialize)]
pub struct Oneof {
    pub fields: Vec<String>,
}
#[derive(Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub positive: Vec<String>,
    #[serde(default)]
    pub at_least_abs: BTreeMap<String, String>,
    #[serde(default)]
    pub oneofs: BTreeMap<String, Oneof>,
    pub discriminator: Option<String>,
    #[serde(default)]
    pub common: Vec<String>,
    #[serde(default)]
    pub lengths: BTreeMap<String, String>,
    pub cases: Option<BTreeMap<String, Case>>,
}
#[derive(Deserialize)]
pub struct Case {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
    #[serde(default)]
    pub values: BTreeMap<String, Vec<u64>>,
}
#[derive(Deserialize)]
pub struct Schema {
    pub messages: BTreeMap<String, Message>,
    pub enums: BTreeMap<String, BTreeMap<String, u32>>,
}
pub fn require(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
impl Schema {
    pub fn builtin() -> Result<Self> {
        serde_json::from_str(include_str!("../../../contracts/generated/schema.json"))
            .map_err(|e| e.to_string())
    }
    pub fn message(&self, name: &str) -> Result<&Message> {
        self.messages.get(name).ok_or("unknown message".into())
    }
    pub fn normalize(&self, name: &str, v: &Value, limits: Limits, depth: usize) -> Result<Value> {
        require(depth <= limits.max_depth, "depth limit")?;
        let m = self.message(name)?;
        let input = v.as_object().ok_or("message object")?;
        require(
            input.keys().all(|k| m.fields.iter().any(|f| &f.name == k)),
            "unknown field",
        )?;
        let mut out = Map::new();
        for f in &m.fields {
            let Some(v) = input.get(&f.name).filter(|v| !v.is_null()) else {
                require(!f.required, "missing required")?;
                continue;
            };
            let normalized = if f.repeated {
                let a = v.as_array().ok_or("collection")?;
                require(
                    a.len() <= limits.max_items && u32::try_from(a.len()).is_ok(),
                    "count limit",
                )?;
                let mut values = a
                    .iter()
                    .map(|v| self.scalar(f, v, limits, depth))
                    .collect::<Result<Vec<_>>>()?;
                if let Some(key) = &f.set {
                    let mut keys = BTreeSet::new();
                    let mut pairs = Vec::new();
                    for item in values {
                        let (typ, val) = if key == "self" {
                            (f.r#type.as_str(), &item)
                        } else {
                            let k = self
                                .message(&f.r#type)?
                                .fields
                                .iter()
                                .find(|x| &x.name == key)
                                .ok_or("set key")?;
                            (k.r#type.as_str(), item.get(key).ok_or("set key")?)
                        };
                        let encoded = self.canonical_scalar(typ, val, limits)?;
                        require(keys.insert(encoded.clone()), "duplicate set key")?;
                        pairs.push((encoded, item));
                    }
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    values = pairs.into_iter().map(|p| p.1).collect();
                }
                Value::Array(values)
            } else {
                self.scalar(f, v, limits, depth)?
            };
            out.insert(f.name.clone(), normalized);
        }
        require(
            out.get("schema_version").and_then(Value::as_u64) == Some(1),
            "unknown schema",
        )?;
        for (key, count) in &m.rules.lengths {
            require(
                out.get(key)
                    .and_then(Value::as_array)
                    .map(|a| a.len() as u64)
                    == out.get(count).and_then(Value::as_u64),
                "dimension mismatch",
            )?;
        }
        if let Some(cases) = &m.rules.cases {
            let discriminator = m.rules.discriminator.as_deref().unwrap_or("kind");
            let c = cases
                .get(&out[discriminator].to_string())
                .ok_or("variant")?;
            require(
                c.required.iter().all(|k| out.contains_key(k))
                    && out.keys().all(|k| {
                        k == "schema_version"
                            || k == discriminator
                            || m.rules.common.contains(k)
                            || c.required.contains(k)
                            || c.optional.contains(k)
                    }),
                "variant fields",
            )?;
            for (k, values) in &c.values {
                require(
                    out.get(k)
                        .and_then(Value::as_u64)
                        .is_some_and(|v| values.contains(&v)),
                    "variant enum",
                )?;
            }
        }
        for group in m.rules.oneofs.values() {
            require(
                group.fields.iter().filter(|k| out.contains_key(*k)).count() == 1,
                "required oneof",
            )?;
        }
        for key in &m.rules.positive {
            require(
                out.get(key)
                    .and_then(Value::as_f64)
                    .is_some_and(|v| v > 0.0),
                "positive field required",
            )?;
        }
        for (key, signed) in &m.rules.at_least_abs {
            let v = out
                .get(key)
                .and_then(Value::as_f64)
                .ok_or("absolute evidence")?;
            let a = out
                .get(signed)
                .and_then(Value::as_f64)
                .ok_or("signed evidence")?;
            require(v >= a.abs(), "absolute evidence below signed magnitude")?;
        }
        Ok(Value::Object(out))
    }
    fn scalar(&self, f: &Field, v: &Value, l: Limits, d: usize) -> Result<Value> {
        let t = f.r#type.as_str();
        if self.messages.contains_key(t) {
            return self.normalize(t, v, l, d + 1);
        }
        let out = if let Some(e) = self.enums.get(t) {
            let n = v.as_u64().ok_or("enum")?;
            require(
                n != 0 && e.values().any(|x| u64::from(*x) == n),
                "unknown enum",
            )?;
            v.clone()
        } else {
            match t {
                "uint32" => {
                    require(
                        v.as_u64().is_some_and(|x| u32::try_from(x).is_ok()),
                        "u32 overflow",
                    )?;
                    v.clone()
                }
                "uint64" => {
                    let s = v.as_str().ok_or("u64 decimal string")?;
                    let n = s.parse::<u64>().map_err(|_| "u64 overflow")?;
                    require(s == n.to_string(), "decimal spelling")?;
                    v.clone()
                }
                "sint64" => {
                    let s = v.as_str().ok_or("i64 decimal string")?;
                    let n = s.parse::<i64>().map_err(|_| "i64 overflow")?;
                    require(s == n.to_string(), "decimal spelling")?;
                    v.clone()
                }
                "double" => {
                    let n = v.as_f64().ok_or("double")?;
                    require(n.is_finite(), "nonfinite")?;
                    Value::from(if n == 0.0 { 0.0 } else { n })
                }
                "bool" => {
                    require(v.is_boolean(), "boolean")?;
                    v.clone()
                }
                "string" => {
                    let s = v.as_str().ok_or("string")?;
                    require(s.len() <= l.max_bytes, "string limit")?;
                    if f.nonempty {
                        require(!s.is_empty(), "empty")?;
                    }
                    v.clone()
                }
                "bytes" => {
                    let s = v.as_str().ok_or("hex bytes")?;
                    require(s.len() / 2 <= l.max_bytes, "bytes limit")?;
                    let b = unhex(s)?;
                    if let Some(n) = f.size {
                        require(b.len() == n, "byte size")?;
                    }
                    Value::String(hex(&b))
                }
                _ => return Err("unknown type".into()),
            }
        };
        if let Some(min) = f.min {
            require(out.as_f64().is_some_and(|v| v >= min), "below minimum")?;
        }
        if let Some(max) = f.max {
            require(out.as_f64().is_some_and(|v| v <= max), "above maximum")?;
        }
        Ok(out)
    }
}

pub struct Reader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}
impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8], l: Limits) -> Result<Self> {
        require(data.len() <= l.max_bytes, "message size")?;
        Ok(Self { data, pos: 0 })
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        require(n <= self.data.len() - self.pos, "truncated")?;
        let b = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(b)
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| "u32")?,
        ))
    }
    pub fn blob(&mut self) -> Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    pub fn done(&self) -> bool {
        self.pos == self.data.len()
    }
}
pub fn append_blob(out: &mut Vec<u8>, b: &[u8]) -> Result<()> {
    out.extend(
        u32::try_from(b.len())
            .map_err(|_| "length overflow")?
            .to_le_bytes(),
    );
    out.extend(b);
    Ok(())
}
