//! User-confirmed P5-25 acceptance boundary (2026-10-26), not a runtime mock.
//! x/m are caller-owned fixture coordinates, never inputs to C. Real H/L
//! transactions and independent full-history replay belong to P5-26/P5-29.
use super::*;

type PureC = fn(
    &DynamicsState,
    Option<&ReactionEvent>,
    &DynamicsLaw,
    &DynamicsFuses,
) -> Result<StagedDynamics>;

struct CallerFixture {
    x: [f64; 4],
    m: [f64; 4],
    theta: DynamicsState,
}

#[test]
fn ac2_09_changing_caller_x_initial_and_m_preserves_every_theta_and_estimator_bit() {
    // Compile-time check: no x, m, R, ledger or commit capability in C's API.
    let c: PureC = stage;
    let initial = DynamicsState::genesis(law()).unwrap();
    let events: Vec<_> = (0..128)
        .map(|i| event(i, if i % 29 < 14 { 0.6 } else { -0.4 }))
        .collect();
    let mut baseline = initial.clone();
    let expected: Vec<_> = events
        .iter()
        .map(|e| {
            baseline = advance(&baseline, e);
            baseline.clone()
        })
        .collect();
    assert_ne!(expected.last().unwrap(), &initial); // rejects identity C
    for x in [[0.0; 4], [-1.0, 1.0, -0.5, 0.5], [1.0, -1.0, 0.75, -0.75]] {
        for m in [[0.0; 4], [0.25, -0.5, 0.75, -1.0], [-1.0, 0.75, -0.5, 0.25]] {
            let mut caller = CallerFixture {
                x,
                m,
                theta: initial.clone(),
            };
            for (e, expected) in events.iter().zip(&expected) {
                let before = caller.theta.clone();
                let result = c(&caller.theta, Some(e), &caller.theta.law, &fuses()).unwrap();
                identical(&caller.theta, &before); // no implicit application
                assert_eq!(
                    components(result.state().values()).map(f64::to_bits),
                    components(expected.values()).map(f64::to_bits),
                );
                identical(result.state(), expected); // every estimator field too
                caller.theta = result.into_state(); // explicit local caller choice
            }
            assert_eq!(caller.x.map(f64::to_bits), x.map(f64::to_bits));
            assert_eq!(caller.m.map(f64::to_bits), m.map(f64::to_bits));
        }
    }
    println!(
        "AC2-09 pure C: 9 x-initial/m combinations, 128 identical external events, every prefix bit-identical"
    );
}

#[test]
fn i13_c_only_stages_theta_change_and_cannot_commit_or_mutate_the_input() {
    let c: PureC = stage;
    let input = DynamicsState::genesis(law()).unwrap();
    let before = input.clone();
    let e = event(0, 0.8);
    let first = c(&input, Some(&e), &input.law, &fuses()).unwrap();
    let change = first.change.as_ref().unwrap();
    assert_eq!(change.before, input.values);
    assert_eq!(change.after, first.state().values);
    assert_eq!(change.events, vec![e.id.clone()]);
    assert_ne!(change.before, change.after);
    identical(&input, &before);
    // Discarding the result cannot advance C's input or a hidden event cursor.
    let again = c(&input, Some(&e), &input.law, &fuses()).unwrap();
    identical(first.state(), again.state());
    assert_eq!(first.change, again.change);
    drop(first);
    drop(again);
    identical(&input, &before);
    let no_event = c(&input, None, &input.law, &fuses()).unwrap();
    assert!(no_event.change.is_none());
    identical(no_event.state(), &before);
}
