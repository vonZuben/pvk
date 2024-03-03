pub use crate::PFN_vkVoidFunction as VoidFunction;
pub use crate::{CommandLoadError, LoadCommands};
use crate::{FunctionLoader, VulkanCommand};

pub trait Version {
    const VERSION: crate::VkVersion;
}

pub trait Commands {
    type Commands: LoadCommands;
}

pub trait Extensions {
    fn list_of_extensions() -> impl AsRef<[*const std::ffi::c_char]>;
}

/// define what API version and extensions should be used with an instance
#[macro_export]
macro_rules! instance_context {
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $e_provider:ident )* ) => {
        #[allow(non_upper_case_globals)]
        $vis const $name: $name::$name = $name::$name;

        #[allow(non_snake_case)]
        pub mod $name {
            #[derive(Copy, Clone)]
            pub struct $name;

            impl $crate::commands::Commands for $name {
                type Commands = commands::$name;
            }

            mod commands {
                $(
                    use $crate::version::instance::traits::$v_provider; // this is here so that rust analyzer auto complete can provide good suggestions see (https://blog.emi0x7d1.dev/improving-autocompletion-in-your-rust-macros/)
                    $crate::version::instance::macros::$v_provider!($name);
                    impl $crate::commands::Version for $name {
                        const VERSION: $crate::VkVersion = $crate::VkVersion::from_triple($crate::version::numbers::$v_provider);
                    }
                )?

                $(
                    use $crate::extension::instance::traits::$e_provider; // this is here for autocomplete (see above)
                    impl $crate::dependencies::traits::$e_provider for $name {}
                    $crate::extension::instance::macros::$e_provider!($name);
                    const _ : () = {
                        $crate::dependencies::instance::$e_provider::check_dependencies(std::marker::PhantomData::<$name>)
                    };
                )*

                impl $crate::commands::Extensions for super::$name {
                    fn list_of_extensions() -> impl AsRef<[*const std::ffi::c_char]> {
                        use std::ffi::c_char;
                        use $crate::commands::macro_helper::*;
                        let l = End;
                        $( $crate::dependencies::instance::$e_provider!(l); )*
                        l
                    }
                }

                #[allow(non_snake_case)]
                pub struct $name {
                    $( $v_provider: $crate::version::instance::structs::$v_provider, )?
                    $( $e_provider: $crate::extension::instance::structs::$e_provider, )*
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $( $v_provider: $crate::version::instance::structs::$v_provider::load(loader)?, )?
                                $( $e_provider: $crate::extension::instance::structs::$e_provider::load(loader)?, )*
                            }
                        )
                    }
                }
            }
        }
    }
}

/// define what API version and extensions should be used with a device
#[macro_export]
macro_rules! device_context {
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $e_provider:ident)* ) => {
        #[allow(non_upper_case_globals)]
        $vis const $name: $name::$name = $name::$name;

        #[allow(non_snake_case)]
        pub mod $name {
            #[derive(Copy, Clone)]
            pub struct $name;

            impl $crate::commands::Commands for $name {
                type Commands = commands::$name;
            }

            mod commands {
                $(
                    use $crate::version::device::traits::$v_provider; // this is here so that rust analyzer auto complete can provide good suggestions see (https://blog.emi0x7d1.dev/improving-autocompletion-in-your-rust-macros/)
                    $crate::version::device::macros::$v_provider!($name);
                    impl $crate::commands::Version for $name {
                        const VERSION: $crate::VkVersion = $crate::VkVersion::from_triple($crate::version::numbers::$v_provider);
                    }
                )?

                $(
                    use $crate::extension::device::traits::$e_provider; // this is here for autocomplete (see above)
                    impl $crate::dependencies::traits::$e_provider for $name {}
                    $crate::extension::device::macros::$e_provider!($name);
                    const _ : () = {
                        $crate::dependencies::device::$e_provider::check_dependencies(std::marker::PhantomData::<$name>)
                    };
                )*

                impl $crate::commands::Extensions for super::$name {
                    fn list_of_extensions() -> impl AsRef<[*const std::ffi::c_char]> {
                        use std::ffi::c_char;
                        use $crate::commands::macro_helper::*;
                        let l = End;
                        $( $crate::dependencies::device::$e_provider!(l); )*
                        l
                    }
                }

                #[allow(non_snake_case)]
                pub struct $name {
                    $( $v_provider: $crate::version::device::structs::$v_provider, )?
                    $( $e_provider: $crate::extension::device::structs::$e_provider, )*
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $( $v_provider: $crate::version::device::structs::$v_provider::load(loader)?, )?
                                $( $e_provider: $crate::extension::device::structs::$e_provider::load(loader)?, )*
                            }
                        )
                    }
                }
            }
        }
    }
}

/// The below code is for a trick to simplify the above macros
///
/// Ultimately, in the above macros, we want to build an array or slice of c-strings, which are names of extensions to enable, based on the
/// names provided by the user. However, some of the names provided by the user do not have a corresponding name to include in said list.
/// Thus, we cannot do a simple expansion of expressions (that evaluate to the name of an extension to enable), since some expressions would
/// be empty, and invalid. We can do a simple expansion to a list of items (e.g. statements), but then how do we create an array or slice
/// from a list of items?
///
/// My answer is to incrementally build a sort of list type based on the Hlist trick from crates like frunk.
/// An initial empty list is represented by End.
/// An item "T" can be appended to a list "C" by wrapping it with R<C, T>
///
/// The Len and ListOf traits ensure that a given R<C, T> (with repr(C)) can soundly be reinterpreted as a slice of T
///
/// Of course, we could just do more advanced macro tricks like incremental token munchers to more directly construct an array or slice, but this has
/// unfortunate compile time effects. I prefer to keep the macro as simple as possible to keep compile times down. The above macros can expand without any recursion.
#[doc(hidden)]
pub mod macro_helper {
    #[repr(C)]
    pub struct R<C, T>(pub C, pub T);

    #[repr(C)]
    pub struct End;

    /// provide the number of elements in the list
    pub unsafe trait Len {
        const LEN: usize;
    }

    unsafe impl Len for End {
        const LEN: usize = 0;
    }

    unsafe impl<C: Len, T> Len for R<C, T> {
        const LEN: usize = 1 + C::LEN;
    }

    /// ensure each element in the list is the same type
    pub unsafe trait ListOf<T> {}

    unsafe impl<T> ListOf<T> for End {}

    unsafe impl<C: ListOf<T>, T> ListOf<T> for R<C, T> {}

    impl End {
        pub const fn as_slice<'a, T>(&'a self) -> &'a [T] {
            &[]
        }
    }

    impl<C, T> R<C, T>
    where
        Self: ListOf<T> + Len,
    {
        pub const fn as_slice<'a>(&'a self) -> &'a [T] {
            let ptr: *const T = self as *const Self as *const T;
            unsafe { std::slice::from_raw_parts(ptr, Self::LEN) }
        }
    }

    impl<T> AsRef<[T]> for End {
        fn as_ref(&self) -> &[T] {
            self.as_slice()
        }
    }

    impl<C, T> AsRef<[T]> for R<C, T>
    where
        Self: ListOf<T> + Len,
    {
        fn as_ref(&self) -> &[T] {
            self.as_slice()
        }
    }
}
