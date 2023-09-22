use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct Display {
    private: [u8; 0],
}

#[link(name = "X11")]
extern "system" {
    pub fn XOpenDisplay(name: *const c_char) -> *mut Display;
    pub fn XCloseDisplay(display: *mut Display) -> c_int;
    pub fn XFree(data: *mut c_void) -> c_int;
    pub fn XDefaultScreen(display: *mut Display) -> c_int;
}
