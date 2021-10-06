use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use filemagic::Magic;
use goblin::Object;
use infer::Type;
use libc::{S_IRWXG, S_IRWXO, S_IRWXU, S_IXUSR};
use memmap::MmapOptions;

use crate::{bytes::Bytes, error::Result, group, lstat::Lstat, user};

#[non_exhaustive]
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

        let (mode, permission) = self.permissions();
        writeln!(f, "permissions: {} ({:o})", permission, mode)?;

        if let Some((uid, username)) = self.owner_user() {
            writeln!(f, "owner: {} ({})", username, uid)?;
        }

        if let Some((gid, group_name)) = self.owner_group() {
            writeln!(f, "owner's group: {} ({})", group_name, gid)?;
        }

        if self.is_executable() {
            if let Some(libraries) = self.read_dynamic_dependencies() {
                // Placeholder text, gotta change this later
                writeln!(f, "[libraries]")?;
                for library in libraries {
                    writeln!(f, "· {}", library)?;
                }
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
        let file = |magic: &Magic| magic.file(&self.path).ok();

        self.magic.as_ref().and_then(file)
    }

    /// Attempts to read the file's MIME type through the `infer` crate.
    /// This is used as a fallback method since getting this data through libmagic
    /// yields more information.
    pub fn fallback_mime_type(&self) -> Option<&str> {
        let mime = |t: Type| t.mime_type();

        infer::get_from_path(&self.path).ok()?.map(mime)
    }

    /// Returns the size of the file in a Display-capable struct
    pub fn size(&self) -> Bytes {
        Bytes::new(self.stat.size() as u64)
    }

    /// Returns the user id (uid) and username of the user that owns the file represented by `self`.
    pub fn owner_user(&self) -> Option<(u32, String)> {
        let user_id = self.stat.owner_user_id();

        user::get_username(user_id).map(|username| (user_id, username))
    }

    /// Returns the group id (gid) and group name of the group that contains the owner of the file represented by `self`.
    pub fn owner_group(&self) -> Option<(u32, String)> {
        let group_id = self.stat.owner_group_id();

        group::get_group_name(group_id).map(|group_name| (group_id, group_name))
    }

    /// Returns the raw permission bits of this file, alongside a `ls`-like representation of said file permissions.
    pub fn permissions(&self) -> (u32, String) {
        let mode = self.stat.mode();
        (
            mode & (S_IRWXU | S_IRWXG | S_IRWXO),
            unix_mode::to_string(mode),
        )
    }

    /// Returns true if this file is in the `application`
    pub fn is_application(&self) -> bool {
        let mime_type = self.fallback_mime_type().unwrap_or_default();

        //                          cation
        mime_type.starts_with("appli")
    }

    /// Returns true if this file has the executable bit turned on
    pub fn is_executable(&self) -> bool {
        (self.stat.mode() & S_IXUSR) != 0
    }

    /// If this file is a dynamically-linked binary, this file will attempt to retrieve the
    /// libraries on which the file depends. Note that, unlike tools like `ldd`, this function will only
    /// return the direct dependencies of a file. Dependencies of a dependency will not be included.
    pub fn read_dynamic_dependencies(&self) -> Option<Vec<String>> {
        let clone_libs = |libs: Vec<&str>| libs.into_iter().map(ToOwned::to_owned).collect();

        let get_libraries = |obj| -> Option<Vec<String>> {
            match obj {
                Object::Elf(elf) => Some(clone_libs(elf.libraries)),
                Object::PE(pe) => Some(clone_libs(pe.libraries)),
                _ => None,
            }
        };

        let file = fs::File::open(&self.path).ok()?;
        let map = unsafe { MmapOptions::new().map(&file) }.ok()?;

        Object::parse(&map).ok().and_then(get_libraries)
    }
}
