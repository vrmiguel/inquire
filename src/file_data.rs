use std::path::{Path, PathBuf};

use filemagic::{Magic, flags::Flags};
use infer::Type;

use crate::{bytes::Bytes, error::Result, group, lstat::Lstat, user};

pub struct FileData {
    path: PathBuf,
    stat: Lstat,
    magic: Option<Magic>
}

pub type MaybeMime<'a> = Option<(&'a str, &'a str)>;

impl FileData {
    pub fn read(path: PathBuf) -> Result<Self> {

        let init_magic = || {
            let magic = Magic::open(Default::default()).ok()?;
            magic.load::<String>(&[]).ok()?;

            Some(magic)
        };


        Ok(Self {
            stat: Lstat::lstat(&path)?,
            path,
            magic: init_magic()
        })
    }

    pub fn mime_type(&self) -> Result<MaybeMime> {
        let mime_and_extension = |t: Type| (t.mime_type(), t.extension());

        Ok(infer::get_from_path(&self.path)?.map(mime_and_extension))
    }

    pub fn mime_type2(&self) -> Option<String> {
        if let Some(ref magic) = self.magic {
            return magic.file(&self.path).ok();
        }

        None
    }

    pub fn size(&self) -> Bytes {
        Bytes::new(self.stat.size() as u64)
    }

    pub fn owner_user(&self) -> Option<String> {
        let user_id = self.stat.owner_user_id();

        user::get_username(user_id)
    }

    pub fn owner_group(&self) -> Option<String> {
        let group_id = self.stat.owner_group_id();

        group::get_group_name(group_id)
    }

    pub fn permissions(&self) -> String {
        unix_mode::to_string(self.stat.mode())
    }
}
