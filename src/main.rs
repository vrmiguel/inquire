use std::env;

use inquire::FileData;

fn main() {
    if let Some(s) = env::args_os().nth(1) {
        let data = FileData::read(s).unwrap();

        // // let _ = dbg!(data.mime_type());
        // dbg!(data.mime_type2());
        // dbg!(data.owner_user());
        // dbg!(data.owner_group());
        println!("{}", data);
    }
}
