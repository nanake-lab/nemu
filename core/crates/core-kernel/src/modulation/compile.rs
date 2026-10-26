use core_types::generated::{LearnedCoefficient, Lineage, NatureReal};

use crate::effects::{ProcessTime, compile_authorized_effects};
use crate::fast;

use super::numeric::{add_bound, finite};
use super::*;

fn nature_real(values: [f64; 4]) -> NatureReal {
    NatureReal {
        schema_version: 1,
        connection: values[0],
        empathy: values[1],
        curiosity: values[2],
        achievement: values[3],
    }
}

fn item(
    direction: &DirectionInput,
    tendency_id: Option<String>,
    x: f64,
) -> Result<IncludedDirection, JFault> {
    Ok(IncludedDirection {
        tendency_id,
        direction: direction.direction.clone(),
        validation: direction.binding.validation.clone(),
        beta: direction.beta,
        coefficient: finite(direction.beta * x, "coefficient")?,
    })
}

fn receipt(
    snapshot_id: &str,
    anchor_time: ProcessTime,
    projected_at: ProcessTime,
    mode: CompilationMode,
    config: &CarrierConfig,
) -> Result<NumericReceipt, JFault> {
    if snapshot_id.is_empty() {
        return Err(JFault::EmptySnapshotId);
    }
    if projected_at < anchor_time {
        return Err(JFault::TimeBeforeAnchor);
    }
    Ok(NumericReceipt {
        snapshot_id: snapshot_id.to_owned(),
        anchor_time,
        projected_at,
        mode,
        resources: config.resources.clone(),
        combination_validation: config.combination.validation.clone(),
        included: Vec::new(),
        excluded: Vec::new(),
        body_readout_version: String::new(),
    })
}

/// Numeric part of J(θ,x,R,t,validity;carrier_config). The committed x is
/// projected exactly once via P5-10, and R⁺ is resolved by P5-27. It returns
/// neither a forged receipt hash nor a production ModulationEnvelopeV2.
pub fn compile_numeric(
    view: JStateView<'_>,
    config: &CarrierConfig,
    body: &BodyReadoutConfig,
) -> Result<NumericCompilation, JFault> {
    let mut receipt = receipt(
        view.snapshot_id,
        view.anchor_time,
        view.process_time,
        CompilationMode::Connected,
        config,
    )?;
    let effects = compile_authorized_effects(view.r, view.theta, view.process_time, view.validity)
        .map_err(JFault::Effects)?;
    let x = fast::project_time(view.x, &effects, view.process_time.0 - view.anchor_time.0)
        .map_err(JFault::Fast)?;
    let mut nature = [0.0; 4];
    for (i, direction) in config.nature.iter().enumerate() {
        let contribution = item(direction, None, x.nature[i])?;
        nature[i] = contribution.coefficient;
        receipt.included.push(contribution);
    }
    let mut bound = config.nature_bound;
    let mut learned = Vec::new();
    for (effect, value) in effects.effects.iter().zip(&x.learned) {
        if !effect.direction_validated {
            receipt.excluded.push(ExcludedDirection {
                tendency_id: effect.tendency_id.clone(),
                reason: ExclusionReason::Unvalidated,
            });
            continue;
        }
        let found = config.learned.binary_search_by(|d| {
            core_types::semantic::tendency_order(&d.tendency_id, &effect.tendency_id)
        });
        let Ok(index) = found else {
            receipt.excluded.push(ExcludedDirection {
                tendency_id: effect.tendency_id.clone(),
                reason: ExclusionReason::MissingDirection,
            });
            continue;
        };
        let direction = &config.learned[index];
        if direction.concept_id != effect.concept {
            return Err(JFault::ConceptMismatch(effect.tendency_id.clone()));
        }
        // Absolute worst-case budget, independent of current x, sign or any
        // cancellation between physical directions. Never shrink older terms.
        bound = add_bound(bound, direction.direction.beta.abs())?;
        if bound > config.combination.max_beta {
            return Err(JFault::TotalBudgetExceeded);
        }
        let contribution = item(
            &direction.direction,
            Some(effect.tendency_id.clone()),
            value.value,
        )?;
        learned.push(LearnedCoefficient {
            schema_version: 1,
            tendency_id: effect.tendency_id.clone(),
            coefficient: contribution.coefficient,
        });
        receipt.included.push(contribution);
    }
    let zero = nature.iter().all(|v| *v == 0.0) && learned.iter().all(|v| v.coefficient == 0.0);
    let body = body.read(&x.nature, zero)?;
    receipt.body_readout_version = body.version.clone();
    Ok(NumericCompilation {
        nature_coefficients: nature_real(nature),
        learned_coefficients: learned,
        total_bound: bound,
        body,
        receipt,
    })
}

/// Explicit early-rehearsal F-disabled path. It has no x input and therefore
/// cannot project a fabricated trajectory. LIFE/EXPERIMENT cannot use it.
/// The runtime still has to set MODULATION_NOT_ENABLED and stop autonomy.
pub fn rehearsal_without_fast(
    lineage: Lineage,
    snapshot_id: &str,
    process_time: ProcessTime,
    config: &CarrierConfig,
    body: &BodyReadoutConfig,
) -> Result<NumericCompilation, JFault> {
    if lineage != Lineage::REHEARSAL {
        return Err(JFault::RehearsalOnly);
    }
    let mut receipt = receipt(
        snapshot_id,
        process_time,
        process_time,
        CompilationMode::RehearsalWithoutFast,
        config,
    )?;
    for direction in &config.nature {
        receipt.included.push(item(direction, None, 0.0)?);
    }
    let body = body.read(&[0.0; 4], true)?;
    receipt.body_readout_version = body.version.clone();
    Ok(NumericCompilation {
        nature_coefficients: nature_real([0.0; 4]),
        learned_coefficients: Vec::new(),
        total_bound: config.nature_bound,
        body,
        receipt,
    })
}
