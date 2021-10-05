use std::env;

use inquire::FileData;

fn main() {
    if let Some(s) = env::args_os().nth(1) {
        let data = FileData::read(s.into()).unwrap();

        // let _ = dbg!(data.mime_type());
        // dbg!(data.owner_user());
        // dbg!(data.owner_group());
        println!("{}", data.permissions());
        // println!("{}", data.size());
    }
}
