use std::ffi::c_char;

/// UTF8 c string
///
/// rust std::ffi::CStr is not guaranteed to be UTF8, but vulkan interfaces need UTF8
/// vulkan also need null terminated strings.
///
/// this type combines the guarantees of str anc CStr
#[derive(Debug, Clone, Copy)]
pub struct VkStr<'a>(&'a str);

impl<'a> VkStr<'a> {

    /// create a VkStr from a regular &str
    /// unsafe since the caller must ensure it is null terminated
    pub const unsafe fn new(s: &'a str) -> Self {
        Self(s)
    }

    /// get the raw pointer to the c style string
    pub const fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }
}

/// convenience macro to safely create a VkStr with a reference to a normal rust string
/// will concat!() a null ending to ensure it is a proper c string
#[macro_export]
macro_rules! vk_str {
    ( $str:literal ) => {{
        let _: &str = $str; // just to confirm that a &str is provided
        unsafe { $crate::VkStr::new(concat!($str, "\0")) }
    }};
}