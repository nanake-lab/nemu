use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};

static STOP: AtomicBool = AtomicBool::new(false);

extern "system" fn handle(event: u32) -> i32 {
    if matches!(event, CTRL_BREAK_EVENT | CTRL_C_EVENT) {
        STOP.store(true, Ordering::Release);
        return 1;
    }
    0
}

pub fn install() -> std::io::Result<()> {
    // SAFETY: handle has the required Win32 ABI, stays valid for the entire
    // process, and only signals an atomic flag from the console control thread.
    if unsafe { SetConsoleCtrlHandler(Some(handle), 1) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

pub fn requested() -> bool {
    STOP.load(Ordering::Acquire)
}
