use std::ffi::c_char;
use std::fmt;

pub use crate::PFN_vkVoidFunction as VoidFunction;
pub use crate::{CommandLoadError, LoadCommands};
use crate::{FunctionLoader, VulkanCommand};

pub trait Version {
    const VERSION_TRIPLE: (u32, u32, u32);
}

#[macro_export]
macro_rules! instance_context {
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $($e_provider:ident),+ $(,)? )? ) => {
        #[allow(non_snake_case)]
        $vis struct $name {
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

        $(
            $crate::version::instance::$v_provider!($name);

            impl $crate::commands::Version for $name {
                const VERSION_TRIPLE: (u32, u32, u32) = <$crate::$v_provider as $crate::VulkanVersion>::VersionTriple;
            }
        )?

        $(
            $(
                $crate::extension::instance::$e_provider!($name);
            )+
        )?
    }
}

#[macro_export]
macro_rules! device_context {
    ( $vis:vis $name:ident : $($v_provider:ident)? $( + $($e_provider:ident),+ $(,)? )? ) => {
        #[allow(non_snake_case)]
        $vis struct $name {
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

        $(
            $crate::version::device::$v_provider!($name);

            impl $crate::commands::Version for $name {
                const VERSION_TRIPLE: (u32, u32, u32) = <$crate::$v_provider as $crate::VulkanVersion>::VersionTriple;
            }
        )?

        $(
            $(
                $crate::extension::device::$e_provider!($name);
            )+
        )?
    }
}
