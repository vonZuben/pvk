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

// TODO this should be deleted since I do not think there are ant vulkan interface that do not require UTF8
// UTF8 is not guaranteed by CStr. use VkStr instead
impl ToC<*const c_char> for Option<&CStr> {
    fn to_c(self) -> *const c_char {
        match self {
            Some(s) => s.as_ptr(),
            None => std::ptr::null(),
        }
    }
}

impl ToC<*const c_char> for crate::VkStr<'_> {
    fn to_c(self) -> *const c_char {
        self.as_ptr()
    }
}

impl ToC<*const c_char> for Option<crate::VkStr<'_>> {
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