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
