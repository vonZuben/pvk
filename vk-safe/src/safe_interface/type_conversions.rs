use std::os::raw::c_char;
use std::ffi::CStr;

pub trait ToC<C> {
    fn to_c(self) -> C;
}

impl<C> ToC<C> for C {
    fn to_c(self) -> C {
        self
    }
}

impl ToC<*const c_char> for Option<&CStr> {
    fn to_c(self) -> *const c_char {
        match self {
            Some(s) => s.as_ptr(),
            None => std::ptr::null(),
        }
    }
}

impl<'a, P> ToC<*const P> for Option<&'a P> {
    fn to_c(self) -> *const P {
        // Option<&P> should be same as &P
        unsafe { std::mem::transmute(self) }
    }
}