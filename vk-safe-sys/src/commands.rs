pub use crate::PFN_vkVoidFunction as VoidFunction;
pub use crate::{CommandLoadError, LoadCommands};
use crate::{FunctionLoader, VulkanCommand};

pub trait Version {
    const VERSION_TRIPLE: (u32, u32, u32);
}

pub trait Commands {
    type Commands: LoadCommands;
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
                        const VERSION_TRIPLE: (u32, u32, u32) = $crate::version::numbers::$v_provider;
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
                        const VERSION_TRIPLE: (u32, u32, u32) = $crate::version::numbers::$v_provider;
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
