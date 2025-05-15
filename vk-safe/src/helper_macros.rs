macro_rules! make_enumerator {
    ( $command:expr; ( $($param:expr),* ) ) => {{
        #[allow(unused)]
        use $crate::type_conversions::ToC;
        $crate::enumerator::EnumeratorClosure::new(move |len: *mut _, buffer: *mut _| unsafe { $command($($param.to_c(),)* len, buffer) })
    }};
}

pub(crate) fn str_len(s: &[std::ffi::c_char]) -> usize {
    s.iter().take_while(|&&c| c != 0).count()
}

// Use this to create wrappers around simple structs that are scoped
macro_rules! struct_wrapper {
    // take macro input and pack it
    // need to pack the generics into a single tt so it can
    // be expanded multiple times in each "traits" match
    (
        $( #[$($attributes:tt)*] )*
        $name:ident
        $(< $( $lt:lifetime , )* $( $(& $tlt:lifetime)? $ty:ident , )* >)?
        $(impl $($traits:ident),+ $(,)?)?
    ) => {
        struct_wrapper!(
            @PACKED
            { $(#[$($attributes)*])* }
            $name
            { $( $( $lt , )* $( $(& $tlt)? $ty , )* )? }
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
        struct_wrapper!(
            @DEF
            $attributes
            $name
            $generics
        );

        $(
            struct_wrapper!(
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
        { $( $lt:lifetime , )* $( $(& $tlt:lifetime)? $ty:ident , )* }
    ) => {
        $($attributes)*
        #[repr(transparent)]
        #[allow(non_snake_case)]
        pub struct $name<$( $lt , )* $( $ty, )*> {
            inner: vk_safe_sys::$name,
            lifetimes: std::marker::PhantomData< ( $( & $lt (), )* ) >,
            types: std::marker::PhantomData< ( $( $(& $tlt)? $ty , )* ) >,
            // $( $($generics)?: std::marker::PhantomData<$(& $lt)? $($generics)?>, )*
        }

        unsafe impl<$( $lt , )* $( $ty , )*>
            crate::type_conversions::ConvertWrapper<vk_safe_sys::$name>
            for $name<$( $lt , )* $( $ty , )*> {}

        unsafe impl<$( $lt , )* $( $ty , )*>
            crate::type_conversions::Wrapper
            for $name<$( $lt , )* $( $ty , )*> {
                type Wrapped = vk_safe_sys::$name;
            }
    };

    // generate any optional trait implementations

    ( @IMPL Deref $name:ident { $( $lt:lifetime , )* $( $(& $tlt:lifetime)? $ty:ident , )* }) => {
        impl<$( $lt , )* $( $ty, )*> std::ops::Deref for $name<$( $lt , )* $( $ty, )*> {
            type Target = vk_safe_sys::$name;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };

    ( @IMPL Debug $name:ident { $( $lt:lifetime , )* $( $(& $tlt:lifetime)? $ty:ident , )* }) => {
        impl<$( $lt , )* $( $ty, )*> std::fmt::Debug for $name<$( $lt , )* $( $ty, )*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.inner.fmt(f)
            }
        }
    };

    ( @IMPL Clone $name:ident { $( $lt:lifetime , )* $( $(& $tlt:lifetime)? $ty:ident , )* }) => {
        impl<$( $lt , )* $( $ty, )*> Clone for $name<$( $lt , )* $( $ty, )*> {
            fn clone(&self) -> Self {
                *self
            }
        }
    };

    ( @IMPL Copy $name:ident { $( $lt:lifetime , )* $( $(& $tlt:lifetime)? $ty:ident , )* }) => {
        impl<$( $lt , )* $( $ty, )*> Copy for $name<$( $lt , )* $( $ty, )*> { }
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
        $( #[cfg($feature:ident)] $block:tt );* $(;)?
    ) => {
        $( pub_use_modules!(@INNER $feature $block); )*
    };
    (
        @INNER
        $feature:ident
        {
            $(
                $(#[$($attributes:tt)*])*
                $name:ident
            );*
            $(;)?
        }
    ) => {
        $(
            #[cfg($feature)]
            $(#[$($attributes)*])*
            mod $name;
            #[allow(unused_imports)]
            pub use $name::*;
        )*
    };
}

macro_rules! handle_command_collection_trait {
    (
        NAME( $command_trait_name:ident )
        IMPL $implementor:tt
        $(
            #[cfg($cfg_trait_name:ident)] {
                $($fn_trait_name:ident),*
                $(,)?
            }
        )+
    ) => {

        mod commands {
            #![allow(non_snake_case)]
            $(
                $(
                    #[cfg($cfg_trait_name)]
                    pub mod $fn_trait_name;
                )*
            )+
        }

        $(
            $(
                #[cfg($cfg_trait_name)]
                pub use commands::$fn_trait_name::$fn_trait_name;
                handle_command_collection_trait!(@IMPL $cfg_trait_name $fn_trait_name $implementor );
            )*
        )*

        mod cfg_trait {
            $(
                #[cfg($cfg_trait_name)]
                #[allow(non_camel_case_types)]
                #[doc = concat!("Subtrait for ", stringify!($cfg_trait_name), " commands")]
                pub trait $cfg_trait_name: $( super::$fn_trait_name + )* {}
                #[cfg($cfg_trait_name)]
                impl<T> $cfg_trait_name for T where T: $( super::$fn_trait_name + )* {}

                #[cfg(not($cfg_trait_name))]
                #[allow(non_camel_case_types)]
                #[doc = concat!(stringify!($cfg_trait_name), " is not defined in the version of the Vulkan specification used ot build this")]
                pub trait $cfg_trait_name {}
                #[cfg(not($cfg_trait_name))]
                impl<T> $cfg_trait_name for T {}
            )*
        }

        pub trait $command_trait_name: $( cfg_trait::$cfg_trait_name + )+ {}
        impl<T> $command_trait_name for T where T: $( cfg_trait::$cfg_trait_name + )+ {}

    };
    (
        @IMPL
        $cfg_trait_name:ident
        $fn_trait_name:ident
        ( $implementor_name:ident [ $($generics:tt)* ] $( $bounds:tt )* )
    ) => {
        #[cfg($cfg_trait_name)]
        impl<$($generics)*> $fn_trait_name for $implementor_name<$($generics)*> $( $bounds )* {}
    };
}
