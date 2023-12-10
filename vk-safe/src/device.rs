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

        // check_vuid_defs2!( DestroyDevice
        //     pub const VUID_vkDestroyDevice_device_00378 : & 'static [ u8 ] = "All child objects created on device must have been destroyed prior to destroying device" . as_bytes ( ) ;
        //     CHECK {
        //         // all child objects borrow the device, so rust ensures they are Dropped (and dropping destroys)
        //         // **actually** it is possible to forget child objects so that they are not Destroyed
        //         // However, it is understood that this can at worst cause resource/memory leaks, and is not "unsound"
        //         // Therefore, I accept that this rule is broken until "unsound" behavior can be observed
        //     }
        //     pub const VUID_vkDestroyDevice_device_00379 : & 'static [ u8 ] = "If VkAllocationCallbacks were provided when device was created, a compatible set of callbacks must be provided here" . as_bytes ( ) ;
        //     CHECK {
        //         // when supported, the Device handle will store the allocation callbacks used and automatically use them
        //     }
        //     pub const VUID_vkDestroyDevice_device_00380 : & 'static [ u8 ] = "If no VkAllocationCallbacks were provided when device was created, pAllocator must be NULL" . as_bytes ( ) ;
        //     CHECK {
        //         // ensured along with the VUID_vkDestroyDevice_device_00379
        //     }
        //     pub const VUID_vkDestroyDevice_device_parameter: &'static [u8] =
        //         "If device is not NULL, device must be a valid VkDevice handle".as_bytes();
        //     CHECK {
        //         // the Device can only be created with a valid handle in create_device()
        //     }
        //     pub const VUID_vkDestroyDevice_pAllocator_parameter : & 'static [ u8 ] = "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure" . as_bytes ( ) ;
        //     CHECK {
        //         // ensured along with the VUID_vkDestroyDevice_device_00379
        //     }
        // );
    }
}

pub mod allocate_memory;
