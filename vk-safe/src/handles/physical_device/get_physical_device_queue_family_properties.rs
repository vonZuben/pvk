use super::PhysicalDevice;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::structs::QueueFamilies;
use crate::type_conversions::SafeTransmute;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceQueueFamilyProperties;

pub(crate) fn get_physical_device_queue_family_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceQueueFamilyProperties>,
    A: ArrayStorage<vk::QueueFamilyProperties>,
>(
    physical_device: &P,
    mut storage: A,
) -> Result<QueueFamilies<P, A>, Error> {
    check_vuids::check_vuids!(GetPhysicalDeviceQueueFamilyProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pQueueFamilyPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pQueueFamilyPropertyCount is not 0, and pQueueFamilyProperties"
        "is not NULL, pQueueFamilyProperties must be a valid pointer to an array of pQueueFamilyPropertyCount"
        "VkQueueFamilyProperties structures"
        }

        // enumerator_code2!
    }

    let families = enumerator_code2!(
        physical_device.commands().GetPhysicalDeviceQueueFamilyProperties().get_fptr();
        (physical_device.raw_handle()) -> storage)?;

    Ok(QueueFamilies::new(families))
}
