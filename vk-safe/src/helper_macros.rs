macro_rules! result_getter_code {
    ( $fn_name:ident $(<$generic:ident>)? ( $($param:ident : $param_t:ident $(<$($gp:tt)*>)? ),* ) -> $getting:ty ) => {
        fn $fn_name(&self, $($param : $param_t ,)*) -> Result<$getting> {
            let mut get = MaybeUninit::uninit();
            unsafe {
                let res = self.commands.get()($($param.to_c(),)* None.to_c(), get.as_mut_ptr());
                check_raw_err!(res);
                Ok(get.assume_init())
            }
        }
    };
}

// enumerators are all very similar, so why repeat ourselves
macro_rules! enumerator_code {
    ( $fn_name:ident ( $($param:ident : $param_t:ty),* ) -> $getting:ty ) => {
        pub fn $fn_name<S: EnumeratorStorage<$getting>>(&self, $($param : $param_t ,)* mut storage: S) -> Result<S::InitStorage, vk_safe_sys::Result> {
            use std::convert::TryInto;
            let query_len = || {
                let mut num = 0;
                let res;
                unsafe {
                    res = self.commands.get().get_fptr()($($param.to_c(),)* &mut num, std::ptr::null_mut());
                    check_raw_err!(res);
                }
                Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
            };
            storage.query_len(query_len)?;
            let uninit_slice = storage.uninit_slice();
            let mut len = VulkanLenType::from_usize(uninit_slice.len());
            let res;
            unsafe {
                res = self.commands.get().get_fptr()($($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
                check_raw_err!(res);
            }
            Ok(storage.finalize(len.to_usize()))
        }
    };
}

// enumerators are all very similar, so why repeat ourselves
macro_rules! enumerator_code2 {
    ( $handle:expr, $commands:expr; ( $($param:ident : $param_t:ty),* ) -> $storage:ident ) => {{
        use std::convert::TryInto;
        let query_len = || {
            let mut num = 0;
            let res;
            unsafe {
                res = $commands.get().get_fptr()($handle, $($param.to_c(),)* &mut num, std::ptr::null_mut());
                check_raw_err!(res);
            }
            Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
        };
        $storage.query_len(query_len)?;
        let uninit_slice = $storage.uninit_slice();
        let mut len = crate::enumerator_storage::VulkanLenType::from_usize(uninit_slice.len());
        let res;
        unsafe {
            res = $commands.get().get_fptr()($handle, $($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
            check_raw_err!(res);
        }
        $storage.finalize(len.to_usize())
    }};
}

// enumerators are all very similar, so why repeat ourselves
macro_rules! enumerator_code_non_fail {
    ( $handle:expr, $commands:expr; ( $($param:ident : $param_t:ty),* ) -> $storage:ident ) => {{
        use std::convert::TryInto;
        use crate::enumerator_storage::VulkanLenType;
        let query_len = || {
            let mut num = 0;
            unsafe {
                let _: () = $commands.get().get_fptr()($handle, $($param.to_c(),)* &mut num, std::ptr::null_mut());
            }
            Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
        };
        $storage.query_len(query_len);
        let uninit_slice = $storage.uninit_slice();
        let mut len = crate::enumerator_storage::VulkanLenType::from_usize(uninit_slice.len());
        unsafe {
            let _: () = $commands.get().get_fptr()($handle, $($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
        }
        $storage.finalize(len.to_usize())
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
            unsafe { crate::pretty_version::VkVersion::from_raw(self.$version_param) }
        }
    };
}

macro_rules! array {
    (
        $name:ident, $array_ptr:ident, $array_len:ident, $ty:ty
    ) => {
        pub fn $name(&self) -> &[$ty] {
            unsafe { std::slice::from_raw_parts(self.inner.$array_ptr, self.inner.$array_len as usize) }
        }
    };
}

macro_rules! verify_params {
    ( $name:ident( $( $param:ident : $trait:path ),* ) { $($code:tt)* } ) => {
        #[allow(non_camel_case_types)]
        struct $name<$($param: $trait),*>( $( std::marker::PhantomData<$param> ),* );

        impl<$($param: $trait),*> $name<$($param),*> {
            const VERIFY: () = {
                $($code)*
            };
            #[track_caller]
            fn verify($(_: $param),*){ let _ = Self::VERIFY; }
        }
    };
}

macro_rules! verify_vuids {
    ( $vis:vis $name:ident( $( $param:ident : $trait:path ),* ) { $($code:tt)* } ) => {
        $vis struct $name<$($param: $trait),*>( $( std::marker::PhantomData<$param> ),* );

        impl<$($param: $trait),*> $name<$($param),*> {
            fn new() -> Self {
                Self (
                    $(std::marker::PhantomData::<$param>::default()),*
                )
            }
            #[track_caller]
            $vis fn verify($(_: $param),*){
                validate(Self::new()); // validate should be imported in the scope for the type being validated
            }
        }

        #[allow(non_upper_case_globals)]
        impl<$($param: $trait),*> Vuids for $name<$($param),*> { // Vuids should be in the scope
            $($code)*
        }
    };
}


// TODO, exported macro probably belong somewhere else
#[macro_export]
macro_rules! bitmask {
    ( $($bit:ident)|* ) => {
        krs_hlist::hlist!( $( $bit ),* )
    };
    ( $path:path : $($bit:ident)|* ) => {
        krs_hlist::hlist!( $( $path::$bit ),* )
    };
}

// check things that looks like this from where they are defined
// const VUID_vkEnumeratePhysicalDevices_instance_parameter: &'static str = "instance must be a valid VkInstance handle";
// when implementing vuid checks, create definitions checkers to ensure that if the definition changes later, we know to update our check
macro_rules! check_vuid_defs {
    ( $( pub const $vuid:ident : &'static [u8] = $def:expr ;)* ) => {
        #[allow(non_upper_case_globals)]
        #[allow(unused)]
        const CHECK_DEF: () = {
            $(
                match $def {
                    $vuid => {}
                    _ => panic!(concat!("definition for ", stringify!($vuid), " has been updated")),
                }
            )*
        };
    };
}