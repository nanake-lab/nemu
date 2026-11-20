//! Local cognitive working materials, already governed and in their selected order.
//! W19 resolves historical reactions and renders their V2-88 marker. ADR 0042's
//! remote P/D/E/o packer has its own eligibility rules and request meter.
mod retrieval;
use core_types::generated::{CognitiveInput, CognitiveMaterial};
pub use retrieval::{RecalledMaterial, RecalledPage};

#[derive(Clone, Copy)]
pub struct MaterialBudget {
    pub items: usize,
    pub bytes: usize,
    pub tokens: usize,
}

/// Keep a prefix of whole materials. The caller meters the actual rendering,
/// including source labels, past-reaction markers and tokenizer boundaries.
/// Return the omitted count so the caller can retain the truncation decision.
pub fn bound_working_materials<E>(
    input: &mut CognitiveInput,
    budget: MaterialBudget,
    mut measure: impl FnMut(&[CognitiveMaterial]) -> Result<(usize, usize), E>,
) -> Result<usize, E> {
    let kept = fitting_prefix(&input.materials, budget, &mut measure)?;
    let omitted = input.materials.len() - kept;
    input.materials.truncate(kept);
    Ok(omitted)
}

fn fitting_prefix<T, E>(
    materials: &[T],
    budget: MaterialBudget,
    mut measure: impl FnMut(&[T]) -> Result<(usize, usize), E>,
) -> Result<usize, E> {
    let mut kept = 0;
    for end in 1..=materials.len().min(budget.items) {
        let (bytes, tokens) = measure(&materials[..end])?;
        if bytes > budget.bytes || tokens > budget.tokens {
            break;
        }
        kept = end;
    }
    Ok(kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_label_in_whole_material_and_preserves_sources() {
        let rows: serde_json::Value =
            serde_json::from_str(include_str!("../../../../contracts/golden/legal.json")).unwrap();
        let mut input: CognitiveInput = serde_json::from_value(
            rows.as_array()
                .unwrap()
                .iter()
                .find(|v| {
                    v["request"]["name"] == "CognitiveInput"
                        && v["request"]["value"]["materials"]
                            .as_array()
                            .is_some_and(|m| !m.is_empty())
                })
                .unwrap()["request"]["value"]
                .clone(),
        )
        .unwrap();
        let mut old = input.materials[0].clone();
        old.past_reaction = true;
        input.materials = vec![old.clone(), old.clone()];
        let full = "【这是她当时的反应】\n测试".len();
        let budget = MaterialBudget {
            items: 2,
            bytes: full,
            tokens: 2,
        };
        let omitted = bound_working_materials(&mut input, budget, |m| {
            Ok::<_, ()>((full * m.len(), 2 * m.len()))
        })
        .unwrap();
        assert_eq!(omitted, 1);
        assert_eq!(input.materials, [old]);
        assert!(
            bound_working_materials(&mut input, budget, |_| Err::<(usize, usize), _>(
                "unavailable"
            ))
            .is_err()
        );
        assert_eq!(input.materials.len(), 1);
    }
}
