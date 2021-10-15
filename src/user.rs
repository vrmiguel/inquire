use std::{mem::{self, MaybeUninit}, ptr};

use libc::{c_char, getpwuid_r, passwd};
use unixstring::UnixString;

// fn real_user_id() -> u32 {
//     // Safety: the POSIX Programmer's Manual states that
//     // getuid will always be successful.
//     unsafe { libc::getuid() }
// }

// fn effective_user_id() -> u32 {
//     // Safety: the POSIX Programmer's Manual states that
//     // geteuid will always be successful.
//     unsafe { libc::geteuid() }
// }

pub fn get_username(uid: u32) -> Option<String> {
    const BUF_SIZ: usize = 2048;
    let mut buf: [c_char; BUF_SIZ] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut result = ptr::null_mut();
    
    // Safety: the all-zero byte pattern is a valid struct passwd
    let mut passwd: passwd = unsafe { mem::zeroed() };

    let status = unsafe { getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

    if status == 0 && !result.is_null() {
        // If getpwuid_r succeeded, let's get the username from it

        let username = unsafe { UnixString::from_ptr(passwd.pw_name) }.into_string_lossy();

        return Some(username);
    }

    None
}
