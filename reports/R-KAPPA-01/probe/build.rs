use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let fast = root
        .join("../../../core/crates/core-kernel/src/fast")
        .canonicalize()
        .unwrap();
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let source = fs::read_to_string(fast.join("drive.rs")).unwrap();
    let needle = "let c = class_weight(input.class, law.kappa);";
    assert_eq!(
        source.matches(needle).count(),
        1,
        "baseline changed; review the intervention"
    );
    // The entire research intervention: bypass the class multiplier ONLY in
    // this generated copy. check_law, source enum, n and all bound checks stay.
    fs::write(
        output.join("drive.rs"),
        source.replace(needle, "let c = 1.0_f64;"),
    )
    .unwrap();
    let module = format!(
        "#[allow(dead_code)]\npub mod ablated {{\n#[path={:?}] pub mod types;\n#[path={:?}] pub mod space;\n#[path={:?}] pub mod drive;\n}}\n",
        fast.join("types.rs"),
        fast.join("space.rs"),
        output.join("drive.rs")
    );
    fs::write(output.join("ablated.rs"), module).unwrap();
    for file in ["drive.rs", "space.rs", "types.rs"] {
        println!("cargo:rerun-if-changed={}", fast.join(file).display());
    }
}
