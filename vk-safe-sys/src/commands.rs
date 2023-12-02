use std::ffi::c_char;
use std::fmt;

pub use crate::PFN_vkVoidFunction as VoidFunction;
pub use crate::{CommandLoadError, LoadCommands};
use crate::{FunctionLoader, VulkanCommand};

pub trait Version {
    const VersionTriple: (u32, u32, u32);
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
            impl $crate::version::instance::provider::$v_provider for $name {
                fn $v_provider(&self) -> &$crate::version::instance::$v_provider {
                    &self.$v_provider
                }
            }

            impl $crate::commands::Version for $name {
                const VersionTriple: (u32, u32, u32) = <$crate::$v_provider as $crate::VulkanVersion>::VersionTriple;
            }
        )?

        $(
            $(
                impl $crate::extension::instance::provider::$e_provider for $name {
                    fn $e_provider(&self) -> &$crate::extension::instance::$e_provider {
                        &self.$e_provider
                    }
                }
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
            impl $crate::version::device::provider::$v_provider for $name {
                fn $v_provider(&self) -> &$crate::version::device::$v_provider {
                    &self.$v_provider
                }
            }

            impl $crate::commands::Version for $name {
                const VersionTriple: (u32, u32, u32) = <$crate::$v_provider as $crate::VulkanVersion>::VersionTriple;
            }
        )?

        $(
            $(
                impl $crate::extension::device::provider::$e_provider for $name {
                    fn $e_provider(&self) -> &$crate::extension::device::$e_provider {
                        &self.$e_provider
                    }
                }
            )+
        )?
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn command_load_test() {
        // use crate::generated::VERSION_1_0;
        use crate::command::DestroyInstance;
        use crate::version::instance::provider::VERSION_1_0;

        let mut instance = crate::Instance {
            handle: std::ptr::null(),
        };
        let loader = |name| {
            // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
            unsafe { crate::GetInstanceProcAddr(instance, name) }
        };

        instance_context!(MyCx: VERSION_1_0);

        let create_instance = crate::CreateInstance::load(loader).unwrap();

        let mut info =
            unsafe { std::mem::MaybeUninit::<crate::InstanceCreateInfo>::zeroed().assume_init() };
        info.s_type = crate::StructureType::INSTANCE_CREATE_INFO;

        unsafe { create_instance.get_fptr()(&info, std::ptr::null(), &mut instance) };

        // reset since otherwise instance borrow is aliased
        let loader = |name| {
            // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
            unsafe { crate::GetInstanceProcAddr(instance, name) }
        };

        let instance_commands = MyCx::load(loader).unwrap();

        println!("{:p}", instance_commands.DestroyInstance().get_fptr());

        // println!("{:?}", instance);

        // let loader = |name| {
        //     // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
        //     unsafe { crate::GetInstanceProcAddr(instance, name) }
        // };
    }
}
