//! Strict Protobuf transport parser: validates before exposing a message.
use crate::contract::{Limits, Reader, Result, Schema, require};
use crate::scalar::{hex, unhex};
use serde_json::{Map, Value};

fn varint(mut n: u64) -> Vec<u8> {
    let mut b = Vec::new();
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
    b
}
fn read_varint(r: &mut Reader<'_>) -> Result<u64> {
    let mut n = 0;
    for i in 0..10 {
        let b = r.take(1)?[0];
        require(i < 9 || b <= 1, "varint overflow")?;
        n |= u64::from(b & 127) << (7 * i);
        if b < 128 {
            return Ok(n);
        }
    }
    Err("varint overflow".into())
}
fn length<'a>(r: &mut Reader<'a>) -> Result<&'a [u8]> {
    let n = usize::try_from(read_varint(r)?).map_err(|_| "length overflow")?;
    r.take(n)
}
impl Schema {
    fn wire_type(&self, t: &str) -> u64 {
        if self.messages.contains_key(t) || matches!(t, "string" | "bytes") {
            2
        } else if t == "double" {
            1
        } else {
            0
        }
    }
    fn wire_atom(&self, t: &str, v: &Value, l: Limits) -> Result<Vec<u8>> {
        let bytes = if self.messages.contains_key(t) {
            self.encode_protobuf(t, v, l)?
        } else {
            match t {
                "string" => v.as_str().ok_or("string")?.as_bytes().to_vec(),
                "bytes" => unhex(v.as_str().ok_or("bytes")?)?,
                "double" => return Ok(v.as_f64().ok_or("double")?.to_le_bytes().to_vec()),
                "sint64" => {
                    let n = v.as_str().ok_or("i64")?.parse::<i64>().map_err(|_| "i64")?;
                    return Ok(varint(((n as u64) << 1) ^ ((n >> 63) as u64)));
                }
                "uint64" => {
                    return Ok(varint(
                        v.as_str().ok_or("u64")?.parse::<u64>().map_err(|_| "u64")?,
                    ));
                }
                "bool" => return Ok(varint(u64::from(v.as_bool().ok_or("boolean")?))),
                _ => return Ok(varint(v.as_u64().ok_or("enum/uint32")?)),
            }
        };
        let mut out = varint(bytes.len() as u64);
        out.extend(bytes);
        Ok(out)
    }
    pub fn encode_protobuf(&self, name: &str, v: &Value, l: Limits) -> Result<Vec<u8>> {
        let v = self.normalize(name, v, l, 0)?;
        let mut out = Vec::new();
        for f in &self.message(name)?.fields {
            let Some(value) = v.get(&f.name) else {
                continue;
            };
            let items = if f.repeated {
                value.as_array().ok_or("array")?.iter().collect::<Vec<_>>()
            } else {
                vec![value]
            };
            for item in items {
                out.extend(varint(
                    (u64::from(f.number) << 3) | self.wire_type(&f.r#type),
                ));
                out.extend(self.wire_atom(&f.r#type, item, l)?);
                require(out.len() <= l.max_bytes, "message size")?;
            }
        }
        Ok(out)
    }
    fn read_wire_atom(
        &self,
        t: &str,
        wire: u64,
        r: &mut Reader<'_>,
        l: Limits,
        d: usize,
    ) -> Result<Value> {
        require(wire == self.wire_type(t), "wire type")?;
        if wire == 2 {
            let b = length(r)?;
            return if self.messages.contains_key(t) {
                self.decode_protobuf(t, b, l, d + 1)
            } else if t == "string" {
                Ok(Value::String(
                    std::str::from_utf8(b).map_err(|_| "UTF-8")?.to_owned(),
                ))
            } else {
                Ok(Value::String(hex(b)))
            };
        }
        if wire == 1 {
            let n = f64::from_le_bytes(r.take(8)?.try_into().map_err(|_| "double")?);
            require(n.is_finite(), "nonfinite")?;
            return Ok(Value::from(n));
        }
        let n = read_varint(r)?;
        Ok(match t {
            "sint64" => Value::String(((n >> 1) as i64 ^ -((n & 1) as i64)).to_string()),
            "uint64" => Value::String(n.to_string()),
            "bool" => {
                require(n < 2, "boolean")?;
                Value::from(n == 1)
            }
            _ => Value::from(n),
        })
    }
    pub fn decode_protobuf(&self, name: &str, b: &[u8], l: Limits, d: usize) -> Result<Value> {
        require(d <= l.max_depth, "depth limit")?;
        let m = self.message(name)?;
        let mut r = Reader::new(b, l)?;
        let mut out = Map::new();
        for f in &m.fields {
            if f.repeated {
                out.insert(f.name.clone(), Value::Array(Vec::new()));
            }
        }
        while !r.done() {
            let key = read_varint(&mut r)?;
            let number = key >> 3;
            let wire = key & 7;
            let f = m
                .fields
                .iter()
                .find(|f| u64::from(f.number) == number)
                .ok_or("unknown field")?;
            if !f.repeated {
                require(!out.contains_key(&f.name), "duplicate singular")?;
                out.insert(
                    f.name.clone(),
                    self.read_wire_atom(&f.r#type, wire, &mut r, l, d)?,
                );
            } else {
                let a = out
                    .get_mut(&f.name)
                    .and_then(Value::as_array_mut)
                    .ok_or("array")?;
                if wire == 2 && self.wire_type(&f.r#type) != 2 {
                    let mut packed = Reader::new(length(&mut r)?, l)?;
                    while !packed.done() {
                        require(a.len() < l.max_items, "count limit")?;
                        a.push(self.read_wire_atom(
                            &f.r#type,
                            self.wire_type(&f.r#type),
                            &mut packed,
                            l,
                            d,
                        )?);
                    }
                } else {
                    require(a.len() < l.max_items, "count limit")?;
                    a.push(self.read_wire_atom(&f.r#type, wire, &mut r, l, d)?);
                }
            }
        }
        self.normalize(name, &Value::Object(out), l, d)
    }
}
