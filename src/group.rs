use std::ffi::CStr;

use libc::{group, getgrgid};

pub fn get_group_name(gid: u32) -> Option<String> {

    // TODO: investigate if it'd be important to use getgrgid_r here
    let grp: *const group = unsafe { getgrgid(gid) };

    if grp.is_null() {
        return None
    }

    // Safety: we've just checked that this raw pointer is non-null.
    // Dereferencing it is safe, therefore.
    let grp = unsafe { *grp };

    let group_name = unsafe { CStr::from_ptr(grp.gr_name) };
    let group_name = String::from_utf8_lossy(group_name.to_bytes());

    Some(group_name.into())
}
