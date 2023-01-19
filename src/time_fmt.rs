use std::mem;

use cstr::cstr;
use libc::{c_char, localtime_r, size_t, tm};
use unixstring::UnixString;

use crate::uninitialized;

const BUF_SIZ: usize = 64;

extern "C" {
    pub fn strftime(
        s: *mut c_char,
        maxsize: size_t,
        format: *const c_char,
        timeptr: *const tm,
    ) -> size_t;

    pub fn tzset();
}

pub fn format_timestamp(timestamp: i64) -> String {
    // Safety: the all-zero byte-pattern is valid struct tm
    let mut new_time: tm = unsafe { mem::zeroed() };

    unsafe { tzset() };

    // Safety: localtime_r is memory safe, thread-safe.
    unsafe { localtime_r(&timestamp as *const i64, &mut new_time as *mut tm) };

    // Safety: it's ok for this to be uninitialized since `strftime` will
    // null-terminate this c-string
    let mut char_buf: [c_char; BUF_SIZ] = unsafe { uninitialized() };

    let format = cstr!("%A %b/%d/%Y %H:%M:%S");

    unsafe {
        strftime(
            char_buf.as_mut_ptr(),
            BUF_SIZ,
            format.as_ptr(),
            &new_time as *const tm,
        )
    };

    let timestamp = unsafe { UnixString::from_ptr(char_buf.as_ptr()) };

    timestamp.into_string_lossy()
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::format_timestamp;

    #[test]
    fn timestamp_formatting() {
        // We'll use the chrono crate to make sure that
        // our own formatting (done through libc's strftime) works
        let date_time = Local::now();

        let chrono_formatted = date_time.format("%A %b/%d/%Y %H:%M:%S").to_string();

        assert_eq!(&chrono_formatted, &format_timestamp(date_time.timestamp()));
    }
}
