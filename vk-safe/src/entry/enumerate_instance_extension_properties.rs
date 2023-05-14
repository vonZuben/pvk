use super::command_impl_prelude::*;

use std::fmt;

use crate::vk_str::VkStr;

//===========ExtensionProperties
simple_struct_wrapper!(ExtensionProperties);

impl ExtensionProperties {
    get_str!(extension_name);
}

impl fmt::Debug for ExtensionProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionProperties")
            .field("Name", &self.extension_name())
            .field("Spec Version", &self.spec_version)
            .finish()
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceExtensionProperties.html
*/
impl_safe_entry_interface! {
EnumerateInstanceExtensionProperties {
    enumerator_code!(enumerate_instance_extension_properties(layer_name: Option<VkStr<'_>>) -> ExtensionProperties);
}}

mod enumerate_instance_extension_properties_validation {
    use vk_safe_sys::validation::EnumerateInstanceExtensionProperties::*;

    pub struct Validation;

    #[allow(non_upper_case_globals)]
    impl Vuids for Validation {
        const VUID_vkEnumerateInstanceExtensionProperties_pLayerName_parameter: () = {
            // verified by Option<VkStr>
        };

        const VUID_vkEnumerateInstanceExtensionProperties_pPropertyCount_parameter: () ={
            // handled by enumerator_code!()
        };

        const VUID_vkEnumerateInstanceExtensionProperties_pProperties_parameter: () = {
            // handled by enumerator_code!()
        };
    }

    check_vuid_defs!(
        pub const VUID_vkEnumerateInstanceExtensionProperties_pLayerName_parameter:
            &'static [u8] =
            "If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string"
                .as_bytes();
        pub const VUID_vkEnumerateInstanceExtensionProperties_pPropertyCount_parameter:
            &'static [u8] = "pPropertyCount must be a valid pointer to a uint32_t value".as_bytes();
        pub const VUID_vkEnumerateInstanceExtensionProperties_pProperties_parameter : & 'static [ u8 ] = "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures" . as_bytes ( ) ;
    );
}