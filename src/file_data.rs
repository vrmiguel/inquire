use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use filemagic::Magic;
use infer::Type;

use crate::{bytes::Bytes, dylib, error::Result, group, lstat::Lstat, user};

pub struct FileData {
    path: PathBuf,
    stat: Lstat,
    magic: Option<Magic>,
}

fn libmagic_display(mime_msg: String, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for line in mime_msg.split(',') {
        writeln!(f, "· {}", line.trim())?;
    }

    Ok(())
}

impl Display for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}]", self.path.display())?;

        if let Some(mime) = self.libmagic_mime_type() {
            libmagic_display(mime, f)?;
        } else if let Some(mime) = self.fallback_mime_type() {
            writeln!(f, "type: {}", mime)?;
        }

        writeln!(f, "size: {}", self.size())?;

        writeln!(f, "permissions: {}", self.permissions())?;

        if let Some(owner_user) = self.owner_user() {
            writeln!(f, "owner's username: {}", owner_user)?;
        }

        if let Some(owner_group) = self.owner_group() {
            writeln!(f, "owner's group: {}", owner_group)?;
        }

        if let Some(libraries) = dylib::read_dynamic_dependencies(&self.path) {
            // Placeholder text, gotta change this later
            writeln!(f, "[libraries]")?;
            for library in libraries {
                writeln!(f, "· {}", library)?;
            }
        }

        Ok(())
    }
}

impl FileData {
    /// ```rust
    /// use inquire::FileData;
    ///
    /// let cargo_toml = FileData::read("Cargo.toml").unwrap();
    /// println!("{}", cargo_toml.size());
    /// ```
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = fs::canonicalize(path)?;

        let init_magic = || {
            let magic = Magic::open(Default::default()).ok()?;
            magic.load::<String>(&[]).ok()?;

            Some(magic)
        };

        Ok(Self {
            stat: Lstat::lstat(&path)?,
            path,
            magic: init_magic(),
        })
    }

    /// Attempts to read the file's MIME type through libmagic.
    pub fn libmagic_mime_type(&self) -> Option<String> {
        if let Some(ref magic) = self.magic {
            return magic.file(&self.path).ok();
        }

        None
    }

    /// Attempts to read the file's MIME type through the `infer` crate.
    /// This is used as a fallback method since getting this data through libmagic
    /// yields more information.
    pub fn fallback_mime_type(&self) -> Option<&str> {
        let mime = |t: Type| t.mime_type();

        infer::get_from_path(&self.path).ok()?.map(mime)
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
