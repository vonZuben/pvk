use crate::pretty_version::VkVersion;
use crate::type_conversions::ToC;
use std::marker::PhantomData;
use vk_safe_sys as vk;

use crate::physical_device::PhysicalDevice;
use crate::scope::{Scope, Scoped};

use vk::has_command::DestroyDevice;

use vk::commands::{CommandLoadError, LoadCommands, Version};

pub trait DeviceConfig {
    const VERSION: VkVersion;
    type DropProvider;
    type Commands: LoadCommands + DestroyDevice<Self::DropProvider>;
}

pub struct Config<P, Cmd> {
    drop_provider: PhantomData<P>,
    commands: PhantomData<Cmd>,
}

impl<P, Cmd> Clone for Config<P, Cmd> {
    fn clone(&self) -> Self {
        Self {
            drop_provider: PhantomData,
            commands: PhantomData,
        }
    }
}

impl<P, Cmd> Copy for Config<P, Cmd> {}

impl<P, Cmd> DeviceConfig for Config<P, Cmd>
where
    Cmd: LoadCommands + DestroyDevice<P> + Version,
{
    const VERSION: VkVersion = VkVersion::new(
        Cmd::VERSION_TRIPLE.0,
        Cmd::VERSION_TRIPLE.1,
        Cmd::VERSION_TRIPLE.2,
    );
    type DropProvider = P;
    type Commands = Cmd;
}

pub type ScopedDeviceType<'d, C, Pd> = Scope<'d, DeviceType<C, Pd>>;

pub trait Device:
    Scoped + std::ops::Deref<Target = DeviceType<Self::Config, Self::PhysicalDevice>> + Copy
{
    type Config: DeviceConfig<Commands = Self::Commands>;
    type PhysicalDevice: PhysicalDevice;
    type Commands;
}

impl<'scope, C: DeviceConfig, Pd: PhysicalDevice> Device for ScopedDeviceType<'scope, C, Pd> {
    type Config = C;
    type PhysicalDevice = Pd;
    type Commands = C::Commands;
}

pub struct DeviceType<C: DeviceConfig, Pd: PhysicalDevice> {
    pub(crate) handle: vk::Device,
    pub(crate) commands: C::Commands,
    _pd: std::marker::PhantomData<Pd>,
}

impl<C: DeviceConfig, Pd: PhysicalDevice> std::fmt::Debug for DeviceType<C, Pd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle)
            .field("version", &C::VERSION)
            .finish()
    }
}

impl<C: DeviceConfig, Pd: PhysicalDevice> DeviceType<C, Pd> {
    pub(crate) fn load_commands(handle: vk::Device) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetDeviceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            commands: C::Commands::load(loader)?,
            _pd: PhantomData,
        })
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyDevice.html
*/
impl<C: DeviceConfig, Pd: PhysicalDevice> Drop for DeviceType<C, Pd> {
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
