use std::{ffi::{CStr, OsStr}, mem, os::unix::prelude::OsStrExt, path::PathBuf, ptr};

use libc::{getpwuid_r, passwd};

// fn real_user_id() -> u32 {
//     // Safety: the POSIX Programmer's Manual states that
//     // getuid will always be successful.
//     unsafe { libc::getuid() }
// }

fn effective_user_id() -> u32 {
    // Safety: the POSIX Programmer's Manual states that
    // geteuid will always be successful.
    unsafe { libc::geteuid() }
}

pub fn get_username(uid: u32) -> Option<String> {
    let mut buf = [0; 2048];
    let mut result = ptr::null_mut();
    let mut passwd: passwd = unsafe { mem::zeroed() };

    let getpwuid_r_code =
        unsafe { getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

    if getpwuid_r_code == 0 && !result.is_null() {
        // If getpwuid_r succeeded, let's get the username from it
        let username = unsafe { CStr::from_ptr(passwd.pw_name) };
        let username = String::from_utf8_lossy(username.to_bytes());

        return Some(username.into());
    }

    None
}