use std::ffi::CString;

pub mod cdef;

pub struct Display
{
    _private: *mut cdef::Display
}

impl Display
{
    pub fn create_invalid() -> Self
    {
        Display
        {
            _private: std::ptr::null_mut()
        }
    }
}

/// An xlib equivalent to Rust's Box that uses XFree to free memory.
/// Make sure to only use this struct with resources that are meant
/// to be freed with XFree.
pub struct XBox<T: ?Sized>
{
    data: XBoxFatPtr,
    phantom: std::marker::PhantomData<*const T>
}

#[allow(dead_code)]
struct XBoxFatPtr
{
    data: *const std::ffi::c_void,
    length: usize
}

impl<T> XBox<T>
{
    pub fn from_raw(ptr:*mut T) -> Self
    {
        XBox {
            data: XBoxFatPtr
            {
                data: ptr as *const std::ffi::c_void,
                length: 1,
            },
            phantom: std::marker::PhantomData
        }
    }
}

impl<T> XBox<[T]>
{
    pub fn boxed_slice_from_raw(ptr:*mut T, length: usize) -> XBox<[T]>
    {
        XBox {
            data: XBoxFatPtr
            {
                data: ptr as *const std::ffi::c_void,
                length: length
            },
            phantom: std::marker::PhantomData
        }
    }
}

impl<T> std::ops::Deref for XBox<T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        unsafe
        {
            &*(self.data.data as *const Self::Target)
        }
    }
}

impl<T> std::ops::DerefMut for XBox<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        unsafe
        {
            &mut *(self.data.data as *mut Self::Target)
        }
    }
}

impl<T> std::ops::Deref for XBox<[T]>
{
    type Target = [T];

    fn deref(&self) -> &Self::Target
    {
        unsafe
        {
            // FIXME: incredibly unsafe and ugly
            let temp: & *const Self::Target = std::mem::transmute(&self.data);
            &*(*temp)
        }
    }
}

impl<T> std::ops::DerefMut for XBox<[T]>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        unsafe
        {
            // FIXME: incredibly unsafe and ugly
            let temp: & *mut Self::Target = std::mem::transmute(&self.data);
            &mut *(*temp)
        }
    }
}

impl<T: ?Sized> Drop for XBox<T>
{
    fn drop(&mut self)
    {
        unsafe
        {
            cdef::XFree(self.data.data as *mut std::ffi::c_void);
        }
    }
}

pub fn x_open_display(display_name: Option<&str>) -> Result<Display, Box<dyn std::error::Error>>
{
    match display_name
    {
        None =>
        {
            let display = unsafe {
                cdef::XOpenDisplay(std::ptr::null())
            };

            return Ok( Display { _private: display } )
        },
        Some(name) =>
        {
            match CString::new(name)
            {
                Err(err) =>
                {
                    return Err(
                        Box::new(err)
                    );
                },
                Ok(cstr) =>
                {
                    let ptr = cstr.as_ptr();
                    let result = unsafe
                    {
                        cdef::XOpenDisplay(ptr)
                    };
                    return Ok( Display { _private: result } );
                }
            }
        }
    }
}

pub fn x_close_display(display: Display) -> i32
{
    unsafe
    {
        cdef::XCloseDisplay(display._private)
    }
}

pub fn x_default_screen(display: &Display) -> i32
{
    unsafe
    {
        cdef::XDefaultScreen(display._private)
    }
}
