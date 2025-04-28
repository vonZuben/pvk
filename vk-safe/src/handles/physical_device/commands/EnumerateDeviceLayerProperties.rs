use crate::enumerator::Enumerator;
use crate::handles::DispatchableHandle;
use crate::scope::Captures;
use crate::structs::LayerProperties;

use vk_safe_sys as vk;

pub trait EnumerateDeviceLayerProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    fn enumerate_device_layer_properties<X>(
        &self,
    ) -> impl Enumerator<LayerProperties<Self>> + Captures<&Self>
    where
        Self::Commands: vk::has_command::EnumerateDeviceLayerProperties<X>,
    {
        use vk::has_command::EnumerateDeviceLayerProperties;

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
            self.commands().EnumerateDeviceLayerProperties().get_fptr();
            (self.raw_handle())
        )
    }
}
