use std::ffi::NulError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("An interior NULL byte was found when creating a C string")]
    NullByte(#[from] NulError)
}

pub type Result<T> = std::result::Result<T, Error>;