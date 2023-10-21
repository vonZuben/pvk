use crate::safe_interface::type_conversions::ToC;
use vk_safe_sys as vk;

use crate::scope::{Scope, Scoped};

use crate::pretty_version::VkVersion;

use std::marker::PhantomData;

use vk::commands::{CommandLoadError, LoadCommands, Version};
use vk::has_command::DestroyInstance;

pub trait InstanceConfig {
    const VERSION: VkVersion;
    type DropProvider;
    type Commands: DestroyInstance<Self::DropProvider> + LoadCommands + Version;
}

pub struct Config<P, Cmd> {
    _drop_provider: PhantomData<P>,
    _commands: PhantomData<Cmd>,
}

impl<P, Cmd> Clone for Config<P, Cmd> {
    fn clone(&self) -> Self {
        Self {
            _drop_provider: PhantomData,
            _commands: PhantomData,
        }
    }
}

impl<P, Cmd> Copy for Config<P, Cmd> {}

impl<P> Config<P, ()> {
    pub fn new<Cmd>() -> Config<P, Cmd>
    where
        Cmd: DestroyInstance<P>,
    {
        Config {
            _drop_provider: PhantomData,
            _commands: PhantomData,
        }
    }
}

impl<P, Cmd> InstanceConfig for Config<P, Cmd>
where
    Cmd: LoadCommands + DestroyInstance<P> + Version,
{
    const VERSION: VkVersion = VkVersion::new(
        Cmd::VersionTriple.0,
        Cmd::VersionTriple.1,
        Cmd::VersionTriple.2,
    );
    type DropProvider = P;
    type Commands = Cmd;
}

pub type ScopedInstanceType<'scope, C> = Scope<'scope, InstanceType<C>>;

pub trait Instance: Scoped + std::ops::Deref<Target = InstanceType<Self::Config>> + Copy {
    type Config: InstanceConfig<Commands = Self::Commands>;
    type Commands;
}

impl<'scope, C: InstanceConfig> Instance for ScopedInstanceType<'scope, C> {
    type Config = C;
    type Commands = C::Commands;
}

pub struct InstanceType<C: InstanceConfig> {
    handle: vk::Instance,
    pub(crate) commands: C::Commands,
}

impl<C: InstanceConfig> InstanceType<C> {
    pub(crate) fn load_commands(handle: vk::Instance) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetInstanceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            commands: C::Commands::load(loader)?,
        })
    }
}

impl<C: InstanceConfig> std::fmt::Debug for InstanceType<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("handle", &self.handle)
            .field("version", &C::VERSION)
            .finish()
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyInstance.html
*/
impl<C: InstanceConfig> Drop for InstanceType<C> {
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

        unsafe { self.commands.DestroyInstance().get_fptr()(self.handle, None.to_c()) }
    }
}

mod command_impl_prelude {
    pub use super::ScopedInstanceType;
    pub use crate::array_storage::{ArrayStorage, VulkanLenType};
    pub use crate::safe_interface::type_conversions::*;
    pub use vk_safe_sys as vk;
    pub use vk_safe_sys::{VulkanExtension, VulkanVersion};
}

// This is how each safe command can be implemented on top of each raw command
// macro_rules! impl_safe_instance_interface {
//     ( $interface:ident { $($code:tt)* }) => {
//         impl<'scope, C: InstanceConfig> ScopedInstance<'scope, C>
//         where
//             C::Commands: GetCommand<vk::$interface> {
//             $($code)*
//         }
//     };
// }

mod enumerate_physical_devices;

pub use enumerate_physical_devices::*;
