use std::path::Path;
use std::io::Read;
use std::fs;

use goblin::Object;

pub fn read_dynamic_dependencies(path: impl AsRef<Path>) -> Option<Vec<String>> {
    // We can probably read less than 64 and have this function to still work
    let mut bytes = [0_u8; 16];

    let mut file = fs::File::open(path.as_ref()).ok()?;
    let metadata = file.metadata().ok()?;
    if metadata.len() < 16 {
        return None;
    }

    file.read_exact(&mut bytes).ok()?;

    let clone_libs = |libs: Vec<&str>| {
        libs.into_iter().map(ToOwned::to_owned).collect()
    };

    let get_libraries = |obj| -> Option<Vec<String>> {
        match obj {
            Object::Elf(elf) => Some(clone_libs(elf.libraries)),
            Object::PE(pe) => Some(clone_libs(pe.libraries)),
            _ => {
                None
            }
        }
    };


    let t = fs::read(path).unwrap();
    assert!(t[0..16] == bytes);

    // TODO: what is going on?
    if let Ok(t) = Object::parse(&t) {
        return get_libraries(t)
    }

    Object::parse(&bytes).ok().and_then(get_libraries)
}