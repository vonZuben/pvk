use super::PhysicalDevice;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::structs::ExtensionProperties;
use crate::VkStr;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceExtensionProperties;

pub(crate) fn enumerate_device_extension_properties<
    P: PhysicalDevice<Commands: EnumerateDeviceExtensionProperties>,
    A: ArrayStorage<ExtensionProperties<P>>,
>(
    physical_device: &P,
    layer_name: Option<VkStr>,
    mut storage: A,
) -> Result<A::InitStorage, Error> {
    check_vuids::check_vuids!(EnumerateDeviceExtensionProperties);

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceExtensionProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // always valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceExtensionProperties_pLayerName_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string"
        }

        // Option<VkStr>
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceExtensionProperties_pPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceExtensionProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
        "must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures"
        }

        // enumerator_code2!
    }

    enumerator_code2!(physical_device.commands().EnumerateDeviceExtensionProperties().get_fptr();
        (physical_device.raw_handle(), layer_name) -> storage)
}
