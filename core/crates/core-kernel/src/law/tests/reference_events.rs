//! Same-input phase-machine states with prepared tendencies and W/S plans.
use super::fixture::tendency;
use super::reference_support::{ReferenceChild, fault_code};
use super::state_bits::{group_bits_eq, snapshot_bits_eq};
use super::world::{Fixture, default_theta};
use crate::law::pending::{Allocation, EvidenceGroup, Tendency};
use crate::law::plan::{Plan, Values};
use crate::law::{GroupContent, GroupItem, RState, Step};
use core_types::scalar::DecimalU64;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

struct Input<'a>(std::str::SplitWhitespace<'a>);
impl Input<'_> {
    fn token(&mut self) -> &str {
        self.0.next().unwrap()
    }
    fn number(&mut self) -> usize {
        self.token().parse().unwrap()
    }
    fn float(&mut self) -> f64 {
        f64::from_bits(u64::from_str_radix(self.token(), 16).unwrap())
    }
    fn id(&mut self) -> String {
        format!("{:<32}", self.token()).replace(' ', "!")
    }
    fn states(&mut self, f: &Fixture, p: f64) -> Vec<Tendency> {
        let n = self.number();
        let mut out = Vec::new();
        for _ in 0..n {
            let id = self.id();
            let mut t = tendency(&f.law, &id, if p == 0.0 { None } else { Some(p) });
            t.evidence_a = self.float();
            t.evidence_v = self.float();
            t.magnitude_sum = self.float();
            t.magnitude_weight = self.float();
            t.pi = self.float();
            t.evidence_b = self.float();
            t.h_hat = self.float();
            t.typical_magnitude = self.float();
            t.group_count = DecimalU64(self.number() as u64);
            t.flip_count = DecimalU64(self.number() as u64);
            t.last_sign = match self.token() {
                "1" => Some(core_types::generated::EvidenceSign::SIGN_POSITIVE),
                "-1" => Some(core_types::generated::EvidenceSign::SIGN_NEGATIVE),
                "0" => None,
                s => panic!("{s}"),
            };
            out.push(t);
        }
        out
    }
    fn groups(&mut self) -> BTreeMap<String, EvidenceGroup> {
        let n = self.number();
        let mut groups = BTreeMap::new();
        for _ in 0..n {
            let id = self.id();
            let gid = self.id();
            let ordinal = self.number() as u64;
            let intensity = self.float();
            let support = self.float();
            let share = self.float();
            let actual_retention = self.float();
            let g = groups.entry(gid.clone()).or_insert_with(|| EvidenceGroup {
                schema_version: 1,
                group_id: gid.clone(),
                members: vec![format!("e:{gid}")],
                source_key: gid,
                credibility: 1.0,
                verifiability: 1.0,
                allocations: Vec::new(),
            });
            g.allocations.push(Allocation {
                schema_version: 1,
                tendency_id: id,
                support,
                intensity,
                share,
                ordinal: DecimalU64(ordinal),
                mu: 0.0,
                projection: None,
                actual_retention,
            });
        }
        for g in groups.values_mut() {
            g.allocations
                .sort_by(|a, b| a.tendency_id.cmp(&b.tendency_id));
        }
        groups
    }
}

