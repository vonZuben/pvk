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
        use crate::type_conversions::ToC;
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
            res = $command($($param.to_c(),)* &mut len, uninit_slice.to_c());
            check_raw_err!(res);
        }
        let ret: Result<_, crate::error::Error> = Ok($storage.finalize(len.to_usize()));
        ret
    }};
}

macro_rules! make_enumerator {
    ( $command:expr; ( $($param:expr),* ) ) => {{
        use crate::type_conversions::ToC;

        struct Enumerator<F, C>(F, std::marker::PhantomData<*mut C>);

        impl<F, C, R, Res: $crate::error::VkResultExt> $crate::enumerator::Enumerator<R> for Enumerator<F, C>
        where
            F: Fn(&mut u32, *mut C) -> Res,
            R: crate::type_conversions::ConvertWrapper<C>,
        {
            fn get_len(&self) -> Result<usize, $crate::error::Error> {
                let mut len = 0;
                // UNSAFE warning
                // the call to this is actually unsafe, but can't be reflected with the Fn trait
                // However, this can only be used internal to this macro, so it is fine
                let res = self.0(&mut len, std::ptr::null_mut());
                check_raw_err!(res);
                Ok(len.try_into()?)
            }

            fn get_enumerate<B: $crate::array_storage::Buffer<R>>(
                &self,
                mut buffer: B,
            ) -> Result<B, $crate::error::Error> {
                let mut len = buffer.capacity().try_into()?;
                // UNSAFE warning
                // the call to this is actually unsafe, but can't be reflected with the Fn trait
                // However, this can only be used internal to this macro, so it is fine
                let res = self.0(&mut len, buffer.ptr_mut().to_c());
                check_raw_err!(res);
                unsafe {
                    buffer.set_len(len.try_into()?);
                }
                Ok(buffer)
            }
        }

        Enumerator( move |len: &mut _, buffer: *mut _| unsafe { $command($($param.to_c(),)* len, buffer) }, std::marker::PhantomData )
    }};
}

pub(crate) fn str_len(s: &[std::ffi::c_char]) -> usize {
    s.iter().take_while(|&&c| c != 0).count()
}

// Use this to create wrappers around simple structs that are scoped
macro_rules! simple_struct_wrapper_scoped {
    // take macro input and pack it
    // need to pack the generics into a single tt so it can
    // be expanded multiple times in each "traits" match
    (
        $( #[$($attributes:tt)*] )*
        $name:ident
        $(<$($generics:ident),*>)?
        $(impl $($traits:ident),+ $(,)?)?
    ) => {
        simple_struct_wrapper_scoped!(
            @PACKED
            { $(#[$($attributes)*])* }
            $name
            { $( $($generics),* )? }
            { $( $($traits),+ )? }
        );
    };
    // handle the packed input, and expand
    // the Def and all optional Trait impls
    (
        @PACKED
        $attributes:tt
        $name:ident
        $generics:tt
        { $($traits:ident),* }
    ) => {
        simple_struct_wrapper_scoped!(
            @DEF
            $attributes
            $name
            $generics
        );

        $(
            simple_struct_wrapper_scoped!(
                @IMPL
                $traits
                $name
                $generics
            );
        )*
    };

    // generate main definition
    (
        @DEF
        { $($attributes:tt)* }
        $name:ident
        { $($generics:ident),* }
    ) => {
        $($attributes)*
        #[repr(transparent)]
        #[allow(non_snake_case)]
        pub struct $name<S, $($generics),*> {
            inner: vk_safe_sys::$name,
            _scope: std::marker::PhantomData<S>,
            $($generics: std::marker::PhantomData<$generics>,)*
        }

        unsafe impl<S, $($generics),*>
            crate::type_conversions::ConvertWrapper<vk_safe_sys::$name>
            for $name<S, $($generics),*> {}

        impl<S, $($generics),*> $name<S, $($generics),*> {
            #[allow(unused)]
            pub(crate) fn new(inner: vk_safe_sys::$name) -> Self {
                Self {
                    inner,
                    _scope: Default::default(),
                    $($generics: Default::default(),)*
                }
            }
        }
    };

    // generate any optional trait implementations

    ( @IMPL Deref $name:ident { $($generics:ident),* }) => {
        impl<S, $($generics),*> std::ops::Deref for $name<S, $($generics),*> {
            type Target = vk_safe_sys::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };

    ( @IMPL Debug $name:ident { $($generics:ident),* }) => {
        impl<S, $($generics),*> std::fmt::Debug for $name<S, $($generics),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }
    };

    ( @IMPL Clone $name:ident { $($generics:ident),* }) => {
        impl<S, $($generics),*> Clone for $name<S, $($generics),*> {
            fn clone(&self) -> Self {
                Self::new(self.inner)
            }
        }
    };

    ( @IMPL Copy $name:ident { $($generics:ident),* }) => {
        impl<S, $($generics),*> Copy for $name<S, $($generics),*> { }
    };
}

macro_rules! input_struct_wrapper {
    (
        $(#[$($attributes:tt)*])*
        $name:ident $(impl $($t:ident),+ $(,)?)?
    ) => {
        $(#[$($attributes)*])*
        #[repr(transparent)]
        pub struct $name<'a, S> {
            pub(crate) inner: vk_safe_sys::$name,
            _params: std::marker::PhantomData<&'a ()>,
            _scope: std::marker::PhantomData<S>,
        }

        unsafe impl<'a, S> crate::type_conversions::ConvertWrapper<vk_safe_sys::$name>
            for $name<'a, S> {}

        $( $( input_struct_wrapper!( @IMPL $t $name ); )+ )?
    };

    ( @IMPL Deref $name:ident ) => {
        impl<S> std::ops::Deref for $name<'_, S> {
            type Target = vk_safe_sys::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };

    ( @IMPL Debug $name:ident ) => {
        impl<S> std::fmt::Debug for $name<'_, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }
    };

    ( @IMPL Clone $name:ident ) => {
        impl<S> Clone for $name<'_, S> {
            fn clone(&self) -> Self {
                Self::new(self.inner)
            }
        }
    };

    ( @IMPL Copy $name:ident ) => {
        impl<S> Copy for $name<'_, S> { }
    };
}

macro_rules! get_str {
    (
        $(#[$($attributes:tt)*])*
        $name:ident
    ) => {
        $(#[$($attributes)*])*
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
        $(#[$($attributes:tt)*])*
        $version_param:ident
    ) => {
        $(#[$($attributes)*])*
        pub fn $version_param(&self) -> crate::VkVersion {
            unsafe { crate::VkVersion::from_raw(self.inner.$version_param) }
        }
    };
}

macro_rules! array {
    (
        $(#[$($attributes:tt)*])*
        $name:ident, $array_ptr:ident, $array_len:ident, $ty:ty
    ) => {
        $(#[$($attributes)*])*
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

/// Publicly include a module and make an `export` module
/// pub uses all the contents for easy exporting
macro_rules! pub_export_modules {
    (
        $(
            $(#[$($attributes:tt)*])*
            $name:ident
        );* $(;)?
    ) => {
        $(
            $(#[$($attributes)*])*
            pub mod $name;
        )*

        pub(crate) mod export {
            $( #[allow(unused_imports)] pub use super::$name::*; )*
        }
    };
}

/// Include a module, and publicly use the modules contents
macro_rules! pub_use_modules {
    (
        $(
            $(#[$($attributes:tt)*])*
            $name:ident
        );* $(;)?
    ) => {
        $(
            $(#[$($attributes)*])*
            mod $name;
            #[allow(unused_imports)]
            pub use $name::*;
        )*
    };
}
