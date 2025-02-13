use super::PhysicalDevice;

use std::mem::MaybeUninit;

use crate::structs::PhysicalDeviceMemoryProperties;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceMemoryProperties;

pub(crate) fn get_physical_device_memory_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceMemoryProperties<X>>,
    X,
>(
    physical_device: &P,
) -> PhysicalDeviceMemoryProperties<P> {
    check_vuids::check_vuids!(GetPhysicalDeviceMemoryProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceMemoryProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pMemoryProperties must be a valid pointer to a VkPhysicalDeviceMemoryProperties structure"
        }

        // MaybeUninit
    }

    let mut properties = MaybeUninit::uninit();
    unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceMemoryProperties()
            .get_fptr()(physical_device.raw_handle(), properties.as_mut_ptr());
        PhysicalDeviceMemoryProperties::from_c(properties.assume_init())
    }
}
