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
        use crate::type_conversions::{ToC, TransmuteUninitSlice};
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
            res = $command($($param.to_c(),)* &mut len, uninit_slice.safe_transmute_uninit_slice());
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

        unsafe impl crate::type_conversions::SafeTransmute<$name> for vk_safe_sys::$name {}
        unsafe impl crate::type_conversions::SafeTransmute<vk_safe_sys::$name> for $name {}

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
        pub struct $name<S> {
            inner: vk_safe_sys::$name,
            _scope: std::marker::PhantomData<S>,
        }

        unsafe impl<S> crate::type_conversions::SafeTransmute<$name<S>> for vk_safe_sys::$name {}
        unsafe impl<S> crate::type_conversions::SafeTransmute<vk_safe_sys::$name> for $name<S> {}

        impl<S> $name<S> {
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
        impl<S> std::ops::Deref for $name<S> {
            type Target = vk_safe_sys::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };

    ( @IMPL Debug $name:ident ) => {
        impl<S> std::fmt::Debug for $name<S> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }
    };

    ( @IMPL Clone $name:ident ) => {
        impl<S> Clone for $name<S> {
            fn clone(&self) -> Self {
                Self::new(self.inner)
            }
        }
    };

    ( @IMPL Copy $name:ident ) => {
        impl<S> Copy for $name<S> { }
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
