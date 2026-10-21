//! Explanation shares of one evidence group (v2.0 §5.2 份额; LP2, I25; D3).
//!
//! A group that several tendencies can explain is shared, not counted twice.
//! The tendencies that the group supports (S > 0) split it in proportion to
//! (1 + π)/2, and so do those it contradicts (S < 0); each side sums to 1.
//! π is the value before the group (V2-40). Sums run in the canonical order
//! of tendency IDs.

use core_types::semantic::tendency_order;

use super::error::{LawError, LawResult};
use super::num::finite;

/// One matching tendency of a group: its ID, π before the group, and S_{c,k}.
#[derive(Clone, Copy, Debug)]
pub struct ShareInput<'a> {
    pub tendency_id: &'a str,
    pub pi: f64,
    pub support: f64,
}

/// The share of each input, in input order. S = 0 gets share 0.
pub fn shares(inputs: &[ShareInput<'_>]) -> LawResult<Vec<f64>> {
    let mut order: Vec<usize> = (0..inputs.len()).collect();
    order.sort_by(|&a, &b| tendency_order(inputs[a].tendency_id, inputs[b].tendency_id));
    if order
        .windows(2)
        .any(|p| inputs[p[0]].tendency_id == inputs[p[1]].tendency_id)
    {
        return Err(LawError::InvalidInput(
            "a tendency appears twice in one group".into(),
        ));
    }
    for x in inputs {
        if !(x.pi > -1.0 && x.pi < 1.0) || !(-1.0..=1.0).contains(&x.support) {
            return Err(LawError::InvalidInput(format!(
                "{}: share needs pi in (-1, 1) and S in [-1, 1]",
                x.tendency_id
            )));
        }
    }
    let q = |x: &ShareInput<'_>| (1.0 + x.pi) / 2.0;
    let mut out = vec![0.0; inputs.len()];
    for positive in [true, false] {
        let side: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| {
                let s = inputs[i].support;
                if positive { s > 0.0 } else { s < 0.0 }
            })
            .collect();
        let total = side.iter().fold(0.0, |acc, &i| acc + q(&inputs[i]));
        for &i in &side {
            out[i] = finite(q(&inputs[i]) / total, "share")?;
        }
    }
    Ok(out)
}
