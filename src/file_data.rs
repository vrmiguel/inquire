// Allow since these casts are not "unnecessary" in all platforms, e.g. `S_IXUSR` on macOS aarch64
// is not a u32.
#![allow(clippy::unnecessary_cast)]

use std::fmt::Display;

use fs_err as fs;
use goblin::Object;
use infer::Type;
use libc::{S_IRWXG, S_IRWXO, S_IRWXU, S_IXUSR};
use memmap::MmapOptions;
use tabular::{Row, Table};
use unixstring::UnixString;
use wizardry::Magic;

use crate::{bytes::Bytes, error::Result, group, lstat::Lstat, time_fmt::format_timestamp, user};

#[non_exhaustive]
pub struct FileData {
    path: UnixString,
    stat: Lstat,
    magic_cookie: Option<Magic>,
}

fn libmagic_display(mime_msg: String, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    for line in mime_msg.split(',') {
        writeln!(f, "· {}", line.trim())?;
    }

    Ok(())
}

impl Display for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "[{}]", self.path.as_path().display())?;

        let mut table = Table::new("{:<}\t\t{:<}\t{:<}");

        if let Some(mime) = self.libmagic_mime_type() {
            libmagic_display(mime, f)?;
        } else if let Some(mime) = self.fallback_mime_type() {
            table.add_row(Row::new().with_cell("type:").with_cell(mime).with_cell(""));
        }

        if self.is_executable() {
            if let Some(libraries) = self.read_dynamic_dependencies() {
                writeln!(f, "\n[dependencies]")?;
                for library in libraries {
                    writeln!(f, "· {library}")?;
                }
            }
        }

        writeln!(f, "\n[file info]")?;

        table.add_row(
            Row::new()
                .with_cell("size:")
                .with_cell(self.size())
                .with_cell(""),
        );

        let (mode, permission) = self.permissions();
        table.add_row(
            Row::new()
                .with_cell("permissions:")
                .with_cell(permission)
                .with_cell(format_args!("0{mode:o}")),
        );

        if let Some((uid, username)) = self.owner_user() {
            table.add_row(
                Row::new()
                    .with_cell("owner:")
                    .with_cell(username)
                    .with_cell(uid),
            );
        }

        if let Some((gid, group_name)) = self.owner_group() {
            table.add_row(
                Row::new()
                    .with_cell("owner's group:")
                    .with_cell(group_name)
                    .with_cell(gid),
            );
        }

        let last_accessed = self.last_accessed_time();
        let last_modified = self.last_modified_time();

        let last_accessed = format_timestamp(last_accessed);
        let last_modified = format_timestamp(last_modified);

        table.add_row(
            Row::new()
                .with_cell("last modified:")
                .with_cell(last_modified)
                .with_cell(""),
        );

        table.add_row(
            Row::new()
                .with_cell("last accessed:")
                .with_cell(last_accessed)
                .with_cell(""),
        );

        write!(f, "{table}")
    }
}

impl FileData {
    /// Important: assumes that the given path is canonical.
    /// ```rust
    /// use inquire::FileData;
    /// use unixstring::UnixString;
    ///
    /// let mut unx = UnixString::new();
    /// unx.push("Cargo.toml");
    ///
    /// let cargo_toml = FileData::read(unx).unwrap();
    /// println!("{}", cargo_toml.size());
    /// ```
    pub fn read(path: UnixString) -> Result<Self> {
        Ok(Self {
            stat: Lstat::new(&path)?,
            path,
            magic_cookie: Magic::new().ok(),
        })
    }

    /// Attempts to read the file's MIME type through libmagic.
    pub fn libmagic_mime_type(&self) -> Option<String> {
        let file = |magic: &Magic| magic.file(&self.path).ok();

        self.magic_cookie.as_ref().and_then(file)
    }

    /// Attempts to read the file's MIME type through the `infer` crate.
    /// This is used as a fallback method since getting this data through libmagic
    /// yields more information.
    pub fn fallback_mime_type(&self) -> Option<&'static str> {
        let mime = |t: Type| t.mime_type();

        infer::get_from_path(&self.path).ok()?.map(mime)
    }

    /// Returns the size of the file in a Display-capable struct
    pub fn size(&self) -> Bytes {
        Bytes::new(self.stat.size() as u64)
    }

    /// Returns the file`s last accessed time represented in Unix timestamp
    pub fn last_accessed_time(&self) -> i64 {
        self.stat.accessed()
    }

    /// Returns the file`s last modified time represented in Unix timestamp
    pub fn last_modified_time(&self) -> i64 {
        self.stat.modified()
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
        let permission_bits = (S_IRWXU | S_IRWXG | S_IRWXO) as u32;
        let mode = self.stat.mode();
        (mode & permission_bits, unix_mode::to_string(mode))
    }

    /// Returns true if this file is in the `application`
    pub fn is_application(&self) -> bool {
        let mime_type = self.fallback_mime_type().unwrap_or_default();

        //                          cation
        mime_type.starts_with("appli")
    }

    /// Returns true if this file has the executable bit turned on
    pub fn is_executable(&self) -> bool {
        (self.stat.mode() & (S_IXUSR as u32)) != 0
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
        let map = unsafe { MmapOptions::new().map(file.file()) }.ok()?;

        Object::parse(&map).ok().and_then(get_libraries)
    }
}
