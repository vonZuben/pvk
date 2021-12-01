use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::ptr::NonNull;

// =================VkVersion===========================
#[repr(transparent)]
#[derive(Default)]
pub struct VkVersion(u32);

impl VkVersion {
    pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self( (variant << 29) | (major << 22) | (minor << 12) | (patch) )
    }
    pub fn parts(&self) -> (u32, u32, u32, u32) {
        (self.0 >> 29, (self.0 >> 22) & 0x7F, (self.0 >> 12) & 0x3FF, self.0 & 0xFFF)
    }
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Debug for VkVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, major, minor, patch) = self.parts();
        write!(f, "{}.{}.{}.{}", variant, major, minor, patch)
    }
}

impl From<(u32, u32, u32)> for VkVersion {
    fn from((major, minor, patch): (u32, u32, u32)) -> Self {
        Self::new(0, major, minor, patch)
    }
}

// =================VkStr===========================

/// This must be compatible with char* in C so that it can be used in arrays of c strings
/// Also ensure that representation with Option can make use of null niche so that
/// Option<VkStr> can be cheaply converted and even maybe used directly in the ffi interface
/// 
/// Also important, the string must never be mutated from this
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct VkStr<'a> {
    ptr: NonNull<c_char>,
    _p: PhantomData<&'a c_char>,
}

impl<'a> VkStr<'a> {
    pub fn ptr(&self) -> *const c_char {
        self.ptr.as_ptr()
    }
}

impl<'a> From<&'a CStr> for VkStr<'a> {
    fn from(from: &'a CStr) -> Self {
        // CStr: "This type represents a borrowed reference to a nul-terminated array of bytes"
        // thus, we should beable to make NonNull
        // VkStr should not ever mutate the string, so taking the *mut should be fine
        Self {
            ptr: unsafe { NonNull::new_unchecked(from.as_ptr() as *mut c_char) },
            _p: PhantomData,
        }
    }
}

// =================OptionPtr===========================

/// This trait is intended to be implemented on Options wrapping ref/ptr like types
/// which can be converted (hopfully cheaply) to c ptr types
pub trait OptionPtr {
    type Ptr;
    fn as_c_ptr(self) -> Self::Ptr;
}

impl<'a> OptionPtr for Option<VkStr<'a>> {
    type Ptr = *const c_char;
    fn as_c_ptr(self) -> Self::Ptr {
        // Note: transmute already ensures that Self and Ptr are the same size
        unsafe { std::mem::transmute(self) }
    }
}