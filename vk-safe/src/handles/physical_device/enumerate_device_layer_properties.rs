use super::PhysicalDevice;

use crate::enumerator::Enumerator;
use crate::scope::Captures;
use crate::structs::LayerProperties;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceLayerProperties;

pub(crate) fn enumerate_device_layer_properties<
    P: PhysicalDevice<Commands: EnumerateDeviceLayerProperties>,
>(
    physical_device: &P,
) -> impl Enumerator<LayerProperties<P>> + Captures<&P> {
    check_vuids::check_vuids!(EnumerateDeviceLayerProperties);

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceLayerProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // always valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceLayerProperties_pPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateDeviceLayerProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
        "must be a valid pointer to an array of pPropertyCount VkLayerProperties structures"
        }

        // enumerator_code2!
    }

    make_enumerator!(
        physical_device.commands().EnumerateDeviceLayerProperties().get_fptr();
        (physical_device.raw_handle())
    )
}
