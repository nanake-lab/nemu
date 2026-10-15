//! CanonicalV1 authority bytes; Protobuf transport is deliberately separate.
use crate::contract::{Limits, Reader, Result, Schema, append_blob, require};
use crate::scalar::{hex, unhex};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

impl Schema {
    pub(crate) fn canonical_scalar(&self, t: &str, v: &Value, l: Limits) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        if self.messages.contains_key(t) {
            append_blob(&mut out, &self.encode_canonical(t, v, l)?)?;
        } else if self.enums.contains_key(t) || t == "uint32" {
            out.extend(
                u32::try_from(v.as_u64().ok_or("u32")?)
                    .map_err(|_| "u32 overflow")?
                    .to_le_bytes(),
            );
        } else {
            match t {
                "uint64" => out.extend(
                    v.as_str()
                        .ok_or("u64")?
                        .parse::<u64>()
                        .map_err(|_| "u64")?
                        .to_le_bytes(),
                ),
                "sint64" => out.extend(
                    v.as_str()
                        .ok_or("i64")?
                        .parse::<i64>()
                        .map_err(|_| "i64")?
                        .to_le_bytes(),
                ),
                "double" => out.extend(v.as_f64().ok_or("double")?.to_le_bytes()),
                "bool" => out.push(u8::from(v.as_bool().ok_or("bool")?)),
                "string" => append_blob(&mut out, v.as_str().ok_or("string")?.as_bytes())?,
                "bytes" => append_blob(&mut out, &unhex(v.as_str().ok_or("bytes")?)?)?,
                _ => return Err("unknown type".into()),
            }
        }
        Ok(out)
    }
    pub fn encode_canonical(&self, name: &str, v: &Value, l: Limits) -> Result<Vec<u8>> {
        let v = self.normalize(name, v, l, 0)?;
        let m = self.message(name)?;
        let mut out = b"NCV1".to_vec();
        out.extend(1u32.to_le_bytes());
        append_blob(&mut out, m.tag.as_bytes())?;
        for f in &m.fields[1..] {
            let value = v.get(&f.name);
            if f.repeated {
                let a = value.and_then(Value::as_array).ok_or("array")?;
                out.extend(u32::try_from(a.len()).map_err(|_| "count")?.to_le_bytes());
                for item in a {
                    out.extend(self.canonical_scalar(&f.r#type, item, l)?);
                    require(out.len() <= l.max_bytes, "message size")?;
                }
            } else {
                if !f.required {
                    out.push(u8::from(value.is_some()));
                }
                if let Some(value) = value {
                    out.extend(self.canonical_scalar(&f.r#type, value, l)?);
                }
            }
            require(out.len() <= l.max_bytes, "message size")?;
        }
        require(out.len() <= l.max_bytes, "message size")?;
        Ok(out)
    }
    pub fn decode_canonical(&self, name: &str, b: &[u8], l: Limits, depth: usize) -> Result<Value> {
        require(depth <= l.max_depth, "depth limit")?;
        let m = self.message(name)?;
        let mut r = Reader::new(b, l)?;
        require(r.take(4)? == b"NCV1" && r.u32()? == 1, "magic/schema")?;
        require(r.blob()? == m.tag.as_bytes(), "type tag")?;
        let mut v = Map::new();
        v.insert("schema_version".into(), 1.into());
        for f in &m.fields[1..] {
            if f.repeated {
                let count = r.u32()? as usize;
                require(count <= l.max_items, "count limit")?;
                let mut a = Vec::new();
                for _ in 0..count {
                    a.push(self.read_canonical_atom(&f.r#type, &mut r, l, depth)?);
                }
                v.insert(f.name.clone(), Value::Array(a));
            } else {
                let present = if f.required {
                    true
                } else {
                    let p = r.take(1)?[0];
                    require(p < 2, "presence")?;
                    p == 1
                };
                if present {
                    v.insert(
                        f.name.clone(),
                        self.read_canonical_atom(&f.r#type, &mut r, l, depth)?,
                    );
                }
            }
        }
        require(r.done(), "trailing bytes")?;
        let out = self.normalize(name, &Value::Object(v), l, depth)?;
        require(
            self.encode_canonical(name, &out, l)? == b,
            "noncanonical encoding",
        )?;
        Ok(out)
    }
    fn read_canonical_atom(
        &self,
        t: &str,
        r: &mut Reader<'_>,
        l: Limits,
        d: usize,
    ) -> Result<Value> {
        if self.messages.contains_key(t) {
            return self.decode_canonical(t, r.blob()?, l, d + 1);
        }
        if self.enums.contains_key(t) || t == "uint32" {
            return Ok(r.u32()?.into());
        }
        Ok(match t {
            "uint64" => Value::String(
                u64::from_le_bytes(r.take(8)?.try_into().map_err(|_| "u64")?).to_string(),
            ),
            "sint64" => Value::String(
                i64::from_le_bytes(r.take(8)?.try_into().map_err(|_| "i64")?).to_string(),
            ),
            "double" => {
                let n = f64::from_le_bytes(r.take(8)?.try_into().map_err(|_| "f64")?);
                require(n.is_finite(), "nonfinite")?;
                Value::from(n)
            }
            "bool" => {
                let b = r.take(1)?[0];
                require(b < 2, "boolean")?;
                Value::from(b == 1)
            }
            "string" => Value::String(
                std::str::from_utf8(r.blob()?)
                    .map_err(|_| "UTF-8")?
                    .to_owned(),
            ),
            "bytes" => Value::String(hex(r.blob()?)),
            _ => return Err("type".into()),
        })
    }
    pub fn digest(&self, name: &str, v: &Value, l: Limits) -> Result<String> {
        let mut input = b"NEMU-CANONICALV1\0".to_vec();
        append_blob(&mut input, self.message(name)?.tag.as_bytes())?;
        input.extend(self.encode_canonical(name, v, l)?);
        Ok(hex(&Sha256::digest(input)))
    }
}
