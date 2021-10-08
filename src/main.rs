use std::env;

use inquire::FileData;

fn main() {
    if let Some(s) = env::args_os().nth(1) {
        let data = FileData::read(s).unwrap();

        print!("{}", data);
    }
}
