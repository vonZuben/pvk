use super::concrete_type::ScopedPhysicalDevice;

use crate::dispatchable_handles::instance::Instance;

use crate::array_storage::ArrayStorage;
use crate::error::Error;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceLayerProperties;

pub use crate::dispatchable_handles::common::layer_properties::LayerProperties;

impl<S, I: Instance> ScopedPhysicalDevice<S, I>
where
    I::Context: EnumerateDeviceLayerProperties,
{
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceLayerProperties.html>
    pub fn enumerate_device_layer_properties<A: ArrayStorage<LayerProperties<S>>>(
        &self,
        mut storage: A,
    ) -> Result<A::InitStorage, Error> {
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

        enumerator_code2!(self.instance.context.EnumerateDeviceLayerProperties().get_fptr(); (self.handle) -> storage)
    }
}
