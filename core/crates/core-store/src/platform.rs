//! Win32 calls are confined here. No privilege escalation or fallback filesystem.
use crate::{Result, error::require};
use std::{
    fs::{self, File, OpenOptions},
    path::Path,
};

pub(crate) fn no_links(path: &Path) -> Result<()> {
    for part in path.ancestors() {
        if part.exists() {
            let m = fs::symlink_metadata(part)?;
            require(
                !m.file_type().is_symlink(),
                "symbolic links are not storage roots",
            )?;
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                require(
                    m.file_attributes() & 0x400 == 0,
                    "reparse/sync directories are not storage roots",
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FlushFileBuffers needs GENERIC_WRITE. BACKUP_SEMANTICS opens a directory;
        // it does not grant privileges. Errors are fatal, never silently ignored.
        OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(0x0200_0000)
            .open(path)?
            .sync_all()?;
    }
    #[cfg(not(windows))]
    File::open(path)?.sync_all()?;
    Ok(())
}

/// Publish a synced file in the same directory without replacing any target.
/// Windows requests the documented write-through move in addition to subsequent
/// file/directory flushes. No COPY_ALLOWED or REPLACE_EXISTING fallback is used.
pub(crate) fn persist_noclobber(temp: tempfile::NamedTempFile, target: &Path) -> Result<()> {
    require(
        temp.path().parent() == target.parent(),
        "object publication must stay in one directory",
    )?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_NORMAL, MOVEFILE_WRITE_THROUGH, MoveFileExW, SetFileAttributesW,
        };
        let source = temp.into_temp_path();
        let from: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let to: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: the live source buffer is NUL-terminated. tempfile creates
        // FILE_ATTRIBUTE_TEMPORARY files; clear that caching hint before publish,
        // matching tempfile's own persist/keep behavior. Failure is fatal.
        if unsafe { SetFileAttributesW(from.as_ptr(), FILE_ATTRIBUTE_NORMAL) } == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        // SAFETY: both path buffers are live and NUL-terminated for the entire
        // call. Flags exclude replacement, cross-volume copy and delayed moves.
        let ok = unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_WRITE_THROUGH) };
        if ok == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    #[cfg(not(windows))]
    temp.persist_noclobber(target)
        .map_err(|e| crate::Error::Io(e.error))?;
    Ok(())
}

/// Own the lock lifetime, not just one descriptor's lifetime. On Unix an
/// in-flight fork can retain the same open file description until exec/exit.
/// Closing our descriptor alone would leave its flock held by that child.
pub(crate) struct WriterLock(File);
impl Drop for WriterLock {
    fn drop(&mut self) {
        if let Err(error) = self.0.unlock() {
            // Drop cannot return an error. Keep the original error observable;
            // closing the descriptor follows, and a later open still fails closed.
            tracing::error!(?error, "authority writer unlock failed");
        }
    }
}

pub(crate) fn writer_lock(root: &Path) -> Result<WriterLock> {
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join("writer.lock"))?;
    lock.try_lock().map_err(crate::Error::WriterLock)?;
    Ok(WriterLock(lock))
}

#[cfg(test)]
mod tests;

#[cfg(windows)]
pub(crate) fn local_volume(path: &Path, _temporary: bool) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW,
    };
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut volume = vec![0u16; 32768];
    let mut format = [0u16; 32];
    // SAFETY: NUL-terminated input and writable buffers of the supplied lengths;
    // null optional output pointers are explicitly supported by these Win32 APIs.
    let ok = unsafe { GetVolumePathNameW(wide.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) };
    if ok == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    // SAFETY: volume is the NUL-terminated output of GetVolumePathNameW.
    let kind = unsafe { GetDriveTypeW(volume.as_ptr()) };
    require(
        kind == 3,
        "authority storage requires a local fixed NTFS volume",
    )?;
    // SAFETY: valid NUL-terminated volume and a bounded writable filesystem-name buffer.
    let ok = unsafe {
        GetVolumeInformationW(
            volume.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            format.as_mut_ptr(),
            format.len() as u32,
        )
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let end = format.iter().position(|c| *c == 0).unwrap_or(format.len());
    require(
        String::from_utf16_lossy(&format[..end]) == "NTFS",
        "authority storage requires NTFS",
    )
}

#[cfg(not(windows))]
pub(crate) fn local_volume(_path: &Path, temporary: bool) -> Result<()> {
    require(
        temporary,
        "production storage is supported only on Windows NTFS",
    )
}
