use super::*;

use crate::array_storage::ArrayStorage;
use crate::error::Error;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceLayerProperties;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceLayerProperties.html
 */
impl<'scope, I: Instance> ScopedPhysicalDeviceType<'scope, I> {
    pub fn enumerate_device_layer_properties<P, S: ArrayStorage<LayerProperties<'scope>>>(
        &self,
        mut storage: S,
    ) -> Result<S::InitStorage, Error>
    where
        I::Commands: EnumerateDeviceLayerProperties<P>,
    {
        // all checks handled by PhysicalDevice creation and enumerator_code2!()
        check_vuid_defs2!(EnumerateDeviceLayerProperties
            pub const VUID_vkEnumerateDeviceLayerProperties_physicalDevice_parameter: &'static [u8] =
                "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
            pub const VUID_vkEnumerateDeviceLayerProperties_pPropertyCount_parameter: &'static [u8] =
                "pPropertyCount must be a valid pointer to a uint32_t value".as_bytes();
            pub const VUID_vkEnumerateDeviceLayerProperties_pProperties_parameter : & 'static [ u8 ] =
                "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkLayerProperties structures" . as_bytes ( ) ;
        );

        enumerator_code2!(self.instance.commands.EnumerateDeviceLayerProperties().get_fptr(); (self.handle) -> storage)
    }
}

simple_struct_wrapper_scoped!(LayerProperties);

impl LayerProperties<'_> {
    get_str!(layer_name);
    get_str!(description);
    pretty_version!(spec_version);
}

impl std::fmt::Debug for LayerProperties<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayerProperties")
            .field("Name", &self.layer_name())
            .field("Spec Version", &self.spec_version())
            .field("Implementation version", &self.inner.implementation_version)
            .field("Description", &self.description())
            .finish()
    }
}
