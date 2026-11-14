//! P5-71 preregistration v2 §8: public-API resource probe, never production wiring.
#![forbid(unsafe_code)]
#[path = "p571/adapter.rs"]
mod adapter;

#[test]
fn p571_arithmetic_boundaries() {
    let status = std::process::Command::new("python")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/p571/arithmetic.py"
        ))
        .status()
        .expect("Python required by P5-71 independent arithmetic checks");
    assert!(status.success());
}

#[test]
fn p571_probe_tooling_one_step() {
    let status = std::process::Command::new("python")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/p571/smoke.py"))
        .arg(std::env::current_exe().unwrap())
        .status()
        .expect("Python required by P5-71 tooling smoke");
    assert!(status.success());
}

#[test]
fn p571_full_tooling_boundaries() {
    let status = std::process::Command::new("python")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/p571/smoke_full.py"
        ))
        .arg(std::env::current_exe().unwrap())
        .status()
        .expect("Python required by P5-71 full adapter tooling checks");
    assert!(status.success());
}

#[test]
fn p571_public_bridge() {
    // The external research driver opts into line-oriented I/O. Normal CI still
    // builds the complete adapter and checks its pure primitive dispatch.
    if std::env::var_os("P571_RPC").is_none() {
        assert_eq!(
            adapter::dispatch(&serde_json::json!({"op":"libm", "function":"sqrt", "x":1.0}))["value"],
            1.0
        );
        return;
    }
    use std::io::{BufRead, Write};
    for line in std::io::stdin().lock().lines() {
        let request = serde_json::from_str(&line.unwrap()).unwrap();
        let result = adapter::dispatch(&request);
        println!("P571:{}", serde_json::to_string(&result).unwrap());
        std::io::stdout().flush().unwrap();
    }
}
