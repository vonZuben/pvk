use crate::type_conversions::ToC;
use vk_safe_sys as vk;

use crate::scope::{RefScope, Scope};

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

impl<P, Cmd> InstanceConfig for Config<P, Cmd>
where
    Cmd: LoadCommands + DestroyInstance<P> + Version,
{
    const VERSION: VkVersion = VkVersion::new(
        Cmd::VERSION_TRIPLE.0,
        Cmd::VERSION_TRIPLE.1,
        Cmd::VERSION_TRIPLE.2,
    );
    type DropProvider = P;
    type Commands = Cmd;
}

pub type ScopedInstanceType<S, C> = RefScope<S, InstanceType<C>>;

pub trait Instance:
    std::ops::Deref<Target = ScopedInstanceType<Self, Self::Config>> + Copy
{
    type Config: InstanceConfig<Commands = Self::Commands>;
    type Commands;
}

impl<'scope, C: InstanceConfig> Instance for Scope<'scope, InstanceType<C>> {
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
        check_vuids::check_vuids!(DestroyInstance);

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00629: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "All child objects created using instance must have been destroyed prior to destroying"
            "instance"
            }

            // all child objects borrow the instance, and *normally* they are dropped/destroyed before the instance is destroyed
            // However, it is well known that rust does not guarantee that values will be dropped. Thus, we cannot enforce this rule
            // In any event, if a child object is not dropped (e.g. forgotten), it should never be used again or dropped. Thus, even if the Instance is
            // dropped, the child objects are merely leaked, and it is "assumed" that this is no real issue even in Vulkan.
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00630: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If VkAllocationCallbacks were provided when instance was created, a compatible set"
            "of callbacks must be provided here"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00631: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If no VkAllocationCallbacks were provided when instance was created, pAllocator must"
            "be NULL"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If instance is not NULL, instance must be a valid VkInstance handle"
            }

            // always a valid handle from creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        unsafe { self.commands.DestroyInstance().get_fptr()(self.handle, None.to_c()) }
    }
}

mod command_impl_prelude {
    pub use super::ScopedInstanceType;
    pub use crate::array_storage::{ArrayStorage, VulkanLenType};
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
