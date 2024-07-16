use super::PhysicalDevice;

use std::mem::MaybeUninit;

use crate::error::Error;
use crate::handles::device::_Device;
use crate::scope::{HasScope, Tag};
use crate::structs::DeviceCreateInfo;
use crate::type_conversions::SafeTransmute;

pub(crate) use private::VersionCheck;

use vk_safe_sys as vk;

use vk::context::{Context, InstanceDependencies, LoadCommands};
use vk::has_command::{CreateDevice, DestroyDevice, EnumerateDeviceExtensionProperties};
use vk::Version;

mod private {
    use vk_safe_sys::Version;

    pub trait VersionCheck<I> {
        const VALID: ();
    }

    impl<I: Version, D: Version> VersionCheck<I> for D {
        const VALID: () = {
            if D::VERSION.raw() > I::VERSION.raw() {
                panic!("version of Instance must be >= version of Device")
            }
        };
    }
}

pub(crate) fn create_device<
    't,
    P: PhysicalDevice<Commands: CreateDevice + EnumerateDeviceExtensionProperties>,
    C,
    O,
    Z: HasScope<P>,
>(
    physical_device: &P,
    create_info: &DeviceCreateInfo<C, Z>,
    tag: Tag<'t>,
) -> Result<
    // impl Device<Context = C::Commands, PhysicalDevice = S, QueueConfig = Z> + Captures<Tag<'t>>,
    _Device<C::Commands, Tag<'t>>,
    Error,
>
where
    C: Context + InstanceDependencies<P::Commands, O> + Send + Sync,
    C::Commands: DestroyDevice + LoadCommands + Version + VersionCheck<P::Commands> + Send + Sync,
{
    // check version requirement
    let _ = C::Commands::VALID;

    check_vuids::check_vuids!(CreateDevice);

    #[allow(unused_labels)]
    'VUID_vkCreateDevice_ppEnabledExtensionNames_01387: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "All required device extensions for each extension in the VkDeviceCreateInfo::ppEnabledExtensionNames"
        "list must also be present in that list"
        }

        // This is ensured by the context creation macros and InstanceDependencies trait
    }

    #[allow(unused_labels)]
    'VUID_vkCreateDevice_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // ensured by PhysicalDevice creation
    }

    #[allow(unused_labels)]
    'VUID_vkCreateDevice_pCreateInfo_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pCreateInfo must be a valid pointer to a valid VkDeviceCreateInfo structure"
        }

        // ensured by DeviceCreateInfo creation
    }

    #[allow(unused_labels)]
    'VUID_vkCreateDevice_pAllocator_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
        "structure"
        }

        // AllocationCallbacks not supported
    }

    #[allow(unused_labels)]
    'VUID_vkCreateDevice_pDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pDevice must be a valid pointer to a VkDevice handle"
        }

        // MaybeUninit
    }

    let mut handle = MaybeUninit::uninit();

    // *********************************************
    // *********Fix with extension support**********
    // **VUID_VkDeviceCreateInfo_pProperties_04451**
    // *********************************************
    for e in physical_device.enumerate_device_extension_properties(None, Vec::new())? {
        if e.extension_name() == "VK_KHR_portability_subset" {
            panic!("Physical device with VK_KHR_portability_subset is not supported")
        }
    }
    // *********************************************

    let device;
    unsafe {
        let res = physical_device.commands().CreateDevice().get_fptr()(
            physical_device.raw_handle(),
            create_info.safe_transmute(),
            std::ptr::null(),
            handle.as_mut_ptr(),
        );
        check_raw_err!(res);
        device = handle.assume_init();
    }
    let loader = |command_name| unsafe { vk::GetDeviceProcAddr(device, command_name) };
    Ok(_Device::new(device, C::Commands::load(loader)?, tag))
}
