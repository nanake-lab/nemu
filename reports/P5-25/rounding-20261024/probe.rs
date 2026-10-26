// Diagnostic only: compare exponent operations without altering production C.
extern crate libm;
use std::hint::black_box;
fn main() {
    let residual = std::env::args().nth(1).unwrap().parse::<f64>().unwrap();
    for (name, value) in [
        ("multiply", residual * residual),
        ("libm_pow", libm::pow(residual, 2.0)),
        ("std_powf_constant", residual.powf(2.0)),
        ("std_powf_runtime", residual.powf(black_box(2.0))),
    ] { println!("{name}\t{value:.17}\t{:016x}", value.to_bits()); }
}
