use super::command_impl_prelude::*;

use std::fmt;

//===========LayerProperties
simple_struct_wrapper!(LayerProperties);

impl LayerProperties {
    get_str!(layer_name);
    get_str!(description);
}

impl fmt::Debug for LayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spec_version = unsafe { crate::utils::VkVersion::from_raw(self.spec_version) };
        f.debug_struct("LayerProperties")
            .field("Name", &self.layer_name())
            .field("Spec Version", &spec_version)
            .field("Implementation Version", &self.implementation_version)
            .field("Description", &self.description())
            .finish()
    }
}

/*
SAFETY (https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceLayerProperties.html)

VUID-vkEnumerateInstanceLayerProperties-pPropertyCount-parameter
pPropertyCount must be a valid pointer to a uint32_t value

- internally handled with a &mut u32

VUID-vkEnumerateInstanceLayerProperties-pProperties-parameter
If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkLayerProperties structures

- internally handled using EnumeratorStorage, rust slices, and len of such slices
*/
impl_safe_entry_interface! {
EnumerateInstanceLayerProperties {
    enumerator_code!(enumerate_instance_layer_properties() -> LayerProperties);
}}
