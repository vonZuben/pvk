use super::command_impl_prelude::*;

use std::ffi::CStr;

//===========ExtensionProperties
simple_struct_wrapper!(ExtensionProperties);

impl ExtensionProperties {
    get_str!(extension_name);
}

/*
SAFETY (https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceExtensionProperties.html)

VUID-vkEnumerateInstanceExtensionProperties-pLayerName-parameter
If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string

- Option<&CStr> TODO, does CStr ensure null-terminated UTF-8 string?

VUID-vkEnumerateInstanceExtensionProperties-pPropertyCount-parameter
pPropertyCount must be a valid pointer to a uint32_t value

- internally handled with a &mut u32

VUID-vkEnumerateInstanceExtensionProperties-pProperties-parameter
If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures

- internally handled using EnumeratorStorage, rust slices, and len of such slices
*/
impl_safe_entry_interface! {
EnumerateInstanceExtensionProperties {
    enumerator_code!(enumerate_instance_extension_properties(layer_name: Option<&CStr>) -> ExtensionProperties);
}}
