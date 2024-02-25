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
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $($e_provider:ident),+ $(,)? )? ) => {
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
                    $crate::version::instance::$v_provider!($name);
                    impl $crate::commands::Version for $name {
                        const VERSION_TRIPLE: (u32, u32, u32) = $crate::version::numbers::$v_provider;
                    }
                )?

                $(
                    $(
                        $crate::extension::instance::$e_provider!($name);
                    )+
                )?

                #[allow(non_snake_case)]
                pub struct $name {
                    $( $v_provider: $crate::version::instance::structs::$v_provider, )?
                    $( $($e_provider: $crate::extension::instance::structs::$e_provider),+ )?
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $($v_provider: $crate::version::instance::structs::$v_provider::load(loader)?,)?
                                $( $($e_provider: $crate::extension::instance::structs::$e_provider::load(loader)?),+ )?
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
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $($e_provider:ident),+ $(,)? )? ) => {
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
                    $crate::version::device::$v_provider!($name);
                    impl $crate::commands::Version for $name {
                        const VERSION_TRIPLE: (u32, u32, u32) = $crate::version::numbers::$v_provider;
                    }
                )?

                $(
                    $(
                        $crate::extension::device::$e_provider!($name);
                    )+
                )?

                #[allow(non_snake_case)]
                pub struct $name {
                    $( $v_provider: $crate::version::device::structs::$v_provider, )?
                    $( $($e_provider: $crate::extension::device::structs::$e_provider),+ )?
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $($v_provider: $crate::version::device::structs::$v_provider::load(loader)?,)?
                                $( $($e_provider: $crate::extension::device::structs::$e_provider::load(loader)?),+ )?
                            }
                        )
                    }
                }
            }
        }
    }
}
