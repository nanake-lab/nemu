//! ADR 0046 §2.1 exact application bytes. This storage-only leaf has no IDL descriptor.
use crate::{LineageKind, PreparedObject, Result, Store, error::require};
use core_types::{
    generated::ObjectRef,
    scalar::{DecimalU64, HexBytes, hex},
};
use sha2::{Digest, Sha256};

pub const RAW_BYTES: &str = "nemu.storage.raw-bytes-v1";

pub(crate) fn storage_key(r: &ObjectRef) -> String {
    let digest = hex(&r.digest.0);
    if r.object_type == RAW_BYTES {
        format!("raw-{digest}")
    } else {
        digest
    }
}

pub(crate) fn verify(r: &ObjectRef, bytes: &[u8], limit: u64) -> Result<()> {
    require(
        r.resource_version == "1"
            && r.byte_length.0 == bytes.len() as u64
            && r.byte_length.0 <= limit
            && Sha256::digest(bytes).as_slice() == r.digest.0,
        "raw resource digest, length or version mismatch",
    )
}

fn credential_field(key: &str) -> bool {
    let key = key.to_ascii_lowercase().replace(['-', '_'], "");
    [
        "authorization",
        "apikey",
        "accesstoken",
        "refreshtoken",
        "authtoken",
        "securitytoken",
        "clientsecret",
        "secretaccesskey",
        "subscriptionkey",
        "password",
        "credentials",
        "cookie",
    ]
    .iter()
    .any(|suffix| key.ends_with(suffix))
}
fn credential_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower
        .split(|c: char| !c.is_ascii_alphanumeric() && !matches!(c, '-' | '_'))
        .any(|word| {
            word.strip_prefix("sk-")
                .is_some_and(|suffix| suffix.len() >= 8)
        })
        || lower.contains("bearer ")
        || text
            .split_inclusive(':')
            .filter_map(|part| part.strip_suffix(':'))
            .any(|part| {
                let before = part.trim_end().trim_end_matches('"');
                let key = before
                    .rsplit(|c: char| !c.is_ascii_alphanumeric() && !matches!(c, '-' | '_'))
                    .next()
                    .unwrap_or("");
                credential_field(key)
            })
}
/// Scan the actual JSON string tokens, including shadowed duplicate members.
fn protected_json(bytes: &[u8]) -> bool {
    let mut offset = 0;
    while let Some(at) = bytes[offset..].iter().position(|b| *b == b'"') {
        offset += at;
        let mut strings =
            serde_json::Deserializer::from_slice(&bytes[offset..]).into_iter::<String>();
        let Some(Ok(s)) = strings.next() else {
            return true;
        };
        offset += strings.byte_offset();
        let key = bytes[offset..].iter().find(|b| !b.is_ascii_whitespace()) == Some(&b':');
        if (key && (credential_field(&s) || s.eq_ignore_ascii_case("headers")))
            || credential_text(&s)
        {
            return true;
        }
    }
    false
}
pub(crate) fn request_body(bytes: &[u8]) -> Result<()> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|_| crate::Error::Rejected("expected JSON application request body"))?;
    require(
        value.is_object() && !protected_json(bytes),
        "transport headers or credentials in application body",
    )
}
impl<L: LineageKind> Store<L> {
    pub fn prepare_request_body(&self, bytes: &[u8]) -> Result<PreparedObject<L>> {
        require(
            bytes.len() as u64 <= self.objects.config.max_object_bytes,
            "raw resource exceeds budget",
        )?;
        request_body(bytes)?;
        self.prepare_raw(bytes)
    }
    pub fn prepare_raw(&self, bytes: &[u8]) -> Result<PreparedObject<L>> {
        require(
            crate::migration::format(&self.db)? == 3,
            "raw resource requires format 3",
        )?;
        require(
            bytes.len() as u64 <= self.objects.config.max_object_bytes,
            "raw resource exceeds budget",
        )?;
        // Inspect the ASCII credential syntax even in binary resources. The original
        // bytes are hashed and stored without converting or repairing UTF-8.
        let inspection = String::from_utf8_lossy(bytes);
        require(
            !credential_text(&inspection)
                && !(serde_json::from_slice::<serde_json::Value>(bytes).is_ok()
                    && protected_json(bytes)),
            "credential-like content in resource",
        )?;
        let reference = ObjectRef {
            schema_version: 1,
            object_type: RAW_BYTES.into(),
            resource_version: "1".into(),
            digest: HexBytes(Sha256::digest(bytes).to_vec()),
            byte_length: DecimalU64(bytes.len() as u64),
        };
        let mut pins = self
            .objects
            .pins
            .lock()
            .map_err(|_| crate::Error::Rejected("retention lock poisoned"))?;
        self.objects.write(&reference, bytes, &self.fault)?;
        *pins.entry(storage_key(&reference)).or_default() += 1;
        Ok(PreparedObject {
            closure: vec![reference.clone()],
            reference,
            pins: self.objects.pins.clone(),
            _lineage: std::marker::PhantomData,
        })
    }
}
