use std::env;

use inquire::FileData;

fn main() {
    if let Some(s) = env::args_os().skip(1).next() {
        let data = FileData::read(s.into()).unwrap();

        dbg!(data.mime_type());
        dbg!(data.owner_user());
        println!("{}", data.size());
    }
}
