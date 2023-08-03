use std::marker::PhantomData;
use crate::pretty_version::VkVersion;
use vk_safe_sys as vk;
use crate::safe_interface::type_conversions::ToC;

use crate::scope::{Scope, Scoped};

use vk::commands::{CommandLoadError, LoadCommands};
use vk::GetCommand;

use vk::validation::DestroyDevice::*;

pub trait DeviceConfig {
    const VERSION: VkVersion;
    type Commands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyDevice>;
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

    type Commands = V::DeviceCommands;
}

pub type ScopeDevice<'d, C, Pd> = Scope<'d, Device<C, Pd>>;

#[derive(Debug)]
pub struct Device<C: DeviceConfig, Pd: Scoped> {
    pub(crate) handle: vk::Device,
    pub(crate) commands: C::Commands,
    _pd: std::marker::PhantomData<Pd>,
}

impl<C: DeviceConfig, Pd: Scoped> Device<C, Pd> {
    pub(crate) fn load_commands(
        handle: vk::Device,
    ) -> Result<Self, CommandLoadError>
    {
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
impl<C: DeviceConfig, Pd: Scoped> Drop for Device<C, Pd> {
    fn drop(&mut self) {
        validate(Validation);
        unsafe { self.commands.get_command().get_fptr()(self.handle, None.to_c()) }
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkDestroyDevice_device_00378: () = {
        // all child objects borrow the device, so rust ensures they are Dropped (and dropping destroys)
        // **actually** it is possible to forget child objects so that they are not Destroyed
        // However, it is understood that this can at worst cause resource/memory leaks, and is not "unsound"
        // Therefore, I accept that this rule is broken until "unsound" behavior can be observed
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

pub mod allocate_memory;