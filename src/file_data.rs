use std::{fmt::Display, fs, path::{Path, PathBuf}};

use filemagic::{Magic, flags::Flags};
use infer::Type;

use crate::{bytes::Bytes, error::Result, group, lstat::Lstat, user};

pub struct FileData {
    path: PathBuf,
    stat: Lstat,
    magic: Option<Magic>
}

impl Display for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Path: `{0}`", self.path.display())?;
        
        if let Some(mime) = self.libmagic_mime_type() {
            writeln!(f, "Type: `{0}`", mime)?;
        } else {
            if let Ok(Some((mime, ext))) = self.mime_type() {
                writeln!(f, "Type: `{} {}`", mime, ext)?;
            }
        }

        writeln!(f, "Bytes: `{}`", self.size())?;

        writeln!(f, "Permissions: `{}`", self.permissions())?;

        if let Some(owner_user) = self.owner_user() {
            writeln!(f, "Owner's user: `{}`", owner_user)?;
        }

        if let Some(owner_group) = self.owner_group() {
            writeln!(f, "Owner's owner group: `{}`", owner_group)?;
        }

        Ok(())
    }
}

pub type MaybeMime<'a> = Option<(&'a str, &'a str)>;

impl FileData {
    pub fn read(path: PathBuf) -> Result<Self> {

        let path = fs::canonicalize(path)?;

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

    pub fn libmagic_mime_type(&self) -> Option<String> {
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
