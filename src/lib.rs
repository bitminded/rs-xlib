use std::ffi::CString;
use std::fmt;

pub mod cdef;

/// A simple wrapper around a raw pointer usually returned by some functions that are part of the
/// safe interface of this library. Its primary purpose is to signal that the resource the raw
/// pointer is pointing to is managed by Xlib itself and therefore is not to be freed manually.
pub struct DoNotFree<T> {
    data: *mut T,
}

impl<T> std::ops::Deref for DoNotFree<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.data) }
    }
}

impl<T> std::ops::DerefMut for DoNotFree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.data) }
    }
}

#[derive(Debug)]
pub struct XlibError {
    message: String,
    kind: ErrorKind,
    side: Option<Box<dyn std::error::Error>>,
}

impl std::error::Error for XlibError {}

impl fmt::Display for XlibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XlibError")
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    InvalidArgumentValue,
}

/// An xlib equivalent to Rust's Box that uses XFree to free memory.
/// Make sure to only use this struct with resources that are meant
/// to be freed with XFree.
pub struct XBox<T: ?Sized> {
    data: XBoxFatPtr,
    phantom: std::marker::PhantomData<*const T>,
}

#[allow(dead_code)]
struct XBoxFatPtr {
    data: *const std::ffi::c_void,
    length: usize,
}

impl<T> XBox<T> {
    pub fn from_raw(ptr: *mut T) -> Self {
        XBox {
            data: XBoxFatPtr {
                data: ptr as *const std::ffi::c_void,
                length: 1,
            },
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> XBox<[T]> {
    pub fn boxed_slice_from_raw(ptr: *mut T, length: usize) -> XBox<[T]> {
        XBox {
            data: XBoxFatPtr {
                data: ptr as *const std::ffi::c_void,
                length: length,
            },
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> std::ops::Deref for XBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.data.data as *const Self::Target) }
    }
}

impl<T> std::ops::DerefMut for XBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.data.data as *mut Self::Target) }
    }
}

impl<T> std::ops::Deref for XBox<[T]> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            // FIXME: incredibly unsafe and ugly
            let temp: &*const Self::Target = std::mem::transmute(&self.data);
            &*(*temp)
        }
    }
}

impl<T> std::ops::DerefMut for XBox<[T]> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // FIXME: incredibly unsafe and ugly
            let temp: &*mut Self::Target = std::mem::transmute(&self.data);
            &mut *(*temp)
        }
    }
}

impl<T: ?Sized> Drop for XBox<T> {
    fn drop(&mut self) {
        unsafe {
            cdef::XFree(self.data.data as *mut std::ffi::c_void);
        }
    }
}

/// Retrieves a connection, also known as a display, to the X server.
///
/// # Parameters
/// ## display_name
/// Specifies the hardware display name, which determines the display and communcations
/// domain to be used. On a POSIX-conformant system, if the display_name is None,
/// it defaults to the value of the DISPLAY environment variable.
///
/// # Return value
/// If display_name contains an invalid string (i.e. contains any 0 bytes),
/// the returned Result will hold an XlibError of kind InvalidArgumentValue.
///
/// If x_open_display succeeds, the Result returned will hold a Display struct
/// that serves as the connection to the X server.
///
/// If x_open_display can't retrieve a display for any other reason, the returned
/// Result will hold a value of None.
///
/// # Remarks
/// x_open_display connects the application to the X server through TCP or DECnet
/// communications protocols, or through some local inter-process communication
/// protocol. If the hostname is a host machine name and a single colon (:)
/// separates the hostname and display number, x_open_display connects using TCP
/// streams. If the hostname is not specified, Xlib uses whatever it believes is
/// the fastest transport. If the hostname is a host machine name and a double
/// colon (::) separates the hostname and display number, x_open_display connects
/// using DECnet. A single X server can support any or all of these transport
/// mechanisms simultaneously. A particular Xlib implementation can support many
/// more of these transport mechanisms.
///
/// After a successful call to x_open_display, all of the screens in the display
/// can be used by the client. The screen number specified in the display_name
/// argument is returned by the x_default_screen function. You can access elements
/// of the Display and Screen structures only by using the information functions.
///
/// Use x_close_display before exiting the program to destroy all resoures created
/// on the display.
pub fn x_open_display(
    display_name: Option<&str>,
) -> Result<Option<DoNotFree<cdef::Display>>, XlibError> {
    let display = match display_name {
        None => unsafe { cdef::XOpenDisplay(std::ptr::null()) },
        Some(display_name) => {
            let display_name = match CString::new(display_name) {
                Err(err) => {
                    return Err(XlibError {
                        message: String::from("Failed to convert display_name to CString."),
                        kind: ErrorKind::InvalidArgumentValue,
                        side: Some(Box::new(err)),
                    });
                }
                Ok(display_name) => display_name,
            };
            let display_name = display_name.as_ptr();
            unsafe { cdef::XOpenDisplay(display_name) }
        }
    };

    if display.is_null() {
        Ok(None)
    } else {
        Ok(Some(DoNotFree { data: display }))
    }
}

pub fn x_close_display(display: DoNotFree<cdef::Display>) -> i32 {
    unsafe { cdef::XCloseDisplay(display.data) }
}

pub fn x_default_screen(display: &mut DoNotFree<cdef::Display>) -> i32 {
    unsafe { cdef::XDefaultScreen(display.data) }
}
