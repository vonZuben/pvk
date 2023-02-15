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
        fn $fn_name<S: EnumeratorStorage<$getting>>(&self, $($param : $param_t ,)* mut storage: S) -> Result<S::InitStorage> {
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
