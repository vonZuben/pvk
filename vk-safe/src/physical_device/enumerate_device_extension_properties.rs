use super::*;

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::vk_str::VkStr;

use vk_safe_sys as vk;

use vk::has_command::EnumerateDeviceExtensionProperties;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceExtensionProperties.html
 */
impl<'scope, I: Instance> ScopedPhysicalDeviceType<'scope, I> {
    pub fn enumerate_device_extension_properties<P, S: ArrayStorage<ExtensionProperties<'scope>>>(
        &self,
        layer_name: Option<VkStr>,
        mut storage: S,
    ) -> Result<S::InitStorage, Error>
    where
        I::Commands: EnumerateDeviceExtensionProperties<P>,
    {
        // all checks handled by PhysicalDevice creation and enumerator_code2!(), except one below
        check_vuid_defs2!(EnumerateDeviceExtensionProperties
            pub const VUID_vkEnumerateDeviceExtensionProperties_physicalDevice_parameter:
                &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
            pub const VUID_vkEnumerateDeviceExtensionProperties_pLayerName_parameter: &'static [u8] =
                "If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string"
                    .as_bytes();
            CHECK {
                // this is handled by Option<VkStr>
            }
            pub const VUID_vkEnumerateDeviceExtensionProperties_pPropertyCount_parameter:
                &'static [u8] = "pPropertyCount must be a valid pointer to a uint32_t value".as_bytes();
            pub const VUID_vkEnumerateDeviceExtensionProperties_pProperties_parameter : & 'static [ u8 ] =
                "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures" . as_bytes ( ) ;
        );

        enumerator_code2!(self.instance.commands.EnumerateDeviceExtensionProperties().get_fptr(); (self.handle, layer_name) -> storage)
    }
}

simple_struct_wrapper_scoped!(ExtensionProperties);

impl ExtensionProperties<'_> {
    get_str!(extension_name);
}

impl std::fmt::Debug for ExtensionProperties<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionProperties")
            .field("Name", &self.extension_name())
            .field("Version", &self.inner.spec_version)
            .finish()
    }
}
