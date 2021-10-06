use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use filemagic::Magic;
use goblin::Object;
use infer::Type;
use libc::S_IXUSR;

use crate::{bytes::Bytes, error::Result, group, lstat::Lstat, user};

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

    pub fn is_application(&self) -> bool {
        let mime_type = self.fallback_mime_type().unwrap_or_default();

        mime_type.starts_with("application")
    }

    pub fn is_executable(&self) -> bool {
        (self.stat.mode() & S_IXUSR) != 0
    }

    pub fn read_dynamic_dependencies(&self) -> Option<Vec<String>> {
        // Parsing the file's dynamic dependencies requires us to read all of the file's data into memory.
        // By the default, we won't do that if the file's bigger than 100MB
        // TODO: add a CLI option to allow reading more than this
        if self.stat.size() >= 100_000_000 {
            return None;
        }

        let clone_libs = |libs: Vec<&str>| libs.into_iter().map(ToOwned::to_owned).collect();

        let get_libraries = |obj| -> Option<Vec<String>> {
            match obj {
                Object::Elf(elf) => Some(clone_libs(elf.libraries)),
                Object::PE(pe) => Some(clone_libs(pe.libraries)),
                _ => None,
            }
        };

        // TODO: try to make this without reading the whole file into memory
        let bytes = fs::read(&self.path).ok()?;

        Object::parse(&bytes).ok().and_then(get_libraries)
    }
}
