//! Only unsafe boundary: statically linked, version-pinned SQLite extension.
use crate::{Error, Result};
pub fn register() -> Result<()> {
    // SAFETY: sqlite-vec's documented C entry point has SQLite's extension ABI.
    // It is linked for the process lifetime and registers only with the same
    // bundled libsqlite3 used by rusqlite; no user-supplied DLL or path is loaded.
    let code = unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *mut std::ffi::c_char,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> std::ffi::c_int,
        >(
            sqlite_vec::sqlite3_vec_init as *const ()
        )))
    };
    if code != rusqlite::ffi::SQLITE_OK {
        return Err(Error::Invalid("sqlite-vec registration failed".into()));
    }
    Ok(())
}
