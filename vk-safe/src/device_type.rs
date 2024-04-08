use crate::type_conversions::ToC;
use crate::VkVersion;
use std::marker::PhantomData;
use vk_safe_sys as vk;

use crate::physical_device::{create_device::DeviceQueueCreateInfo, PhysicalDevice};
use crate::scope::{RefScope, Scope};

use vk::has_command::DestroyDevice;

use vk::context::{CommandLoadError, Commands, LoadCommands};
use vk::Version;

pub trait DeviceConfig {
    const VERSION: VkVersion;
    type Commands: DestroyDevice;
    type PhysicalDevice: PhysicalDevice;
    fn queue_config(&self) -> &[DeviceQueueCreateInfo<Self::PhysicalDevice>];
    fn queue_family_properties(&self) -> &[vk::QueueFamilyProperties];
}

pub struct Config<'a, C, P> {
    commands: PhantomData<C>,
    physical_device: PhantomData<P>,
    queue_config_ref: &'a [DeviceQueueCreateInfo<'a, P>],
    queue_family_properties: &'a [vk::QueueFamilyProperties],
}

impl<'a, C, P> Config<'a, C, P> {
    pub(crate) fn new(
        queue_config_ref: &'a [DeviceQueueCreateInfo<'a, P>],
        queue_family_properties: &'a [vk::QueueFamilyProperties],
    ) -> Self {
        Self {
            commands: PhantomData,
            physical_device: PhantomData,
            queue_config_ref,
            queue_family_properties,
        }
    }
}

impl<C, P: PhysicalDevice> DeviceConfig for Config<'_, C, P>
where
    C: Commands,
    C::Commands: LoadCommands + DestroyDevice + Version,
{
    const VERSION: VkVersion = C::Commands::VERSION;
    type Commands = C::Commands;
    type PhysicalDevice = P;

    fn queue_config(&self) -> &[DeviceQueueCreateInfo<P>] {
        self.queue_config_ref
    }

    fn queue_family_properties(&self) -> &[vk::QueueFamilyProperties] {
        self.queue_family_properties
    }
}

pub type ScopedDeviceType<S, C> = RefScope<S, DeviceType<C>>;

pub trait Device: std::ops::Deref<Target = ScopedDeviceType<Self, Self::Config>> + Copy {
    type Config: DeviceConfig<Commands = Self::Commands, PhysicalDevice = Self::PhysicalDevice>;
    type PhysicalDevice;
    type Commands;
}

impl<'scope, C: DeviceConfig> Device for Scope<'scope, DeviceType<C>> {
    type Config = C;
    type PhysicalDevice = C::PhysicalDevice;
    type Commands = C::Commands;
}

pub struct DeviceType<C: DeviceConfig> {
    pub(crate) handle: vk::Device,
    pub(crate) commands: C::Commands,
    config: C,
}

impl<C: DeviceConfig> std::fmt::Debug for DeviceType<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle)
            .field("version", &C::VERSION)
            .finish()
    }
}

impl<C: DeviceConfig> DeviceType<C>
where
    C::Commands: LoadCommands,
{
    pub(crate) fn load_commands(handle: vk::Device, config: C) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetDeviceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            commands: C::Commands::load(loader)?,
            config,
        })
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyDevice.html
*/
impl<C: DeviceConfig> Drop for DeviceType<C> {
    fn drop(&mut self) {
        unsafe { self.commands.DestroyDevice().get_fptr()(self.handle, None.to_c()) }

        check_vuids::check_vuids!(DestroyDevice);

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_05137: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
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
            check_vuids::cur_description! {
            "If VkAllocationCallbacks were provided when device was created, a compatible set of"
            "callbacks must be provided here"
            }

            // TODO: currently VkAllocationCallbacks are not supported
            // when added, this need to be checked again
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_00380: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If no VkAllocationCallbacks were provided when device was created, pAllocator must"
            "be NULL"
            }

            // This is currently always set to NULL
            // TODO: ensure still ok when VkAllocationCallbacks are supported
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If device is not NULL, device must be a valid VkDevice handle"
            }

            // Device will always be a valid handle. Guaranteed by Device creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO: currently VkAllocationCallbacks are not supported
            // when added, this need to be checked again
        }
    }
}

pub mod allocate_memory;
pub mod get_device_queue;
pub mod map_memory;

pub mod device_exports {
    use super::*;
    pub use allocate_memory::MemoryAllocateInfo;

    pub use super::Device;
    pub use allocate_memory::DeviceMemory;
}
