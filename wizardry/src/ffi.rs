// #![allow(dead_code)]
use libc::{c_char, c_int};

pub enum Magic {}

#[link(name = "magic")]
extern "C" {
    pub fn magic_open(flags: c_int) -> *const Magic;
    pub fn magic_close(cookie: *const Magic);
    // pub fn magic_error(cookie: *const Magic) -> *const c_char;
    // pub fn magic_errno(cookie: *const Magic) -> *const c_int;
    pub fn magic_file(cookie: *const Magic, filename: *const c_char) -> *const c_char;
    pub fn magic_load(cookie: *const Magic, filename: *const c_char) -> c_int;
}