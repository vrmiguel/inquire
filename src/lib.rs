mod bytes;
mod error;
mod file_data;
mod group;
mod lstat;
mod time_fmt;
mod user;

use std::mem::MaybeUninit;

pub use error::{Error, Result};
pub use file_data::FileData;

/// Reserves enough stack space for an element of type T
/// without properly initializing it with anything.
/// 
/// Analogous to the now deprecated [`std::mem::uninitialized`].
/// 
/// # Safety
/// 
/// Caller must ensure that the 
#[allow(clippy::uninit_assumed_init)]
pub unsafe fn uninitialized<T>() -> T {
    MaybeUninit::uninit().assume_init()
}
