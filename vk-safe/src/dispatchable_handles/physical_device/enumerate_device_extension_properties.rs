use super::concrete_type::ScopedPhysicalDeviceType;

use crate::dispatchable_handles::instance::Instance;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::vk_str::VkStr;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceExtensionProperties;

pub use crate::dispatchable_handles::common::extension_properties::ExtensionProperties;

impl<S, I: Instance> ScopedPhysicalDeviceType<S, I>
where
    I::Context: EnumerateDeviceExtensionProperties,
{
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceExtensionProperties.html>
    pub fn enumerate_device_extension_properties<A: ArrayStorage<ExtensionProperties<S>>>(
        &self,
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

        enumerator_code2!(self.instance.context.EnumerateDeviceExtensionProperties().get_fptr(); (self.handle, layer_name) -> storage)
    }
}
