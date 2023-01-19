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

pub unsafe fn uninitialized<T>() -> T {
    MaybeUninit::uninit().assume_init()
}
