use std::{convert::TryFrom, env, path::PathBuf};

use fs_err as fs;
use inquire::FileData;
use unixstring::UnixString;

fn main() {
    if let Err(err) = run() {
        println!("Error: {}", err);
    }
}

fn run() -> inquire::Result<()> {
    for result in env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .map(fs::canonicalize)
    {
        let unix_string = UnixString::try_from(result?)?;
        let file_data = FileData::read(unix_string)?;
        println!("{}", file_data);
    }

    Ok(())
}
