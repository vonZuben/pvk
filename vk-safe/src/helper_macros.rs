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
                    res = self.commands.get()($($param.to_c(),)* &mut num, std::ptr::null_mut());
                    check_raw_err!(res);
                }
                Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
            };
            storage.query_len(query_len)?;
            let uninit_slice = storage.uninit_slice();
            let mut len = VulkanLenType::from_usize(uninit_slice.len());
            let res;
            unsafe {
                res = self.commands.get()($($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
                check_raw_err!(res);
            }
            Ok(storage.finalize(len.to_usize()))
        }
    };
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