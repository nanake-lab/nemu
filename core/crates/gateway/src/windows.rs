//! All unsafe Win32 boundaries live here. Each call documents pointer ownership.
use crate::{Error, Result};
use std::{
    ffi::c_void,
    fs::File,
    io::{Read, Write},
    os::windows::{
        ffi::OsStrExt,
        fs::MetadataExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree},
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        },
        Cryptography::{
            BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom, CRYPT_INTEGER_BLOB,
            CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
        GetTokenInformation, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
    },
    Storage::FileSystem::{
        CREATE_NEW, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, OPEN_EXISTING,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};
use zeroize::{Zeroize, Zeroizing};

fn last_error() -> Error {
    std::io::Error::last_os_error().into()
}
fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
    value.encode_wide().chain(Some(0)).collect()
}

struct Local(*mut c_void);
impl Drop for Local {
    fn drop(&mut self) {
        // SAFETY: only LocalAlloc-family allocations returned by Windows enter Local.
        unsafe {
            LocalFree(self.0);
        }
    }
}
pub(crate) struct Descriptor(Local);
impl Descriptor {
    pub(crate) fn current_user() -> Result<Self> {
        let sddl = wide(std::ffi::OsStr::new(&format!(
            "D:P(A;;GA;;;{})",
            current_sid()?
        )));
        let mut descriptor = ptr::null_mut();
        // SAFETY: sddl is terminated; Windows allocates the descriptor, owned by Local.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(last_error());
        }
        Ok(Self(Local(descriptor)))
    }
    pub(crate) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0.0,
            bInheritHandle: 0,
        }
    }
}
pub fn current_sid() -> Result<String> {
    let mut token = ptr::null_mut();
    // SAFETY: pseudo-process handle is borrowed; token receives an owned real handle.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(last_error());
    }
    // SAFETY: successful OpenProcessToken returned an exclusively owned handle.
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut size = 0;
    // SAFETY: null buffer with zero size is the documented size-query pattern.
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            ptr::null_mut(),
            0,
            &mut size,
        );
    }
    if size < size_of::<TOKEN_USER>() as u32 {
        return Err(last_error());
    }
    // usize storage supplies alignment for TOKEN_USER and its SID buffer.
    let mut storage = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
    // SAFETY: storage is aligned, writable and at least size bytes long.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            storage.as_mut_ptr().cast(),
            size,
            &mut size,
        )
    } == 0
    {
        return Err(last_error());
    }
    // SAFETY: successful query initialized TOKEN_USER in storage, whose lifetime covers conversion.
    let sid = unsafe { (&*storage.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut text = ptr::null_mut();
    // SAFETY: sid points into the live query buffer; text is Windows-allocated on success.
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(last_error());
    }
    let allocated = Local(text.cast());
    let mut length = 0;
    // SAFETY: ConvertSidToStringSidW returns a terminated UTF-16 string.
    while unsafe { *text.add(length) } != 0 {
        length += 1;
    }
    // SAFETY: the initialized string contains length UTF-16 units and remains allocated.
    let result = String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
        .map_err(|_| Error::Rejected("invalid Windows SID"));
    drop(allocated);
    result
}
pub(crate) fn random(bytes: &mut [u8]) -> Result<()> {
    let length = u32::try_from(bytes.len()).map_err(|_| Error::Rejected("random length"))?;
    // SAFETY: writable byte buffer; system-preferred RNG takes a null algorithm handle.
    let status = unsafe {
        BCryptGenRandom(
            ptr::null_mut(),
            bytes.as_mut_ptr(),
            length,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };
    if status < 0 {
        return Err(Error::Rejected("Windows random generation failed"));
    }
    Ok(())
}
pub(crate) fn protect(bytes: &[u8], decrypt: bool) -> Result<Zeroizing<Vec<u8>>> {
    // DPAPI's C signature is mutable but documents input as borrowed. Use a
    // zeroizing owned copy instead of casting away an immutable Rust borrow.
    let mut input = Zeroizing::new(bytes.to_vec());
    let blob = CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(input.len()).map_err(|_| Error::Contract)?,
        pbData: input.as_mut_ptr(),
    };
    let mut out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: ptr::null_mut(),
    };
    // SAFETY: all buffers outlive the calls. No optional UI/entropy/description;
    // LOCAL_MACHINE is deliberately absent, binding encryption to this user.
    let success = unsafe {
        if decrypt {
            CryptUnprotectData(
                &blob,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        } else {
            CryptProtectData(
                &blob,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        }
    };
    if success == 0 {
        return Err(last_error());
    }
    if out.pbData.is_null() {
        return Err(Error::Rejected("invalid DPAPI output"));
    }
    let allocated = Local(out.pbData.cast());
    // SAFETY: successful DPAPI returns an allocation of cbData bytes.
    let buffer = unsafe { std::slice::from_raw_parts_mut(out.pbData, out.cbData as usize) };
    let result = Zeroizing::new(buffer.to_vec());
    buffer.zeroize();
    drop(allocated);
    Ok(result)
}
fn no_links(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        match std::fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 => {
                return Err(Error::Rejected("credential path cannot contain links"));
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && ancestor == path => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
pub(crate) fn create_secret_file(path: &Path, bytes: &[u8]) -> Result<()> {
    no_links(path)?;
    let descriptor = Descriptor::current_user()?;
    let attributes = descriptor.attributes();
    let path = wide(path.as_os_str());
    // SAFETY: path/attributes/descriptor live through creation; CREATE_NEW never
    // replaces an existing installation token. Noninheritable, private DACL.
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_WRITE,
            0,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(last_error());
    }
    // SAFETY: CreateFileW returned an owned file handle; File closes it once.
    let mut file = unsafe { File::from_raw_handle(handle) };
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
pub(crate) fn read_secret_file(path: &Path) -> Result<Zeroizing<Vec<u8>>> {
    no_links(path)?;
    let path = wide(path.as_os_str());
    // SAFETY: valid terminated path, borrowed only during the call. Deny writers
    // while reading; OPEN_REPARSE_POINT and metadata reject final-component links.
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_OPEN_REPARSE_POINT,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(last_error());
    }
    // SAFETY: successful CreateFileW transfers this handle to File.
    let mut file = unsafe { File::from_raw_handle(handle) };
    let meta = file.metadata()?;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || !meta.is_file()
        || meta.len() > 16384
    {
        return Err(Error::Rejected("invalid credential file"));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub(crate) fn create_pipe(
    name: &str,
    config: crate::framing::FrameConfig,
) -> Result<tokio::net::windows::named_pipe::NamedPipeServer> {
    config.validate()?;
    let descriptor = Descriptor::current_user()?;
    let mut attributes = descriptor.attributes();
    // SAFETY: Tokio only borrows SECURITY_ATTRIBUTES during CreateNamedPipeW;
    // both it and the descriptor remain alive. No default/NULL DACL is used.
    Ok(unsafe {
        tokio::net::windows::named_pipe::ServerOptions::new()
            .first_pipe_instance(true)
            .reject_remote_clients(true)
            .max_instances(1)
            .in_buffer_size(4096)
            .out_buffer_size(4096)
            .create_with_security_attributes_raw(
                name,
                (&mut attributes as *mut SECURITY_ATTRIBUTES).cast(),
            )?
    })
}

#[cfg(test)]
pub(crate) fn dacl(handle: std::os::windows::io::RawHandle) -> Result<String> {
    use windows_sys::Win32::Security::{
        Authorization::{
            ConvertSecurityDescriptorToStringSecurityDescriptorW, GetSecurityInfo, SE_KERNEL_OBJECT,
        },
        DACL_SECURITY_INFORMATION,
    };
    let mut sd = ptr::null_mut();
    // SAFETY: handle is a live borrowed kernel handle; Windows allocates sd.
    let error = unsafe {
        GetSecurityInfo(
            handle,
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut sd,
        )
    };
    if error != 0 {
        return Err(std::io::Error::from_raw_os_error(error as i32).into());
    }
    let descriptor = Local(sd);
    let mut text = ptr::null_mut();
    // SAFETY: descriptor is valid; Windows allocates the output string.
    if unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(last_error());
    }
    let allocated = Local(text.cast());
    let mut length = 0;
    // SAFETY: conversion returns a terminated UTF-16 string.
    while unsafe { *text.add(length) } != 0 {
        length += 1;
    }
    // SAFETY: initialized string remains allocated until after conversion.
    let result = String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
        .map_err(|_| Error::Rejected("invalid DACL string"));
    drop(allocated);
    drop(descriptor);
    result
}
