//! CanonicalV1 encoding for core-internal policy objects (ADR 0007).
//!
//! Scope predicates and effect parameter vectors are content-addressed
//! through `ObjectRef` in `EffectContract`. They are not IDL messages; they
//! are interpreted only inside Trusted Core under the versioned
//! `scope_interpreter`/carrier resources of Ω. Their byte layout follows the
//! same rules as ADR 0007: NCV1 header, u32 schema version, length-prefixed
//! frozen type tag, little-endian fixed-width integers, length-prefixed
//! strict UTF-8, sets sorted by canonical key bytes, finite binary64 with
//! negative zero normalised, and SHA-256 with type-tag domain separation.

use sha2::{Digest, Sha256};

use crate::error::PolicyError;
use crate::scope::{Atom, Scope, ScopeExpr};

pub const SCHEMA_VERSION: u32 = 1;
/// Frozen type tag of a scope predicate object.
pub const SCOPE_TAG: &str = "echo.scope-predicate";

/// Canonical writer with checked lengths.
#[derive(Default)]
pub struct Writer {
    pub bytes: Vec<u8>,
}

impl Writer {
    pub fn with_header(tag: &str) -> Result<Self, PolicyError> {
        let mut w = Self::default();
        w.bytes.extend_from_slice(b"NCV1");
        w.u32(SCHEMA_VERSION);
        w.blob(tag.as_bytes())?;
        Ok(w)
    }

    pub fn u32(&mut self, v: u32) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }

    /// Finite binary64, negative zero normalised to positive zero.
    pub fn f64(&mut self, v: f64) -> Result<(), PolicyError> {
        if !v.is_finite() {
            return Err(PolicyError::Encoding("non-finite".into()));
        }
        let v = if v == 0.0 { 0.0 } else { v };
        self.bytes.extend_from_slice(&v.to_bits().to_le_bytes());
        Ok(())
    }

    pub fn count(&mut self, n: usize) -> Result<(), PolicyError> {
        let n = u32::try_from(n).map_err(|_| PolicyError::Encoding("count".into()))?;
        self.u32(n);
        Ok(())
    }

    pub fn blob(&mut self, b: &[u8]) -> Result<(), PolicyError> {
        self.count(b.len())?;
        self.bytes.extend_from_slice(b);
        Ok(())
    }
}

/// Canonical bytes of a length-prefixed string, used as a set sort key.
fn string_key(s: &str) -> Result<Vec<u8>, PolicyError> {
    let mut w = Writer::default();
    w.blob(s.as_bytes())?;
    Ok(w.bytes)
}

/// SHA-256 over the domain-separated canonical bytes (ADR 0007 rule 7).
pub fn digest(tag: &str, canonical: &[u8]) -> Result<[u8; 32], PolicyError> {
    let mut w = Writer::default();
    w.bytes.extend_from_slice(b"NEMU-CANONICALV1\0");
    w.blob(tag.as_bytes())?;
    w.bytes.extend_from_slice(canonical);
    Ok(Sha256::digest(&w.bytes).into())
}

/// Canonical bytes of a validated scope.
pub fn scope_bytes(scope: &Scope) -> Result<Vec<u8>, PolicyError> {
    let mut w = Writer::with_header(SCOPE_TAG)?;
    w.u32(scope.endpoint.code());
    w.blob(scope.catalog_version.as_bytes())?;
    write_expr(&mut w, &scope.expr)?;
    Ok(w.bytes)
}

pub fn scope_digest(scope: &Scope) -> Result<[u8; 32], PolicyError> {
    digest(SCOPE_TAG, &scope_bytes(scope)?)
}

fn write_expr(w: &mut Writer, expr: &ScopeExpr) -> Result<(), PolicyError> {
    match expr {
        ScopeExpr::True => w.u32(0),
        ScopeExpr::Atom(atom) => {
            w.u32(1);
            write_atom(w, atom)?;
        }
        ScopeExpr::And(items) | ScopeExpr::Or(items) => {
            w.u32(if matches!(expr, ScopeExpr::And(_)) {
                2
            } else {
                3
            });
            w.count(items.len())?;
            for item in items {
                write_expr(w, item)?;
            }
        }
    }
    Ok(())
}

fn write_atom(w: &mut Writer, atom: &Atom) -> Result<(), PolicyError> {
    match atom {
        Atom::EnumIn { field, values } => {
            w.u32(1);
            w.blob(field.as_bytes())?;
            w.count(values.len())?;
            // u32 LE canonical bytes do not sort like integers; sort by bytes.
            let mut keys: Vec<[u8; 4]> = values.iter().map(|v| v.to_le_bytes()).collect();
            keys.sort_unstable();
            keys.iter().for_each(|k| w.bytes.extend_from_slice(k));
        }
        Atom::ConceptIn { field, concepts } => {
            w.u32(2);
            w.blob(field.as_bytes())?;
            w.count(concepts.len())?;
            let mut keys = concepts
                .iter()
                .map(|c| string_key(c))
                .collect::<Result<Vec<_>, _>>()?;
            keys.sort_unstable();
            keys.iter().for_each(|k| w.bytes.extend_from_slice(k));
        }
        Atom::NumIn { field, lo, hi } => {
            w.u32(3);
            w.blob(field.as_bytes())?;
            w.i64(*lo);
            w.i64(*hi);
        }
    }
    Ok(())
}
