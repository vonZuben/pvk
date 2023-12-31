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
                    use $crate::version::instance::$v_provider;
                    $v_provider!($name);
                    impl $crate::commands::Version for $name {
                        const VERSION_TRIPLE: (u32, u32, u32) = <$crate::$v_provider as $crate::VulkanVersion>::VersionTriple;
                    }
                )?

                $(
                    $(
                        use $crate::extension::instance::$e_provider;
                        $e_provider!($name);
                    )+
                )?

                #[allow(non_snake_case)]
                pub struct $name {
                    $($v_provider: <$crate::$v_provider as $crate::VulkanVersion>::InstanceCommands,)?
                    $( $($e_provider: <$crate::$e_provider as $crate::VulkanExtension>::InstanceCommands),+ )?
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $($v_provider: <$crate::$v_provider as $crate::VulkanVersion>::InstanceCommands::load(loader)?,)?
                                $( $($e_provider: <$crate::$e_provider as $crate::VulkanExtension>::InstanceCommands::load(loader)?),+ )?
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
                    use $crate::version::device::$v_provider;
                    $v_provider!($name);
                    impl $crate::commands::Version for $name {
                        const VERSION_TRIPLE: (u32, u32, u32) = <$crate::$v_provider as $crate::VulkanVersion>::VersionTriple;
                    }
                )?

                $(
                    $(
                        use $crate::extension::device::$e_provider;
                        $e_provider!($name);
                    )+
                )?

                #[allow(non_snake_case)]
                pub struct $name {
                    $($v_provider: <$crate::$v_provider as $crate::VulkanVersion>::DeviceCommands,)?
                    $( $($e_provider: <$crate::$e_provider as $crate::VulkanExtension>::DeviceCommands),+ )?
                }

                impl $crate::LoadCommands for $name {
                    fn load(loader: impl $crate::FunctionLoader) -> std::result::Result<Self, $crate::CommandLoadError> {
                        Ok(
                            Self {
                                $($v_provider: <$crate::$v_provider as $crate::VulkanVersion>::DeviceCommands::load(loader)?,)?
                                $( $($e_provider: <$crate::$e_provider as $crate::VulkanExtension>::DeviceCommands::load(loader)?),+ )?
                            }
                        )
                    }
                }
            }
        }
    }
}
