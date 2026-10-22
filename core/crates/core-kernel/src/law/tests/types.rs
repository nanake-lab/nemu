//! What G may read and which types R uses: the I10 signature check (no x, m,
//! g or λ) and golden rule 2 (R only in the IDL types of core-types).

use crate::law::{Law, LawContext, LawResult, Output, RState, Step, ThetaView, g};

#[test]
fn i10_g_signature_has_no_x_m_g_or_lambda() {
    // The parameter list is pinned: R, the θ view, the step, the law context.
    let _pinned: fn(RState<'_>, &ThetaView, &Step<'_>, &LawContext<'_>) -> LawResult<Output> = g;
    // No law source names the fast state, the modulation, the gains or the
    // recovery rates (comments excluded); the θ view holds b and σ̂ only.
    let sources = [
        ("engine.rs", include_str!("../engine.rs")),
        ("group.rs", include_str!("../group.rs")),
        ("input.rs", include_str!("../input.rs")),
        ("fuse.rs", include_str!("../fuse.rs")),
        ("num.rs", include_str!("../num.rs")),
        ("params.rs", include_str!("../params.rs")),
        ("plan.rs", include_str!("../plan.rs")),
        ("recurrence.rs", include_str!("../recurrence.rs")),
        ("relation.rs", include_str!("../relation.rs")),
        ("replay.rs", include_str!("../replay.rs")),
        ("shares.rs", include_str!("../shares.rs")),
        ("support.rs", include_str!("../support.rs")),
        ("error.rs", include_str!("../error.rs")),
        ("mod.rs", include_str!("../mod.rs")),
    ];
    let forbidden = [
        "FastStateSnapshot",
        "LearnedComponent",
        "ModulationEnvelope",
        "ThetaValues",
        "gain",
        "recovery",
        "lambda",
        "fast_state",
        "modulation",
    ];
    for (name, src) in sources {
        for (n, line) in src.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            for word in forbidden {
                assert!(
                    !code.contains(word),
                    "{name}:{}: `{word}` in `{line}`",
                    n + 1
                );
            }
        }
    }
    let input = include_str!("../input.rs");
    let start = input.find("pub struct ThetaView {").unwrap();
    let body = &input[start..start + input[start..].find('}').unwrap()];
    let fields: Vec<&str> = body
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(fields, ["baseline: Nature4,", "sigma: Nature4,"]);
    let _ = Law::new; // the law context is built from the law version only
}

#[test]
fn law_defines_no_stored_or_cross_process_type() {
    // Golden rule 2: R, its group summaries and the L payloads are the IDL
    // types of core-types (P5-32, ADR 0016); the structs of `law/` bundle them
    // for one call and never serialize.
    let sources = [
        include_str!("../engine.rs"),
        include_str!("../error.rs"),
        include_str!("../fuse.rs"),
        include_str!("../group.rs"),
        include_str!("../input.rs"),
        include_str!("../mod.rs"),
        include_str!("../num.rs"),
        include_str!("../params.rs"),
        include_str!("../plan.rs"),
        include_str!("../recurrence.rs"),
        include_str!("../relation.rs"),
        include_str!("../replay.rs"),
        include_str!("../shares.rs"),
        include_str!("../support.rs"),
    ];
    for src in sources {
        let code: String = src
            .lines()
            .map(|l| l.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in [
            "Serialize",
            "Deserialize",
            "serde",
            "WireMessage",
            "canonical_bytes",
            "struct Tendency",
            "struct EvidenceGroup",
            "struct EvidenceAllocation",
            "struct RSnapshot",
            "struct TendencyChange",
        ] {
            assert!(!code.contains(forbidden), "law/ defines `{forbidden}`");
        }
    }
    let engine = include_str!("../engine.rs");
    assert!(engine.contains(
        "use core_types::generated::{EvidenceGroupV2, RSnapshotV2, TendencyChange, TendencyV2};"
    ));
    let input = include_str!("../input.rs");
    assert!(input.contains("pub snapshot: &'a RSnapshotV2,"));
    assert!(input.contains("pub groups: &'a [EvidenceGroupV2],"));
}
