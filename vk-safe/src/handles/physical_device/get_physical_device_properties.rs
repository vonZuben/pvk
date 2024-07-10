use super::PhysicalDevice;

use crate::structs::PhysicalDeviceProperties;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceProperties;

pub(crate) fn get_physical_device_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceProperties>,
>(
    physical_device: &P,
) -> PhysicalDeviceProperties<P> {
    let mut properties = MaybeUninit::uninit();

    check_vuids::check_vuids!(GetPhysicalDeviceProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pProperties must be a valid pointer to a VkPhysicalDeviceProperties structure"
        }

        // MaybeUninit
    }

    unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceProperties()
            .get_fptr()(physical_device.raw_handle(), properties.as_mut_ptr());
        PhysicalDeviceProperties::new(properties.assume_init())
    }
}
