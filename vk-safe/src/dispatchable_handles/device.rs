//! Vulkan logical Device
//!
//! A [`Device`] can be created with [`create_device`](crate::dispatchable_handles::physical_device::concrete_type::ScopedPhysicalDevice::create_device)
//! on from a scoped [`PhysicalDevice`](crate::dispatchable_handles::physical_device::PhysicalDevice).
//!
//! Vulkan doc:
//! <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap5.html#devsandqueues-devices>

pub_export_modules!(
allocate_memory;
get_device_queue;
map_memory;
unmap_memory;
wait_idle;
);

use super::ScopedDispatchableHandle;

/// Device handle trait
///
/// Represents a *specific* Device which has been scoped.
///
/// See the available methods on [`_Device`]
///
/// You may note that there are no visible implementors of this trait.
/// You are only ever intended to use opaque implementors of this trait
/// as seen with the return type of [`create_device`](crate::dispatchable_handles::physical_device::concrete_type::ScopedPhysicalDevice::create_device)
pub trait Device: ScopedDispatchableHandle<concrete_type::Device<Self::Config>> {
    #[doc(hidden)]
    type Config: concrete_type::DeviceConfig<
        Context = Self::Context,
        PhysicalDevice = Self::PhysicalDevice,
    >;
    /// The *specific* PhysicalDevice from which this logical Device was created
    type PhysicalDevice;
    /// Device context such as the Version and Extensions being used
    type Context;
}

#[cfg(doc)]
/// Example of concrete Device
///
/// Given some <code>D: [Device]</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This type alias is only generated for the documentation and is not usable in your code.
pub type _Device<S, C> = crate::scope::SecretScope<S, concrete_type::Device<C>>;

pub(crate) mod concrete_type {
    use vk_safe_sys as vk;

    use std::marker::PhantomData;

    use crate::dispatchable_handles::physical_device::{
        create_device::DeviceQueueCreateInfo, PhysicalDevice,
    };
    use crate::scope::{Scope, SecretScope};
    use crate::type_conversions::ToC;
    use crate::VkVersion;

    use vk::has_command::DestroyDevice;

    use vk::context::{CommandLoadError, Context, LoadCommands};
    use vk::Version;

    pub trait DeviceConfig: Send + Sync {
        const VERSION: VkVersion;
        type Context: DestroyDevice + Send + Sync;
        type PhysicalDevice: PhysicalDevice;
        fn queue_config(&self) -> &[DeviceQueueCreateInfo<Self::PhysicalDevice>];
        fn queue_family_properties(&self) -> &[vk::QueueFamilyProperties];
    }

    pub struct Config<'a, C, P> {
        context: PhantomData<C>,
        physical_device: PhantomData<P>,
        queue_config_ref: &'a [DeviceQueueCreateInfo<'a, P>],
        queue_family_properties: &'a [vk::QueueFamilyProperties],
    }

    unsafe impl<'a, C: Send, P> Send for Config<'a, C, P> {}
    unsafe impl<'a, C: Sync, P> Sync for Config<'a, C, P> {}

    impl<'a, C, P> Config<'a, C, P> {
        pub(crate) fn new(
            queue_config_ref: &'a [DeviceQueueCreateInfo<'a, P>],
            queue_family_properties: &'a [vk::QueueFamilyProperties],
        ) -> Self {
            Self {
                context: PhantomData,
                physical_device: PhantomData,
                queue_config_ref,
                queue_family_properties,
            }
        }
    }

    impl<C, P: PhysicalDevice> DeviceConfig for Config<'_, C, P>
    where
        C: Context + Send + Sync,
        C::Commands: LoadCommands + DestroyDevice + Version + Send + Sync,
    {
        const VERSION: VkVersion = C::Commands::VERSION;
        type Context = C::Commands;
        type PhysicalDevice = P;

        fn queue_config(&self) -> &[DeviceQueueCreateInfo<P>] {
            self.queue_config_ref
        }

        fn queue_family_properties(&self) -> &[vk::QueueFamilyProperties] {
            self.queue_family_properties
        }
    }

    pub type ScopedDevice<S, C> = SecretScope<S, Device<C>>;

    impl<C: DeviceConfig> super::Device for Scope<'_, Device<C>> {
        type Config = C;
        type PhysicalDevice = C::PhysicalDevice;
        type Context = C::Context;
    }

    pub struct Device<C: DeviceConfig> {
        pub(crate) handle: vk::Device,
        pub(crate) context: C::Context,
        pub(crate) config: C,
    }

    unsafe impl<C: DeviceConfig> Send for Device<C> {}
    unsafe impl<C: DeviceConfig> Sync for Device<C> {}

    impl<C: DeviceConfig> std::fmt::Debug for Device<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Device")
                .field("handle", &self.handle)
                .field("version", &C::VERSION)
                .finish()
        }
    }

    impl<C: DeviceConfig> Device<C>
    where
        C::Context: LoadCommands,
    {
        pub(crate) fn load_commands(
            handle: vk::Device,
            config: C,
        ) -> Result<Self, CommandLoadError> {
            let loader = |command_name| unsafe { vk::GetDeviceProcAddr(handle, command_name) };
            Ok(Self {
                handle,
                context: C::Context::load(loader)?,
                config,
            })
        }
    }

    /*
    https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyDevice.html
    */
    impl<C: DeviceConfig> Drop for Device<C> {
        fn drop(&mut self) {
            unsafe { self.context.DestroyDevice().get_fptr()(self.handle, None.to_c()) }

            check_vuids::check_vuids!(DestroyDevice);

            #[allow(unused_labels)]
            'VUID_vkDestroyDevice_device_05137: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "All child objects created on device must have been destroyed prior to destroying device"
                }

                // all child objects borrow the device, and *normally* they are dropped/destroyed before the device is destroyed
                // However, it is well known that rust does not guarantee that values will be dropped. Thus, we cannot enforce this rule
                // In any event, if a child object is not dropped (e.g. forgotten), it should never be used again or dropped. Thus, even if the Device is
                // dropped, the child objects are merely leaked, and it is "assumed" that this is no real issue even in Vulkan.
            }

            #[allow(unused_labels)]
            'VUID_vkDestroyDevice_device_00379: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If VkAllocationCallbacks were provided when device was created, a compatible set of"
                "callbacks must be provided here"
                }

                // TODO: currently VkAllocationCallbacks are not supported
                // when added, this need to be checked again
            }

            #[allow(unused_labels)]
            'VUID_vkDestroyDevice_device_00380: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If no VkAllocationCallbacks were provided when device was created, pAllocator must"
                "be NULL"
                }

                // This is currently always set to NULL
                // TODO: ensure still ok when VkAllocationCallbacks are supported
            }

            #[allow(unused_labels)]
            'VUID_vkDestroyDevice_device_parameter: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If device is not NULL, device must be a valid VkDevice handle"
                }

                // Device will always be a valid handle. Guaranteed by Device creation
            }

            #[allow(unused_labels)]
            'VUID_vkDestroyDevice_pAllocator_parameter: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
                "structure"
                }

                // TODO: currently VkAllocationCallbacks are not supported
                // when added, this need to be checked again
            }
        }
    }
}
