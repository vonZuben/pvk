use std::fmt;
// // =================Custom Result=======================
// // there is more than one success value in Vulkan, which is important to know for some APIs
// // so we need to keep it along with any actual returned values
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
    #[allow(unused)]
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