//! EvaluateEffects (O 3.2; S 12): evaluation only. It charges nothing and
//! writes nothing; the receipt is for audit, not a new memory.

use std::collections::BTreeMap;

use core_policy::scope::evaluate;
use core_policy::{Catalog, Endpoint, FieldId, FieldKind, FieldValue, ScopeMatch};

use super::types::{AuthorizedEffectSet, Owner};

/// Context for one endpoint, holding only whitelisted, typed fields.
#[derive(Clone, Debug, PartialEq)]
pub struct TypedContext {
    endpoint: Endpoint,
    catalog_version: String,
    values: BTreeMap<FieldId, FieldValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextError {
    FieldNotAllowed(String),
    KindMismatch(String),
    NonFinite(String),
}

impl TypedContext {
    /// Builds a context; any field outside the endpoint whitelist is rejected
    /// rather than silently dropped.
    pub fn new(
        endpoint: Endpoint,
        catalog: &Catalog,
        values: BTreeMap<FieldId, FieldValue>,
    ) -> Result<Self, ContextError> {
        for (field, value) in &values {
            let spec = catalog
                .fields
                .get(field)
                .filter(|s| s.endpoints.contains(&endpoint))
                .ok_or_else(|| ContextError::FieldNotAllowed(field.clone()))?;
            let ok = matches!(
                (&spec.kind, value),
                (FieldKind::Enum { .. }, FieldValue::Enum(_))
                    | (FieldKind::Concept { .. }, FieldValue::Concept(_))
                    | (FieldKind::Numeric { .. }, FieldValue::Number(_))
            );
            if !ok {
                return Err(ContextError::KindMismatch(field.clone()));
            }
            if let FieldValue::Number(v) = value
                && !v.is_finite()
            {
                return Err(ContextError::NonFinite(field.clone()));
            }
        }
        Ok(Self {
            endpoint,
            catalog_version: catalog.version.clone(),
            values,
        })
    }

    pub fn endpoint(&self) -> Endpoint {
        self.endpoint
    }
}

/// One hit, recorded for audit.
#[derive(Clone, Debug, PartialEq)]
pub struct ReceiptEntry {
    pub responsibility_id: String,
    pub causal_revision: String,
    pub owner: Owner,
    pub endpoint: Endpoint,
    /// Actual magnitude applied: factor · L1(parameters).
    pub magnitude: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Evaluation {
    /// Sparse sum of hit effects, accumulated in the set's canonical order.
    pub values: BTreeMap<u32, f64>,
    pub receipt: Vec<ReceiptEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvaluateError {
    /// The endpoint context lacks an input the scope needs (O 3.2).
    UnsupportedScope {
        responsibility_id: String,
        field: String,
    },
    /// Set and context were built under different interpreter versions, or
    /// evaluation produced a non-finite value.
    DependencyFault(String),
}

/// Evaluates the authorised effects of `context.endpoint()`.
pub fn evaluate_effects(
    set: &AuthorizedEffectSet,
    context: &TypedContext,
    catalog: &Catalog,
) -> Result<Evaluation, EvaluateError> {
    if set.catalog_version != context.catalog_version || catalog.version != set.catalog_version {
        return Err(EvaluateError::DependencyFault(
            "interpreter version mismatch".into(),
        ));
    }
    let mut values: BTreeMap<u32, f64> = BTreeMap::new();
    let mut receipt = Vec::new();
    for effect in set
        .effects
        .iter()
        .filter(|e| e.endpoint == context.endpoint)
    {
        let matched = evaluate(&effect.scope.expr, &context.values, catalog)
            .map_err(|e| EvaluateError::DependencyFault(e.to_string()))?;
        match matched {
            ScopeMatch::Miss => {}
            ScopeMatch::MissingField(field) => {
                return Err(EvaluateError::UnsupportedScope {
                    responsibility_id: effect.responsibility_id.clone(),
                    field,
                });
            }
            ScopeMatch::Hit => {
                for (index, coefficient) in &effect.parameters.entries {
                    *values.entry(*index).or_insert(0.0) += effect.factor * coefficient;
                }
                receipt.push(ReceiptEntry {
                    responsibility_id: effect.responsibility_id.clone(),
                    causal_revision: effect.causal_revision.clone(),
                    owner: effect.owner,
                    endpoint: effect.endpoint,
                    magnitude: effect.factor * effect.parameters.l1(),
                });
            }
        }
    }
    if values.values().any(|v| !v.is_finite()) {
        return Err(EvaluateError::DependencyFault("non-finite sum".into()));
    }
    Ok(Evaluation { values, receipt })
}
