use std::path::PathBuf;

use infer::Type;

use crate::{bytes::Bytes, error::Result, lstat::Lstat, user};

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
        let mime_and_extension = |t: Type| (t.mime_type(), t.extension());

        Ok(infer::get_from_path(&self.path)?.map(mime_and_extension))
    }

    pub fn size(&self) -> Bytes {
        Bytes::new(self.stat.size() as u64)
    }

    pub fn owner_user(&self) -> Option<String> {
        let user_id = self.stat.owner_user_id();

        user::get_username(user_id)
    }
}
