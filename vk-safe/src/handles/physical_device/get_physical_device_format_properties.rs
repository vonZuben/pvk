use super::PhysicalDevice;

use std::mem::MaybeUninit;

use crate::structs::FormatProperties;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFormatProperties;

pub(crate) fn get_physical_device_format_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceFormatProperties<X>>,
    F: vk::enum_traits::Format,
    X,
>(
    physical_device: &P,
    _format: F,
) -> FormatProperties<P, F> {
    check_vuids::check_vuids!(GetPhysicalDeviceFormatProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFormatProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFormatProperties_format_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "format must be a valid VkFormat value"
        }

        // vk::Format
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFormatProperties_pFormatProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pFormatProperties must be a valid pointer to a VkFormatProperties structure"
        }

        // MaybeUninit
    }

    let mut properties = MaybeUninit::uninit();
    unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceFormatProperties()
            .get_fptr()(
            physical_device.raw_handle(),
            F::VALUE,
            properties.as_mut_ptr(),
        );
        FormatProperties::from_c(properties.assume_init())
    }
}
