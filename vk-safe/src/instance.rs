use crate::safe_interface::type_conversions::ToC;
use krs_hlist::Get;
use vk_safe_sys as vk;

use crate::scope::Scope;

use crate::pretty_version::VkVersion;

use std::marker::PhantomData;

use vk::commands::{CommandLoadError, LoadCommands};

pub trait InstanceConfig {
    const VERSION: VkVersion;
    type Commands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyInstance>;
}

#[derive(Debug)]
pub struct Config<V, E> {
    _version: PhantomData<V>,
    _extension: PhantomData<E>,
}

impl<V, E> Clone for Config<V, E> {
    fn clone(&self) -> Self {
        Self { _version: PhantomData, _extension: PhantomData }
    }
}

impl<V, E> Copy for Config<V, E> {}

impl<V, E> Config<V, E> {
    pub fn new(_version: V, _extensions: E) -> Self {
        Self { _version: PhantomData, _extension: PhantomData }
    }
}

impl<V: vk::VulkanVersion, E: vk::VulkanExtension> InstanceConfig for Config<V, E>
where
    V::InstanceCommands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyInstance>,
    E::InstanceCommands : vk::commands::LoadCommands,
{
    const VERSION: VkVersion = VkVersion::new(V::VersionTriple.0, V::VersionTriple.1, V::VersionTriple.2);

    type Commands = V::InstanceCommands;
}

pub type ScopedInstance<'scope, C> = Scope<'scope, Instance<C>>;

// pub trait ScopedInstance<'scope, C: InstanceConfig> {
//     type Commands: C::
// }

#[derive(Debug)]
pub struct Instance<C: InstanceConfig> {
    handle: vk::Instance,
    pub(crate) commands: C::Commands,
}

impl<C: InstanceConfig> Instance<C> {
    pub(crate) fn load_commands(handle: vk::Instance) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetInstanceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            commands: C::Commands::load(loader)?,
        })
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyInstance.html
*/
impl<C: InstanceConfig> Drop for Instance<C> {
    fn drop(&mut self) {

        check_vuid_defs2!( DestroyInstance
            pub const VUID_vkDestroyInstance_instance_00629 : & 'static [ u8 ] = "All child objects created using instance must have been destroyed prior to destroying instance" . as_bytes ( ) ;
            CHECK {
                // it is possible to forget child objects such that they are nto destroyed
                // However, I believe this is at worst a memory leak issue and will never cause undefined behavior
            }
            pub const VUID_vkDestroyInstance_instance_00630 : & 'static [ u8 ] = "If VkAllocationCallbacks were provided when instance was created, a compatible set of callbacks must be provided here" . as_bytes ( ) ;
            CHECK {
                // *******************************************
                // ******************TODO*********************
                // *******************************************
                // when implemented, check this
                // probably the instance object will hold its allocator and automatically use it in drop
            }
            pub const VUID_vkDestroyInstance_instance_00631 : & 'static [ u8 ] = "If no VkAllocationCallbacks were provided when instance was created, pAllocator must be NULL" . as_bytes ( ) ;
            CHECK {
                // *******************************************
                // ******************TODO*********************
                // *******************************************
                // when implemented, check this
                // probably the instance object will hold its allocator and automatically use it in drop
            }
            pub const VUID_vkDestroyInstance_instance_parameter: &'static [u8] =
                "If instance is not NULL, instance must be a valid VkInstance handle".as_bytes();
            CHECK {
                // Instance must have been created with a valid handle, so only valid handle should be dropped
            }
            pub const VUID_vkDestroyInstance_pAllocator_parameter : & 'static [ u8 ] = "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure" . as_bytes ( ) ;
            CHECK {
                // *******************************************
                // ******************TODO*********************
                // *******************************************
                // when implemented, check this
                // probably the instance object will hold its allocator and automatically use it in drop
            }
        );

        unsafe { self.commands.get().get_fptr()(self.handle, None.to_c()) }
    }
}

mod command_impl_prelude {
    pub use super::ScopedInstance;
    pub use crate::array_storage::{ArrayStorage, VulkanLenType};
    pub use crate::safe_interface::type_conversions::*;
    pub use vk_safe_sys as vk;
    pub use vk_safe_sys::{GetCommand, VulkanExtension, VulkanVersion};
}

// This is how each safe command can be implemented on top of each raw command
macro_rules! impl_safe_instance_interface {
    ( $interface:ident { $($code:tt)* }) => {
        impl<'scope, C: InstanceConfig> ScopedInstance<'scope, C>
        where
            C::Commands: GetCommand<vk::$interface> {
            $($code)*
        }
    };
}

mod enumerate_physical_devices;

pub use enumerate_physical_devices::*;