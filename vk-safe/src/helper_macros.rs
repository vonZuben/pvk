// macro_rules! result_getter_code {
//     ( $fn_name:ident $(<$generic:ident>)? ( $($param:ident : $param_t:ident $(<$($gp:tt)*>)? ),* ) -> $getting:ty ) => {
//         fn $fn_name(&self, $($param : $param_t ,)*) -> Result<$getting> {
//             let mut get = MaybeUninit::uninit();
//             unsafe {
//                 let res = self.commands.get()($($param.to_c(),)* None.to_c(), get.as_mut_ptr());
//                 check_raw_err!(res);
//                 Ok(get.assume_init())
//             }
//         }
//     };
// }

// enumerators are all very similar, so why repeat ourselves
macro_rules! enumerator_code2 {
    ( $command:expr; ( $($param:expr),* ) -> $storage:ident ) => {{
        use std::convert::TryInto;
        use crate::array_storage::VulkanLenType;
        #[allow(unused)]
        use crate::safe_interface::type_conversions::ToC;
        let len = || {
            let mut num = 0;
            let res;
            unsafe {
                res = $command($($param.to_c(),)* &mut num, std::ptr::null_mut());
                check_raw_err!(res);
            }
            Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
        };
        $storage.allocate(len)?;
        let uninit_slice = $storage.uninit_slice();
        let mut len = crate::array_storage::VulkanLenType::from_usize(uninit_slice.len());
        let res;
        unsafe {
            res = $command($($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
            check_raw_err!(res);
        }
        let ret: Result<_, crate::error::Error> = Ok($storage.finalize(len.to_usize()));
        ret
    }};
}

// Use this to create wrappers around simple structs
macro_rules! simple_struct_wrapper {
    (
        $name:ident
    ) => {
        #[repr(transparent)]
        pub struct $name {
            inner: vk_safe_sys::$name,
        }

        impl std::ops::Deref for $name {
            type Target = vk_safe_sys::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };
}

pub(crate) fn str_len(s: &[std::ffi::c_char]) -> usize {
    s.iter().take_while(|&&c| c != 0).count()
}

// Use this to create wrappers around simple structs that are scoped
macro_rules! simple_struct_wrapper_scoped {
    (
        $name:ident $(impl $($t:ident),+ $(,)?)?
    ) => {
        #[repr(transparent)]
        pub struct $name<'scope> {
            inner: vk_safe_sys::$name,
            _scope: crate::scope::ScopeId<'scope>,
        }

        unsafe impl crate::safe_interface::type_conversions::SafeTransmute<$name<'_>> for vk_safe_sys::$name {}

        impl<'scope> $name<'scope> {
            #[allow(unused)]
            pub(crate) fn new(inner: vk_safe_sys::$name) -> Self {
                Self {
                    inner,
                    _scope: Default::default(),
                }
            }
        }

        $( $( simple_struct_wrapper_scoped!( @IMPL $t $name ); )+ )?
    };

    ( @IMPL Deref $name:ident ) => {
        impl std::ops::Deref for $name<'_> {
            type Target = vk_safe_sys::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };

    ( @IMPL Debug $name:ident ) => {
        impl std::fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }
    };

    ( @IMPL Clone $name:ident ) => {
        impl Clone for $name<'_> {
            fn clone(&self) -> Self {
                Self::new(self.inner)
            }
        }
    };

    ( @IMPL Copy $name:ident ) => {
        impl Copy for $name<'_> { }
    };
}

macro_rules! get_str {
    (
        $name:ident
    ) => {
        pub fn $name(&self) -> &str {
            let unchecked_utf8;
            unsafe {
                unchecked_utf8 = std::slice::from_raw_parts(self.inner.$name.as_ptr().cast(), crate::helper_macros::str_len(&self.inner.$name));
            }
            std::str::from_utf8(unchecked_utf8).expect("vk safe interface internal error: string from Vulkan implementation is not proper utf8")
        }
    };
}

macro_rules! pretty_version {
    (
        $version_param:ident
    ) => {
        pub fn $version_param(&self) -> crate::pretty_version::VkVersion {
            unsafe { crate::pretty_version::VkVersion::from_raw(self.inner.$version_param) }
        }
    };
}

macro_rules! array {
    (
        $name:ident, $array_ptr:ident, $array_len:ident, $ty:ty
    ) => {
        pub fn $name(&self) -> &[$ty] {
            unsafe {
                std::slice::from_raw_parts(self.inner.$array_ptr, self.inner.$array_len as usize)
            }
        }
    };
}

// TODO, exported macro probably belong somewhere else
#[macro_export]
macro_rules! bitmask {
    ( $($bit:path)|* ) => {
        krs_hlist::hlist!( $( $bit ),* )
    };
    ( $path:path : $($bit:ident)|* ) => {
        krs_hlist::hlist!( $( $path::$bit ),* )
    };
}

// // check things that looks like this from where they are defined
// // const VUID_vkEnumeratePhysicalDevices_instance_parameter: &'static str = "instance must be a valid VkInstance handle";
// // when implementing vuid checks, create definitions checkers to ensure that if the definition changes later, we know to update our check
// macro_rules! check_vuid_defs {
//     ( $( pub const $vuid:ident : &'static [u8] = $def:expr ;)* ) => {
//         #[allow(non_upper_case_globals)]
//         #[allow(unused)]
//         const CHECK_DEF: () = {
//             $(
//                 match $def {
//                     $vuid => {}
//                     _ => panic!(concat!("definition for ", stringify!($vuid), " has been updated")),
//                 }
//             )*
//         };
//     };
// }

// check things that looks like this from where they are defined
// const VUID_vkEnumeratePhysicalDevices_instance_parameter: &'static str = "instance must be a valid VkInstance handle";
// when implementing vuid checks, create definitions checkers to ensure that if the definition changes later, we know to update our check
macro_rules! check_vuid_defs2 {
    ( $target:ident $( pub const $vuid:ident : &'static [u8] = $def:expr ; $( CHECK { $($check_code:tt)* } )? )* ) => {
        #[allow(non_upper_case_globals)]
        {
            use vk_safe_sys::validation::$target::*;

            struct _CheckMissingVuids;

            impl Vuids for _CheckMissingVuids {
                $(
                    const $vuid: () = {};
                )*
            }

            $(
                match $def {
                    $vuid => {}
                    _ => panic!(concat!(stringify!($vuid), " has different definition")),
                }
                $(
                    $($check_code)*
                )?
            )*
        }
    };
}
