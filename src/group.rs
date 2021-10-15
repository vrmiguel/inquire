use libc::{getgrgid, group};
use unixstring::UnixString;

pub fn get_group_name(gid: u32) -> Option<String> {
    // TODO: investigate if it'd be important to use getgrgid_r here
    let grp: *const group = unsafe { getgrgid(gid) };

    if grp.is_null() {
        return None;
    }

    // Safety: we've just checked that this raw pointer is non-null.
    // Dereferencing it is safe, therefore.
    let grp = unsafe { *grp };

    let group_name = unsafe { UnixString::from_ptr(grp.gr_name) }.into_string_lossy();

    Some(group_name)
}
