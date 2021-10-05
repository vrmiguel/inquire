use std::path::PathBuf;

use crate::{bytes::Bytes, error::Result, lstat::Lstat};

pub struct FileData {
    path: PathBuf,
    stat: Lstat,
}

pub type MaybeMime<'a> = Option<(&'a str, &'a str)>;

impl FileData {
    pub fn read(path: PathBuf) -> Result<Self> {
        Ok(Self {
            stat: Lstat::lstat(&path)?,
            path,
        })
    }

    pub fn mime_type(&self) -> Result<MaybeMime> {
        Ok(infer::get_from_path(&self.path)?.map(|t| (t.mime_type(), t.extension())))
    }

    pub fn size(&self) -> Bytes {
        Bytes::new(self.stat.size() as u64)
    }
}
