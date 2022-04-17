use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ffi::CStr;
use std::ptr::NonNull;

// // =================Custom Result=======================
// // there is more than one success value in Vulkan, which is important to know for some APIs
// // so we need to keep it along with any actual returend values
// pub enum VkResult<T> {
//     Ok(T, vk_safe_sys::Result),
//     Err(vk_safe_sys::Result),
// }

// impl <T> VkResult<T> {
//     pub(crate) fn new(t: T, code: vk_safe_sys::Result) -> Self {
//         if code.is_err() {
//             Self::Err(code)
//         }
//         else {
//             Self::Ok(t, code)
//         }
//     }
//     pub(crate) fn err<U>(self) -> VkResult<U> {
//         match self {
//             Self::Ok(_, _) => panic!("vk-safe internal error: converting VkResult that is not an error"),
//             Self::Err(e) => VkResult::Err(e),
//         }
//     }
//     pub(crate) fn val(self) -> T {
//         match self {
//             Self::Ok(t, _) => t,
//             Self::Err(_) => panic!("vk-safe internal error: taking value of error"),
//         }
//     }
//     pub(crate) fn is_err(&self) -> bool {
//         match self {
//             Self::Ok(_, _) => false,
//             Self::Err(_) => true,
//         }
//     }
//     pub fn result(self) -> Result<T, vk_safe_sys::Result> {
//         match self {
//             Self::Ok(t, _) => Ok(t),
//             Self::Err(e) => Err(e),
//         }
//     }
//     pub fn vk_result_code(&self) -> vk_safe_sys::Result {
//         match self {
//             Self::Err(e) => *e,
//             Self::Ok(_, e) => *e,
//         }
//     }
// }

// macro_rules! check_err {
//     ( $result:ident ) => {
//         match $result {
//             VkResult::Ok(t, _) => t,
//             VkResult::Err(e) => return VkResult::Err(e),
//         }
//     };
// }

macro_rules! check_raw_err {
    ( $result:ident ) => {
        if $result.is_err() {
            return Err($result);
        }
    };
}

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
// #[repr(transparent)]
// #[derive(Clone, Copy)]
// pub struct VkStr<'a> {
//     ptr: NonNull<c_char>,
//     _p: PhantomData<&'a c_char>,
// }

// impl<'a> VkStr<'a> {
//     pub fn ptr(&self) -> *const c_char {
//         self.ptr.as_ptr()
//     }
// }

// impl<'a> From<&'a CStr> for VkStr<'a> {
//     fn from(from: &'a CStr) -> Self {
//         // CStr: "This type represents a borrowed reference to a nul-terminated array of bytes"
//         // thus, we should beable to make NonNull
//         // VkStr should not ever mutate the string, so taking the *mut should be fine
//         Self {
//             ptr: unsafe { NonNull::new_unchecked(from.as_ptr() as *mut c_char) },
//             _p: PhantomData,
//         }
//     }
// }

// =================Buffer type that people can use to pass their own allocated space===============
// for som APIs, want to provide the option for the user to provide their own allocation to be filled in
// However, this is kind of difficult to design, because I don't even know all possible reasons to want this (should I even be doing this????)
//
// Just keep this as a work in progress for now
// we can use it later if usful
pub trait BufferInner<T> : AsRef<[T]> + AsMut<[T]> {}
impl<T, B> BufferInner<T> for B where B: AsRef<[T]> + AsMut<[T]> {}

pub struct Buffer<B, T> {
    buffer: B,
    len: usize,
    _p: PhantomData<[T]>,
}

impl<T, B: BufferInner<T>> Buffer<B, T> {
    pub fn uninit(buffer: B) -> Self {
        Self {
            buffer,
            len: 0,
            _p: PhantomData,
        }
    }
}

impl<T, B: BufferInner<T>> From<B> for Buffer<B, T> {
    fn from(buffer: B) -> Self {
        Self::uninit(buffer)
    }
}

impl<T, B: BufferInner<T>> std::ops::Deref for Buffer<B, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.buffer.as_ref()[..self.len]
    }
}
