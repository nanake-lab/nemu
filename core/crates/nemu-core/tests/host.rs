#[cfg(windows)]
#[test]
fn isolated_windows_terminal() {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/terminal.py");
    let status = std::process::Command::new("python")
        .args(["-B", "-I"])
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_nemu-core"))
        .status()
        .expect("launch the repository Python test terminal");
    assert!(status.success());
}

#[cfg(not(windows))]
#[test]
fn reports_unsupported_platform() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_nemu-core"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("只支持 Windows"));
}
