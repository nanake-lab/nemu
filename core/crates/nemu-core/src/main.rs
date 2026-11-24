#![deny(unsafe_code)]

#[cfg(windows)]
mod authority;
#[cfg(windows)]
mod deployment;
#[cfg(windows)]
mod host;
#[cfg(windows)]
mod logging;
#[cfg(windows)]
#[allow(unsafe_code)]
mod signal;

#[cfg(windows)]
fn main() -> std::process::ExitCode {
    match host::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nemu-core: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("nemu-core 只支持 Windows");
    std::process::ExitCode::FAILURE
}
