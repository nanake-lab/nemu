use std::collections::BTreeSet;

use core_types::WireMessage;
use core_types::contract::Limits;
use core_types::generated::ObjectRef;
use core_types::semantic::tendency_order;

use super::numeric::nonnegative;
use super::{CarrierResources, JFault};

const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_items: 4096,
    max_depth: 32,
};

/// In-process view of a resolved validation record, NOT a validation service
/// or a persisted certificate. The missing resource contract must eventually
/// bind these fields to actual P5-31 results (including unit directions and
/// hook/shape/dtype/mask). Numeric checks cannot establish that provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectionBinding {
    pub resources: CarrierResources,
    pub validation: ObjectRef,
    pub max_beta: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectionInput {
    pub direction: ObjectRef,
    pub beta: f64,
    pub binding: DirectionBinding,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearnedDirectionInput {
    pub tendency_id: String,
    /// A concept identifier only; never its definition or example text.
    pub concept_id: String,
    pub direction: DirectionInput,
}

/// Validated numeric configuration, immutable after construction. No default
/// β, model, validation range or resource version is selected by this module.
#[derive(Clone, Debug, PartialEq)]
pub struct CarrierConfig {
    pub(super) resources: CarrierResources,
    pub(super) nature: [DirectionInput; 4],
    pub(super) learned: Vec<LearnedDirectionInput>,
    pub(super) combination: DirectionBinding,
    pub(super) nature_bound: f64,
}

fn reference(value: &ObjectRef) -> Result<(), JFault> {
    value.canonical_bytes(LIMITS).map_err(JFault::Codec)?;
    Ok(())
}

fn binding(value: &mut DirectionBinding, resources: &CarrierResources) -> Result<(), JFault> {
    if &value.resources != resources {
        return Err(JFault::ResourceMismatch);
    }
    reference(&value.validation)?;
    value.max_beta = nonnegative(value.max_beta, "validated_beta")?;
    Ok(())
}

fn direction(value: &mut DirectionInput, resources: &CarrierResources) -> Result<(), JFault> {
    reference(&value.direction)?;
    binding(&mut value.binding, resources)?;
    value.beta = nonnegative(value.beta, "beta")?;
    if value.beta > value.binding.max_beta {
        return Err(JFault::BetaOutsideValidation);
    }
    Ok(())
}

impl CarrierConfig {
    pub fn new(
        resources: CarrierResources,
        mut nature: [DirectionInput; 4],
        mut learned: Vec<LearnedDirectionInput>,
        mut combination: DirectionBinding,
    ) -> Result<Self, JFault> {
        for item in [
            &resources.carrier,
            &resources.model,
            &resources.tokenizer,
            &resources.direction_set,
        ] {
            reference(item)?;
        }
        binding(&mut combination, &resources)?;
        let mut nature_bound = 0.0;
        for item in &mut nature {
            direction(item, &resources)?;
            nature_bound = super::numeric::add_bound(nature_bound, item.beta.abs())?;
        }
        if nature_bound > combination.max_beta {
            return Err(JFault::TotalBudgetExceeded);
        }
        let mut seen = BTreeSet::new();
        for item in &mut learned {
            if item.tendency_id.is_empty() || item.concept_id.is_empty() {
                return Err(JFault::InvalidConfig("learned_identifier"));
            }
            if !seen.insert(item.tendency_id.clone()) {
                return Err(JFault::DuplicateDirection(item.tendency_id.clone()));
            }
            direction(&mut item.direction, &resources)?;
        }
        learned.sort_by(|a, b| tendency_order(&a.tendency_id, &b.tendency_id));
        Ok(Self {
            resources,
            nature,
            learned,
            combination,
            nature_bound,
        })
    }
}
