mod ffi;
pub mod error;

use std::io;

pub use unixstring;
use bitflags::bitflags;
use libc::c_int;
use unixstring::UnixString;

use crate::error::{Error, Result};

bitflags! {
    struct Flags: c_int {
        const DEFAULT = 0x000000;
        /// Treat operating system errors while trying to open files and follow symlinks as real errors, instead of printing them in the magic buffer
        const ERROR   = 0x000200;
        const DEFAULT_WITH_ERRORS = Self::DEFAULT.bits | Self::ERROR.bits;
    }
}

pub struct Magic {
    inner: *const ffi::Magic
}

impl Drop for Magic {
    fn drop(&mut self) {
        unsafe {
            ffi::magic_close(self.inner);
        }
    }
}

impl Magic {

    fn io_err_from_errno() -> Error {
        Error::Io(io::Error::last_os_error())
    }

    pub fn new() -> Result<Magic> {
        let inner = unsafe {
            ffi::magic_open(Flags::DEFAULT_WITH_ERRORS.bits())
        };

        if inner.is_null() {
            return Err(Self::io_err_from_errno());
        }

        let ret = unsafe {
            ffi::magic_load(inner, std::ptr::null())
        };

        if ret != 0 {
            return Err(Self::io_err_from_errno());
        }

        Ok(Self {
            inner
        })
    }

    pub fn file(&self, path: &UnixString) -> Result<String> {

        let description = unsafe {
            ffi::magic_file(self.inner, path.as_ptr())
        };

        if description.is_null() {
            return Err(Error::Io(io::Error::last_os_error()));
        }

        let description = unsafe { UnixString::from_ptr(description) };

        Ok(description.into_string_lossy())
    }
}