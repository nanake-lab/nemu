#![forbid(unsafe_code)]
pub mod adapter;
pub mod probes;
pub mod resources;
pub mod world;
#[cfg(test)]
mod tests {
    use super::*;
    use core_kernel::recompute::{self, Kernel};
    #[test]
    fn mechanical_stage_replay_and_probe_isolation() {
        // Three fixed adapter records, no registered growth world or endpoint inspected.
        let events: Vec<_> = (1..=3)
            .map(|i| world::Event {
                ordinal: i,
                day: 1,
                time: i * 1_000_000_000,
                hours: 8,
                person: "U".into(),
                context: "task".into(),
                a: [0., 0., 0., if i == 2 { -0.25 } else { 0.25 }],
            })
            .collect();
        let h: Vec<_> = events
            .into_iter()
            .map(|e| recompute::Commit {
                id: format!("c{}", e.ordinal),
                position: e.ordinal,
                prefix: format!("p{}", e.ordinal),
                command: recompute::Command::Event(e),
                check: None,
            })
            .collect();
        let mut p =
            recompute::start(&adapter::Adapter, "EXPERIMENT-mechanical", adapter::scope()).unwrap();
        for n in 1..=h.len() {
            p = recompute::step(&adapter::Adapter, &p, &h[..n]).unwrap();
        }
        let replay = recompute::replay(
            &adapter::Adapter,
            "EXPERIMENT-mechanical",
            adapter::scope(),
            &h,
        )
        .unwrap();
        assert!(adapter::Adapter.same_bits(p.state(), replay.state()));
        let mut renamed = adapter::genesis().unwrap();
        for commit in &h {
            let recompute::Command::Event(mut event) = commit.command.clone() else {
                panic!("event");
            };
            event.person = adapter::rename_id(&event.person, false);
            renamed = adapter::advance_named(&renamed, &event, true).unwrap();
        }
        let mut inverse = adapter::state_json(&renamed).unwrap();
        adapter::inverse_names(&mut inverse);
        assert_eq!(inverse, adapter::state_json(p.state()).unwrap());
        let before = adapter::state_json(p.state()).unwrap();
        let restored = continuation::restore(before.clone()).unwrap();
        assert!(adapter::same(&restored, p.state()).unwrap());
        for probe in world::probes() {
            probes::probe(p.state(), &probe, world::DAY).unwrap();
        }
        assert_eq!(before, adapter::state_json(p.state()).unwrap());
    }
    #[test]
    fn world_tables_have_fixed_counts_and_q1_sequence() {
        let rs = world::runs();
        assert_eq!(
            rs.len(),
            rs.iter()
                .map(|r| &r.id)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
        for r in world::runs().iter().filter(|r| r.status == "ready") {
            let es = world::events(r);
            assert_eq!(
                es.len(),
                if r.world == "W5" {
                    0
                } else {
                    (r.days * r.density) as usize
                }
            );
            assert!(es.windows(2).all(|e| e[0].time < e[1].time));
        }
        for a in world::runs().iter().filter(|r| r.world == "W6") {
            let mut b = a.clone();
            b.world = "W7".into();
            assert!(
                world::events(a)
                    .iter()
                    .zip(world::events(&b))
                    .all(|(x, y)| x.a == y.a && x.time == y.time && x.context != y.context)
            );
        }
    }
    #[test]
    fn all_people_follow_preregistered_roles_including_q2() {
        let mut runs = world::runs();
        for seed in [53001, 53002] {
            for hours in [2, 8, 24] {
                runs.push(world::Run {
                    id: "Q2-common".into(),
                    world: "W0".into(),
                    seed,
                    density: 4,
                    hours,
                    days: 30,
                    status: "ready".into(),
                    reason: String::new(),
                });
            }
        }
        for run in runs.iter().filter(|r| r.status == "ready") {
            for event in world::events(run) {
                let special =
                    matches!(run.world.as_str(), "W6" | "W7") || run.world.starts_with("O5-");
                let seventh = ((event.ordinal - 1) / 12) % 7 == 6;
                let relation = event.context == "secret" || event.context == "punctual";
                let expected = if !special && seventh && relation {
                    "V"
                } else {
                    "U"
                };
                assert_eq!(event.person, expected, "{} event {}", run.id, event.ordinal);
            }
        }
    }
    #[test]
    fn no_event_growth_and_zero_marginal_rounding() {
        let g = adapter::genesis().unwrap();
        let before = adapter::state_json(&g).unwrap();
        probes::daily(&g, 365, &world::probes()).unwrap();
        assert_eq!(before, adapter::state_json(&g).unwrap());
        let contribution = 2f64.powi(-54);
        assert_eq!((0.5 + contribution) - 0.5, 0.);
        assert_ne!(0.5 - contribution, 0.5);
    }
}
pub mod compress;
pub mod continuation;
