use sha1::{Digest, Sha1};
use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, ptr};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, INVALID_HANDLE_VALUE},
    System::Memory::{
        CreateFileMappingW,
        FILE_MAP_WRITE,
        MEMORY_MAPPED_VIEW_ADDRESS,
        MapViewOfFile,
        PAGE_READWRITE,
        UnmapViewOfFile,
    },
};

/// Generate memory name string
fn qt_memory_name(key: &str) -> String {
    let letters: String = key.chars().filter(|c| c.is_ascii_alphabetic()).collect();

    let hash: String = Sha1::digest(key.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();

    format!("qipc_sharedmemory_{letters}{hash}")
}

pub struct PortPublication {
    handle: HANDLE,
    view: *mut u8,
}

/// Port Publication mechanism
impl PortPublication {
    pub fn new(port: u16, key: &str) -> io::Result<Self> {
        let name = qt_memory_name(key);
        let wide_name: Vec<u16> = OsStr::new(&name).encode_wide().chain(Some(0)).collect();

        let handle = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                64,
                wide_name.as_ptr(),
            )
        };

        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }

        let error = unsafe { GetLastError() };

        if error == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "ServerPort already assigned",
            ));
        }

        let view = unsafe { MapViewOfFile(handle, FILE_MAP_WRITE, 0, 0, 64) }.Value as *mut u8;

        if view.is_null() {
            let error = io::Error::last_os_error();
            unsafe { CloseHandle(handle) };
            return Err(error);
        }

        let publication = Self { handle, view };

        let payload = port.to_string();
        unsafe {
            ptr::copy_nonoverlapping(payload.as_ptr(), publication.view, payload.len());
            *publication.view.add(payload.len()) = 0;
        }

        Ok(publication)
    }
}

impl Drop for PortPublication {
    fn drop(&mut self) {
        unsafe {
            if !self.view.is_null() {
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: self.view as _,
                });
                self.view = ptr::null_mut();
            }

            if !self.handle.is_null() {
                CloseHandle(self.handle);
                self.handle = ptr::null_mut();
            }
        }
    }
}