#[test]
fn adr0019_python_events_review_and_side_tapes_match_g_phase_machine() {
    let mut child = ReferenceChild(
        Command::new(std::env::var("P523_PYTHON").unwrap_or_else(|_| "python".into()))
            .arg("-B")
            .arg(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src/law/tests/reference_event_vectors.py"),
            )
            .env("PYTHONUTF8", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let mut checked = 0;
    let mut faults = 0;
    for (line_number, line) in BufReader::new(child.0.stdout.take().unwrap())
        .lines()
        .enumerate()
    {
        let line = line.unwrap();
        if let Some(s) = line.strip_prefix("R ") {
            println!("EVENT_REFERENCE {s}");
            continue;
        }
        let mut input = Input(line.split_whitespace());
        assert_eq!(input.token(), "E");
        let prior = input.float();
        let revision = input.id();
        let competitor = input.id();
        let competitor_support = input.float();
        let new_group = input.id();
        let new_weight = input.float();
        let n = input.number();
        let mut new_supports = Vec::new();
        for _ in 0..n {
            new_supports.push((input.id(), input.float()));
        }
        let mut f = Fixture::new(u32::MAX);
        f.law.w_max = f64::MAX;
        let mut states = input.states(&f, prior);
        let mut groups = input.groups();
        for name in new_supports.iter().map(|(id, _)| id).chain(
            std::iter::once(&competitor).filter(|_| revision.trim_end_matches('!') != "none"),
        ) {
            if !states.iter().any(|t| t.tendency_id == *name) {
                states.push(tendency(
                    &f.law,
                    name,
                    if prior == 0.0 { None } else { Some(prior) },
                ));
            }
        }
        let snapshot = f.snapshot(states);
        let mut items = Vec::new();
        let mut plans = BTreeMap::new();
        if revision.trim_end_matches('!') != "none" {
            let old = &groups[&revision];
            let mut values: BTreeMap<String, Values> = old
                .allocations
                .iter()
                .map(|a| {
                    (
                        a.tendency_id.clone(),
                        Values {
                            support: a.support,
                            intensity: a.intensity,
                            mu: 0.0,
                            projection: None,
                        },
                    )
                })
                .collect();
            let intensity = old.allocations[0].intensity;
            values.insert(
                competitor.clone(),
                Values {
                    support: competitor_support,
                    intensity,
                    mu: 0.0,
                    projection: None,
                },
            );
            plans.insert(
                revision.clone(),
                Plan {
                    credibility: 1.0,
                    verifiability: 1.0,
                    reshare_all: false,
                    values,
                },
            );
            items.push(GroupItem {
                group_id: revision.clone(),
                position: 0,
                source_key: revision.clone(),
                members: old.members.clone(),
                verifiable: true,
                content: GroupContent::Experience {
                    appraisals: Vec::new(),
                    matches: Vec::new(),
                },
            });
        }
        if new_group.trim_end_matches('!') != "none" {
            plans.insert(
                new_group.clone(),
                Plan {
                    credibility: 1.0,
                    verifiability: 1.0,
                    reshare_all: true,
                    values: new_supports
                        .iter()
                        .map(|(name, s)| {
                            (
                                name.clone(),
                                Values {
                                    support: *s,
                                    intensity: new_weight,
                                    mu: 0.0,
                                    projection: None,
                                },
                            )
                        })
                        .collect(),
                },
            );
            items.push(GroupItem {
                group_id: new_group.clone(),
                position: 1,
                source_key: new_group.clone(),
                members: vec![format!("e:{new_group}")],
                verifiable: true,
                content: GroupContent::Experience {
                    appraisals: Vec::new(),
                    matches: Vec::new(),
                },
            });
        }
        let group_list: Vec<_> = groups.values().cloned().collect();
        let before_snapshot = snapshot.clone();
        let before_groups = group_list.clone();
        let aliases = BTreeMap::new();
        let step = Step {
            revision: "r",
            candidates: &[],
            groups: &items,
            aliases: &aliases,
        };
        let result = crate::law::engine::execute(
            RState {
                snapshot: &snapshot,
                groups: &group_list,
            },
            &default_theta(),
            &step,
            &f.ctx(),
            |work, item, _| {
                let stored = work.groups.get(&item.group_id).cloned();
                let plan = plans.remove(&item.group_id).unwrap();
                crate::law::group::store_plan(
                    work,
                    item,
                    stored,
                    item.members.clone(),
                    plan,
                    &f.ctx(),
                )
            },
        );
        assert!(
            snapshot_bits_eq(&snapshot, &before_snapshot),
            "input R changed at {line_number}"
        );
        assert_eq!(group_list.len(), before_groups.len());
        assert!(
            group_list
                .iter()
                .zip(&before_groups)
                .all(|(a, b)| group_bits_eq(a, b)),
            "input groups changed at {line_number}"
        );
        let status = input.token().to_owned();
        if status == "fault" {
            let error = result.expect_err("reference event fault accepted");
            assert_eq!(
                fault_code(&error),
                input.token(),
                "fault event {line_number}: {error}; {line}"
            );
            assert!(
                input.0.next().is_none(),
                "extra fault data at {line_number}"
            );
            faults += 1;
            continue;
        }
        assert_eq!(status, "ok");
        let out = result.unwrap_or_else(|e| panic!("event {line_number}: {e}; {line}"));
        let changed = input.number();
        assert!(changed <= 1);
        let n = input.number();
        assert_eq!(out.counterfactual.len(), n, "cf count at {line_number}");
        for _ in 0..n {
            let id = input.id();
            let pi = input.float();
            assert_eq!(
                out.counterfactual[&id].to_bits(),
                pi.to_bits(),
                "cf event {line_number}: {id}"
            );
        }
        let mut expected = f.snapshot(input.states(&f, prior));
        if changed == 1 {
            expected.revision = step.revision.to_owned();
        }
        assert!(
            snapshot_bits_eq(&out.r, &expected),
            "state bits at event {line_number}: {line}"
        );
        for group in out.groups {
            groups.insert(group.group_id.clone(), group);
        }
        let expected_groups = input.groups();
        assert_eq!(groups.len(), expected_groups.len());
        for (id, want) in expected_groups {
            assert!(
                group_bits_eq(&groups[&id], &want),
                "allocation bits at event {line_number}: {line}"
            );
        }
        assert!(
            input.0.next().is_none(),
            "extra event data at {line_number}"
        );
        checked += 1;
    }
    assert!(child.0.wait().unwrap().success());
    assert!(checked > 6000 && faults > 0);
    println!(
        "Full event parity: {checked} unique events, {faults} atomic faults; zero R, allocation or cf mismatches"
    );
}
