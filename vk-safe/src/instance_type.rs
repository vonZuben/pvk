use crate::type_conversions::ToC;
use vk_safe_sys as vk;

use crate::scope::{RefScope, Scope};

use crate::VkVersion;

use std::marker::PhantomData;

use vk::context::{CommandLoadError, Context, LoadCommands};
use vk::has_command::DestroyInstance;
use vk::Version;

pub trait InstanceConfig {
    const VERSION: VkVersion;
    type Context: DestroyInstance;
}

pub struct Config<C> {
    commands: PhantomData<C>,
}

impl<C> InstanceConfig for Config<C>
where
    C: Context,
    C::Commands: LoadCommands + DestroyInstance + Version,
{
    const VERSION: VkVersion = C::Commands::VERSION;
    type Context = C::Commands;
}

pub type ScopedInstanceType<S, C> = RefScope<S, InstanceType<C>>;

pub trait Instance:
    std::ops::Deref<Target = ScopedInstanceType<Self, Self::Config>> + Copy
{
    type Config: InstanceConfig<Context = Self::Context>;
    type Context;
}

impl<'scope, C: InstanceConfig> Instance for Scope<'scope, InstanceType<C>> {
    type Config = C;
    type Context = C::Context;
}

pub struct InstanceType<C: InstanceConfig> {
    handle: vk::Instance,
    pub(crate) context: C::Context,
}

unsafe impl<C: InstanceConfig> Send for InstanceType<C> {}
unsafe impl<C: InstanceConfig> Sync for InstanceType<C> {}

impl<C: InstanceConfig> InstanceType<C>
where
    C::Context: LoadCommands,
{
    pub(crate) fn load_commands(handle: vk::Instance) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetInstanceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            context: C::Context::load(loader)?,
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
            check_vuids::description! {
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
            check_vuids::description! {
            "If VkAllocationCallbacks were provided when instance was created, a compatible set"
            "of callbacks must be provided here"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_00631: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If no VkAllocationCallbacks were provided when instance was created, pAllocator must"
            "be NULL"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_instance_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If instance is not NULL, instance must be a valid VkInstance handle"
            }

            // always a valid handle from creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyInstance_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO: VkAllocationCallbacks not currently supported
        }

        unsafe { self.context.DestroyInstance().get_fptr()(self.handle, None.to_c()) }
    }
}

mod enumerate_physical_devices;

pub mod instance_exports {
    pub use super::Instance;
}
