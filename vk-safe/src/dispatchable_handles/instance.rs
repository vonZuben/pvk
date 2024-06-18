//! Main Vulkan object
//!
//! This is the main object you create ([`create_instance`](crate::vk::create_instance))
//! in Vulkan that stores all application state. The primary thing you will want to do with
//! an Instance is enumerate the PhysicalDevices on the system ([`enumerate_physical_devices`])
//!
//! Vulkan doc:
//! <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstance.html>

pub mod enumerate_physical_devices;

use crate::scope::HandleScope;

/** Instance handle trait

Represents a *specific* Instance which has been scoped.
*/
pub trait Instance: HandleScope<concrete_type::Instance<Self::Config>> {
    #[doc(hidden)]
    type Config: concrete_type::InstanceConfig<Context = Self::Context>;
    /// Instance context such as the Version and Extensions being used
    type Context;
}

/// concrete type for a created Instance
///
/// Do not use this type directly. Instead, after creating an instance you should
/// use [`scope!`](crate::vk::scope) as soon as possible, and then generically use
/// [`Instance`]
pub use concrete_type::Instance as ConcreteInstance;

pub(crate) mod concrete_type {
    use crate::type_conversions::ToC;
    use vk_safe_sys as vk;

    use crate::scope::{Scope, SecretScope, Shareable, ToScope};

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

    pub type ScopedInstance<S, C> = SecretScope<S, Instance<C>>;

    impl<C: InstanceConfig> super::Instance for Scope<'_, Instance<C>> {
        type Config = C;
        type Context = C::Context;
    }

    pub struct Instance<C: InstanceConfig> {
        pub(crate) handle: vk::Instance,
        pub(crate) context: C::Context,
    }

    unsafe impl<C: InstanceConfig> Shareable for Instance<C> {}

    impl<C: InstanceConfig> ToScope for Instance<C> {}

    unsafe impl<C: InstanceConfig> Send for Instance<C> {}
    unsafe impl<C: InstanceConfig> Sync for Instance<C> {}

    impl<C: InstanceConfig> Instance<C>
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

    impl<C: InstanceConfig> std::fmt::Debug for Instance<C> {
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
    impl<C: InstanceConfig> Drop for Instance<C> {
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
}
