#![forbid(unsafe_code)]
fn main() {
    eprintln!(
        "echoctl is a P0-01 scaffold. inspect/replay/verify/checkpoint/restore/birth are not implemented; no storage was opened."
    );
    std::process::exit(2);
}
