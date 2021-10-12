use std::{env, path::PathBuf};

use fs_err as fs;
use inquire::FileData;

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
        let path = result?;
        let file_data = FileData::read(path)?;
        println!("{}", file_data);
    }

    Ok(())
}
