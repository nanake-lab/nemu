use super::*;

// An adversarial law for the generic cache contract. A production law may
// canonicalize zero on ingress; the mechanism must not assume that it does.
struct BitKernel;
impl Kernel<f64> for BitKernel {
    type State = Vec<u64>;
    type Input = f64;
    fn same_event(&self, a: &f64, b: &f64) -> bool {
        a.to_bits() == b.to_bits()
    }
    fn same_input(&self, a: &f64, b: &f64) -> bool {
        a.to_bits() == b.to_bits()
    }
    fn genesis(&self, _: &Scope) -> Result<Self::State> {
        Ok(vec![])
    }
    fn project(&self, h: &[Commit<f64>], _: &Scope) -> Result<Vec<f64>> {
        let events: Vec<_> = h
            .iter()
            .filter_map(|r| match r.command {
                Command::Event(x) => Some(x),
                _ => None,
            })
            .collect();
        // A newly appended interpretation revises every old projected input.
        Ok(vec![*events.last().unwrap_or(&0.0); events.len()])
    }
    fn advance(&self, state: &Self::State, input: &f64, _: &Scope) -> Result<Self::State> {
        let mut next = state.clone();
        next.push(input.to_bits());
        Ok(next)
    }
    fn same_bits(&self, a: &Self::State, b: &Self::State) -> bool {
        a == b
    }
}

#[test]
fn zero_sign_change_in_projection_invalidates_incremental_cache() {
    let kernel = BitKernel;
    let mut h = vec![commit(1, Command::Event(0.0))];
    let old = replay(&kernel, "experiment", scope(), &h).unwrap();
    h.push(commit(2, Command::Event(-0.0)));
    let inc = step(&kernel, &old, &h).unwrap();
    let full = replay(&kernel, "experiment", scope(), &h).unwrap();
    assert_eq!(inc.state(), full.state());
    assert_eq!(inc.state(), &vec![(-0.0_f64).to_bits(); 2]);
    assert_eq!(old.state(), &vec![0.0_f64.to_bits()]);
}

#[test]
fn zero_sign_change_in_old_history_is_not_an_immutable_extension() {
    let kernel = BitKernel;
    let mut h = vec![commit(1, Command::Event(0.0))];
    let old = replay(&kernel, "experiment", scope(), &h).unwrap();
    h[0].command = Command::Event(-0.0);
    h.push(commit(2, Command::Event(1.0)));
    assert!(matches!(step(&kernel, &old, &h), Err(Fault::History(_))));
}
