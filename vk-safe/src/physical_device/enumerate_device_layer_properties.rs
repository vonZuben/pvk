use super::*;

use crate::array_storage::ArrayStorage;
use crate::error::Error;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceLayerProperties;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceLayerProperties.html
 */
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn enumerate_device_layer_properties<A: ArrayStorage<LayerProperties<S>>>(
        &self,
        mut storage: A,
    ) -> Result<A::InitStorage, Error>
    where
        I::Commands: EnumerateDeviceLayerProperties,
    {
        check_vuids::check_vuids!(EnumerateDeviceLayerProperties);

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceLayerProperties_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // always valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceLayerProperties_pPropertyCount_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "pPropertyCount must be a valid pointer to a uint32_t value"
            }

            // enumerator_code2!
        }

        #[allow(unused_labels)]
        'VUID_vkEnumerateDeviceLayerProperties_pProperties_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::cur_description! {
            "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
            "must be a valid pointer to an array of pPropertyCount VkLayerProperties structures"
            }

            // enumerator_code2!
        }

        enumerator_code2!(self.instance.commands.EnumerateDeviceLayerProperties().get_fptr(); (self.handle) -> storage)
    }
}

simple_struct_wrapper_scoped!(LayerProperties);

impl<S> LayerProperties<S> {
    get_str!(layer_name);
    get_str!(description);
    pretty_version!(spec_version);
}

impl<S> std::fmt::Debug for LayerProperties<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayerProperties")
            .field("Name", &self.layer_name())
            .field("Spec Version", &self.spec_version())
            .field("Implementation version", &self.inner.implementation_version)
            .field("Description", &self.description())
            .finish()
    }
}
