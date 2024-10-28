use super::PhysicalDevice;

use crate::structs::PhysicalDeviceFeatures;
use crate::type_conversions::ConvertWrapper;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFeatures;

pub(crate) fn get_physical_device_features<
    P: PhysicalDevice<Commands: GetPhysicalDeviceFeatures>,
>(
    physical_device: &P,
) -> PhysicalDeviceFeatures<P> {
    check_vuids::check_vuids!(GetPhysicalDeviceFeatures);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFeatures_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFeatures_pFeatures_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pFeatures must be a valid pointer to a VkPhysicalDeviceFeatures structure"
        }

        // MaybeUninit
    }

    let mut features = MaybeUninit::uninit();
    unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceFeatures()
            .get_fptr()(physical_device.raw_handle(), features.as_mut_ptr());
        PhysicalDeviceFeatures::from_c(features.assume_init())
    }
}
