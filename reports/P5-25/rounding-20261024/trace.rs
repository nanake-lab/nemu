// Diagnostic uses production numeric.rs and the already-pinned libm crate.
extern crate libm;
use std::collections::BTreeMap;
#[derive(Debug)]
pub enum Fault { NonFinite, Underflow, ZeroDenominator, CounterExhausted }
pub type Result<T> = std::result::Result<T, Fault>;
#[allow(dead_code)]
#[path = "../../../core/crates/core-kernel/src/dynamics/numeric.rs"]
mod numeric;
use numeric as n;
fn main() {
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let input: BTreeMap<_,_> = text.lines().map(|s| {
        let (name, bits) = s.split_once('\t').unwrap();
        (name, f64::from_bits(u64::from_str_radix(bits, 16).unwrap()))
    }).collect();
    let residual = n::sub(input["a"], input["old_baseline"]).unwrap();
    let residual_square = n::mul(residual, residual).unwrap();
    let decayed_square_sum = n::mul(input["rho"], input["old_square_sum"]).unwrap();
    let square_sum = n::add(decayed_square_sum, residual_square).unwrap();
    let decayed_weight = n::mul(input["rho"], input["old_weight"]).unwrap();
    let weight = n::add(decayed_weight, 1.0).unwrap();
    let decayed_reaction_sum = n::mul(input["rho"], input["old_reaction_sum"]).unwrap();
    let reaction_sum = n::add(decayed_reaction_sum, input["a"]).unwrap();
    let prior_baseline_numerator = n::mul(input["nb"], input["b0"]).unwrap();
    let baseline_numerator = n::add(prior_baseline_numerator, reaction_sum).unwrap();
    let baseline_denominator = n::add(input["nb"], weight).unwrap();
    let baseline = n::div(baseline_numerator, baseline_denominator).unwrap();
    let prior_sigma_square = n::mul(0.3, 0.3).unwrap();
    let weighted_prior_variance = n::mul(input["ns"], prior_sigma_square).unwrap();
    let variance_numerator = n::add(weighted_prior_variance, square_sum).unwrap();
    let variance_denominator = n::add(input["ns"], weight).unwrap();
    let variance = n::div(variance_numerator, variance_denominator).unwrap();
    let sigma = n::finite(libm::sqrt(variance)).unwrap();
    let gain_numerator = n::mul(1.0, 0.3).unwrap();
    let raw_gain = n::div(gain_numerator, sigma).unwrap();
    let gain = raw_gain.clamp(0.2, 3.0);
    for (name,value) in [
        ("residual",residual),("residual_square",residual_square),
        ("decayed_square_sum",decayed_square_sum),("square_sum",square_sum),
        ("decayed_weight",decayed_weight),("weight",weight),
        ("decayed_reaction_sum",decayed_reaction_sum),("reaction_sum",reaction_sum),
        ("prior_baseline_numerator",prior_baseline_numerator),
        ("baseline_numerator",baseline_numerator),("baseline_denominator",baseline_denominator),
        ("baseline",baseline),("prior_sigma_square",prior_sigma_square),
        ("weighted_prior_variance",weighted_prior_variance),("variance_numerator",variance_numerator),
        ("variance_denominator",variance_denominator),("variance",variance),
        ("sigma",sigma),("gain_numerator",gain_numerator),("raw_gain",raw_gain),("gain",gain),
        ("sqrt_python_variance",libm::sqrt(input["python_variance"])),
        ("sqrt_rust_variance",libm::sqrt(input["rust_order_variance"])),
    ] {println!("{name}\t{value:.17}\t{:016x}",value.to_bits());}
}
