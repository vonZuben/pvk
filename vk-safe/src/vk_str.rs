use std::ffi::c_char;

/// UTF8 c string
///
/// rust std::ffi::CStr is not guaranteed to be UTF8, but vulkan interfaces needs UTF8
/// vulkan also needs null terminated c strings.
///
/// this type combines the guarantees of str anc CStr
#[derive(Debug, Clone, Copy)]
pub struct VkStr<'a>(&'a str);

impl<'a> VkStr<'a> {
    /// create a VkStr from a regular str slice
    /// The caller must guarantee that the str slice is null terminated
    pub const unsafe fn new(s: &'a str) -> Self {
        Self(s)
    }

    /// get the raw pointer to the c style string
    pub const fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }
}

impl std::cmp::PartialEq<vk_safe_sys::VkStrRaw> for VkStr<'_> {
    fn eq(&self, other: &vk_safe_sys::VkStrRaw) -> bool {
        let mut s1: *const std::ffi::c_char = self.0.as_ptr().cast();
        let mut s2: *const std::ffi::c_char = other.as_ptr();
        unsafe {
            while *s1 == *s2 && *s1 != 0 {
                s1 = s1.add(1);
                s2 = s2.add(1);
            }
            *s1 == *s2
        }
    }
}

/// Safely create a VkStr
///
/// This is a convenience macro that takes a user provided string literal, ensures it is valid as &str,
/// and appends a null character with concat!()
#[macro_export]
macro_rules! vk_str {
    ( $str:literal ) => {{
        let _: &str = $str; // just to confirm that a &str is provided
        unsafe { $crate::VkStr::new(concat!($str, "\0")) }
    }};
}
