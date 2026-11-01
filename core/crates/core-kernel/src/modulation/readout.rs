use core_types::generated::BodyReadoutV2;

use super::JFault;
use super::numeric::finite;

/// Resolved genesis coefficients, in the fixed four-nature order. The view
/// is local, not a persisted resource format. No aesthetic defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct BodyReadoutConfig {
    version: String,
    weights: [[f64; 4]; 3],
}

impl BodyReadoutConfig {
    pub fn new(version: String, weights: [[f64; 4]; 3]) -> Result<Self, JFault> {
        if version.is_empty() {
            return Err(JFault::InvalidConfig("body_readout_version"));
        }
        let mut weights = weights;
        for row in &mut weights {
            for value in row {
                *value = finite(*value, "body_readout_weight")?;
            }
        }
        Ok(Self { version, weights })
    }

    pub(super) fn read(&self, nature: &[f64; 4], zero: bool) -> Result<BodyReadoutV2, JFault> {
        let mut values = [0.5, 0.0, 0.5];
        if !zero {
            for (row, value) in self.weights.iter().zip(&mut values) {
                let mut sum = 0.0;
                for (weight, x) in row.iter().zip(nature) {
                    let term = finite(weight * x, "body_readout_product")?;
                    sum = finite(sum + term, "body_readout_sum")?;
                }
                // Check BEFORE clipping: an overflow is a fault, not an extreme posture.
                *value = finite(*value + sum, "body_readout_offset")?;
            }
        }
        Ok(BodyReadoutV2 {
            schema_version: 1,
            version: self.version.clone(),
            facing: values[0].clamp(0.0, 1.0),
            tension: finite(values[1].clamp(-1.0, 1.0), "tension")?,
            energy: values[2].clamp(0.0, 1.0),
        })
    }
}
