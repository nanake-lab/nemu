//! Frozen world assumptions from PREREGISTRATION-V0.2 §§2,5,6.
use serde::{Deserialize, Serialize};
pub const DAY: u64 = 86_400_000_000_000;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub ordinal: u64,
    pub day: u32,
    pub time: u64,
    pub hours: u32,
    pub person: String,
    pub context: String,
    pub a: [f64; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub id: String,
    pub world: String,
    pub seed: u64,
    pub density: u32,
    pub hours: u32,
    pub days: u32,
    pub status: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Probe {
    pub id: String,
    pub person: String,
    pub context: String,
    pub a: [f64; 4],
}
pub const CONTEXTS: [&str; 7] = [
    "contact", "task", "explore", "empathy", "play", "pattern", "secret",
];
pub fn contexts() -> Vec<&'static str> {
    let mut v = CONTEXTS.to_vec();
    v.push("punctual");
    v.push("feedback");
    v
}
pub fn channel(c: &str) -> usize {
    match c {
        "task" => 3,
        "explore" | "pattern" => 2,
        "empathy" => 1,
        _ => 0,
    }
}
pub fn ids(person: &str, context: &str) -> Vec<String> {
    if context == "feedback" {
        return vec![
            format!("{person}:feedback:0:positive"),
            format!("{person}:feedback:0:available"),
            format!("{person}:feedback:0:negative"),
            format!("{person}:feedback:0:available-negative"),
        ];
    }
    let channels = if context == "play" {
        vec![0, 3]
    } else {
        vec![channel(context)]
    };
    channels
        .into_iter()
        .flat_map(|i| {
            [
                format!("{person}:{context}:{i}:positive"),
                format!("{person}:{context}:{i}:negative"),
            ]
        })
        .collect()
}
pub fn all_ids() -> Vec<String> {
    ["U", "V"]
        .into_iter()
        .flat_map(|p| contexts().into_iter().flat_map(move |c| ids(p, c)))
        .collect()
}
pub fn runs() -> Vec<Run> {
    let mut out = vec![];
    for seed in [53001, 53002] {
        for hours in [2, 8, 24] {
            for world in [
                "W0",
                "W1",
                "W2",
                "W3",
                "W4",
                "W5",
                "W6",
                "W7",
                "W9-nonzero",
                "W9-zero",
            ] {
                for density in [1, 4, 12] {
                    if (world == "W5" || world.starts_with("W9")) && density != 4 {
                        continue;
                    }
                    out.push(Run {
                        id: format!("{world}-s{seed}-n{density}-h{hours}"),
                        world: world.into(),
                        seed,
                        density,
                        hours,
                        days: 365,
                        status: "ready".into(),
                        reason: String::new(),
                    });
                }
            }
        }
    }
    for seed in [53001, 53002] {
        for c in ["positive", "negative"] {
            for k in ["0", "025", "050"] {
                for perturb in ["none", "temporary", "permanent"] {
                    out.push(Run {
                        id: format!("O5-{c}-k{k}-{perturb}-s{seed}"),
                        world: format!("O5-{c}-k{k}-{perturb}"),
                        seed,
                        density: 4,
                        hours: 8,
                        days: 1095,
                        status: "ready".into(),
                        reason: String::new(),
                    });
                }
            }
        }
    }
    for seed in [53001, 53002] {
        for world in [
            "W8-30",
            "W8-365",
            "W8-1095",
            "W8-three-nights",
            "W8-trip",
            "W8-fading",
            "W8-competing",
            "O16",
            "O17",
            "autonomous-feedback",
        ] {
            out.push(Run {
                id: format!("{world}-s{seed}"),
                world: world.into(),
                seed,
                density: 4,
                hours: 8,
                days: match world {
                    "W8-365" => 455,
                    "W8-1095" => 1185,
                    "W8-fading" => 180,
                    "O16" | "O17" => 365,
                    "autonomous-feedback" => 1095,
                    _ => 120,
                },
                status: "not_run".into(),
                reason: match world {
                    "O16" | "O17" => {
                        "ADR 0038 recall curve/age/target-update implementation unavailable"
                    }
                    "autonomous-feedback" => {
                        "approved action-readout to external-response adapter unavailable"
                    }
                    _ => {
                        "V2-77/78 absence qualification and same-absence C-once adapter unavailable"
                    }
                }
                .into(),
            });
        }
    }
    out
}
// Explicit xorshift64* and Fisher-Yates; never uses library default RNG.
fn next(s: &mut u64) -> u64 {
    *s ^= *s >> 12;
    *s ^= *s << 25;
    *s ^= *s >> 27;
    s.wrapping_mul(0x2545_f491_4f6c_dd1d)
}
pub fn events(run: &Run) -> Vec<Event> {
    if run.world == "W5" || run.status != "ready" {
        return vec![];
    }
    let templates = [
        "contact", "contact", "task", "task", "explore", "explore", "empathy", "empathy", "play",
        "pattern", "secret", "punctual",
    ];
    let mut rng = run.seed;
    let mut order: Vec<usize> = (0..12).collect();
    let mut out = vec![];
    for ordinal in 0..u64::from(run.days * run.density) {
        let block = ordinal / 12;
        let slot = (ordinal % 12) as usize;
        if slot == 0 {
            order = (0..12).collect();
            for i in (1..12).rev() {
                let j = (next(&mut rng) % (i as u64 + 1)) as usize;
                order.swap(i, j);
            }
        }
        let day = (ordinal / u64::from(run.density)) as u32 + 1;
        let time = u64::from(day - 1) * DAY
            + (ordinal % u64::from(run.density) + 1) * u64::from(run.hours) * 3_600_000_000_000
                / u64::from(run.density + 1);
        let mut c = templates[order[slot]];

        let bad = match run.world.as_str() {
            "W1" => true,
            "W2" => day > 180,
            "W3" => day > 270,
            "W4" => (day - 1) / 30 % 2 == 1,
            _ => false,
        };
        let positive = if run.world == "W3" && (181..=270).contains(&day) {
            block % 4 < 2
        } else if bad {
            block % 4 == 0
        } else {
            block % 4 < 3
        };
        let mut sign = if positive { 1. } else { -1. };
        let person = if (block + 1) % 7 == 0 && matches!(c, "contact" | "secret" | "punctual") {
            "V"
        } else {
            "U"
        };
        let mut person = person;
        if matches!(run.world.as_str(), "W6" | "W7") {
            person = "U";
            sign = if ordinal % 2 == 0 { 1. } else { -1. };
            c = if (ordinal % 2 == 0) == (run.world == "W6") {
                "secret"
            } else {
                "punctual"
            };
        }
        let mut a = [0.; 4];
        a[channel(c)] = sign * 0.25;
        if c == "explore" && !positive {
            a = [0.; 4];
        }
        if c == "play" {
            a[3] = sign * 0.25;
        }
        if run.world == "W9-zero" && matches!(c, "play" | "pattern") {
            a = [0.; 4];
        }
        if run.world.starts_with("O5-") {
            c = "feedback";
            person = "U";
        }
        out.push(Event {
            ordinal: ordinal + 1,
            day,
            time,
            hours: run.hours,
            person: person.into(),
            context: c.into(),
            a,
        });
    }
    out
}
pub fn probes() -> Vec<Probe> {
    let mut out = vec![];
    for i in 0..4 {
        for sign in [-1., 1.] {
            let mut a = [0.; 4];
            a[i] = sign * 0.25;
            out.push(Probe {
                id: format!("neutral-{i}-{sign}"),
                person: String::new(),
                context: String::new(),
                a,
            });
        }
    }
    for p in ["U", "V"] {
        for c in contexts() {
            for sign in [-1., 1.] {
                let mut a = [0.; 4];
                a[channel(c)] = sign * 0.25;
                if c == "play" {
                    a[3] = sign * 0.25;
                }
                out.push(Probe {
                    id: format!("{p}-{c}-{sign}"),
                    person: p.into(),
                    context: c.into(),
                    a,
                });
            }
        }
    }
    out
}
