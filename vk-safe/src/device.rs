use std::marker::PhantomData;
use crate::pretty_version::VkVersion;
use vk_safe_sys as vk;
use crate::safe_interface::type_conversions::ToC;

use vk::commands::{CommandLoadError, LoadCommands};
use vk::GetCommand;

use vk::validation::DestroyDevice::*;

pub trait DeviceConfig {
    const VERSION: VkVersion;
    type DeviceCommands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyDevice>;
    type DeviceExtensions : vk::commands::LoadCommands;
}

#[derive(Debug)]
pub struct Config<V, E> {
    _version: PhantomData<V>,
    _extension: PhantomData<E>,
}

impl<V, E> Config<V, E> {
    pub fn new(_version: V, _extensions: E) -> Self {
        Self { _version: PhantomData, _extension: PhantomData }
    }
}

impl<V: vk::VulkanVersion, E: vk::VulkanExtension> DeviceConfig for Config<V, E>
where
    V::DeviceCommands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyDevice>,
    E::DeviceCommands : vk::commands::LoadCommands,
{
    const VERSION: VkVersion = VkVersion::new(V::VersionTriple.0, V::VersionTriple.1, V::VersionTriple.2);

    type DeviceCommands = V::DeviceCommands;

    type DeviceExtensions = E::DeviceCommands;
}

#[derive(Debug)]
pub struct Device<'instance, I, C: DeviceConfig> {
    handle: vk::Device,
    pub(crate) feature_commands: C::DeviceCommands,
    pub(crate) extension_commands: C::DeviceExtensions,
    _instance: std::marker::PhantomData<&'instance I>,
}

impl<'instance, I, C: DeviceConfig> Device<'instance, I, C> {
    pub(crate) fn load_commands(
        handle: vk::Device,
    ) -> Result<Self, CommandLoadError>
    {
        let loader = |command_name| unsafe { vk::GetDeviceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            feature_commands: C::DeviceCommands::load(loader)?,
            extension_commands: C::DeviceExtensions::load(loader)?,
            _instance: PhantomData,
        })
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyDevice.html
*/
impl<'instance, I, C: DeviceConfig> Drop for Device<'instance, I, C> {
    fn drop(&mut self) {
        validate(Validation);
        unsafe { self.feature_commands.get().get_fptr()(self.handle, None.to_c()) }
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkDestroyDevice_device_00378: () = {
        // all child objects borrow the device, so rust ensures they are Dropped (and dropping destroys)
    };

    const VUID_vkDestroyDevice_device_00379: () = {
        // when supported, the Device handle will store the allocation callbacks used and automatically use them
    };

    const VUID_vkDestroyDevice_device_00380: () = {
        // ensured along with the VUID_vkDestroyDevice_device_00379
    };

    const VUID_vkDestroyDevice_device_parameter: () = {
        // the Device can only be created with a valid handle in create_device()
    };

    const VUID_vkDestroyDevice_pAllocator_parameter: () = {
        // ensured along with the VUID_vkDestroyDevice_device_00379
    };
}

check_vuid_defs!(
    pub const VUID_vkDestroyDevice_device_00378 : & 'static [ u8 ] = "All child objects created on device must have been destroyed prior to destroying device" . as_bytes ( ) ;
        pub const VUID_vkDestroyDevice_device_00379 : & 'static [ u8 ] = "If VkAllocationCallbacks were provided when device was created, a compatible set of callbacks must be provided here" . as_bytes ( ) ;
        pub const VUID_vkDestroyDevice_device_00380 : & 'static [ u8 ] = "If no VkAllocationCallbacks were provided when device was created, pAllocator must be NULL" . as_bytes ( ) ;
        pub const VUID_vkDestroyDevice_device_parameter: &'static [u8] =
            "If device is not NULL, device must be a valid VkDevice handle".as_bytes();
        pub const VUID_vkDestroyDevice_pAllocator_parameter : & 'static [ u8 ] = "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure" . as_bytes ( ) ;
);