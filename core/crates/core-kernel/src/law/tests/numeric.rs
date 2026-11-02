//! D9: the exact group mean against reference vectors computed with Python's
//! `Fraction` (`reports/P5-23/gen_mean_vectors.py`).

use crate::law::exact_mean;

use super::mean_vectors::MEAN_VECTORS;

#[test]
fn exact_mean_matches_correctly_rounded_reference() {
    let mut checked = 0;
    for (i, (input, expected)) in MEAN_VECTORS.iter().enumerate() {
        let xs: Vec<f64> = input.iter().map(|b| f64::from_bits(*b)).collect();
        let got = exact_mean(&xs).unwrap();
        assert_eq!(got.to_bits(), *expected, "vector {i}: {xs:?}");
        checked += 1;
    }
    assert_eq!(checked, 283);
}
