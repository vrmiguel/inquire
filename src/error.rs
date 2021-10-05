use std::ffi::NulError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // TODO: check errno when this happens and subdivide the errors
    #[error("lstat failed")]
    Lstat,
    #[error("Internal zero byte found during CString construction")]
    InternalNulByte(#[from] NulError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("libmagic error: {0}")]
    Magic(#[from] filemagic::FileMagicError)
}

pub type Result<T> = std::result::Result<T, Error>;
