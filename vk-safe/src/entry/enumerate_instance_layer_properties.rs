use super::command_impl_prelude::*;

pub trait EnumerateInstanceLayerProperties {
    fn enumerate_instance_layer_properties<S: EnumeratorStorage<LayerProperties>>(
        &self,
        storage: S,
    ) -> Result<S::InitStorage, vk::Result>;
}

//===========LayerProperties
simple_struct_wrapper!(LayerProperties);

impl LayerProperties {
    get_str!(layer_name);
    get_str!(description);
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
