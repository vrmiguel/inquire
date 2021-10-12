use std::{ffi::CStr, mem};

use cstr::cstr;
use libc::{c_char, localtime_r, size_t, time, tm};

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

pub fn format_timestamp(timestamp: u64) -> String {
    let mut timestamp = timestamp;

    // Safety: the all-zero byte-pattern is valid struct tm
    let mut new_time: tm = unsafe { mem::zeroed() };

    // Safety: time is memory-safe
    // TODO: it'd be better to call `time(NULL)` here
    let ltime = unsafe { time(&mut timestamp as *mut u64 as *mut i64) };

    unsafe { tzset() };

    // Safety: localtime_r is memory safe, thread-safe.
    unsafe { localtime_r(&ltime as *const i64, &mut new_time as *mut tm) };

    let mut char_buf = [0; BUF_SIZ];

    // RFC3339 timestamp
    let format = cstr!("%Y-%m-%dT%T");

    unsafe {
        strftime(
            char_buf.as_mut_ptr(),
            BUF_SIZ,
            format.as_ptr(),
            &new_time as *const tm,
        )
    };

    let c_str = unsafe { CStr::from_ptr(char_buf.as_ptr()) };
    let utf8_encoded = c_str.to_string_lossy();

    utf8_encoded.into()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::Local;

    use super::format_timestamp;

    #[test]
    fn rfc3339_formatting() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        // We'll use the chrono crate to make sure that
        // our own formatting (done through libc's strftime) works
        let date_time = Local::now();

        // YYYY-MM-DDThh:mm:ss
        let rfc3339 = date_time.format("%Y-%m-%dT%T").to_string();

        assert_eq!(&rfc3339, &format_timestamp(now.as_secs()));
    }
}
