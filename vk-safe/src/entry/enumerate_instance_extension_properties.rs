use super::command_impl_prelude::*;

use std::fmt;

use crate::vk_str::VkStr;

use crate::error::Error;

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
pub fn enumerate_instance_extension_properties<S: ArrayStorage<ExtensionProperties>>(layer_name: Option<VkStr>, mut storage: S) -> Result<S::InitStorage, Error> {
    let command = super::entry_fn_loader::<vk::EnumerateInstanceExtensionProperties>().unwrap().get_fptr();
    enumerator_code2!(command; (layer_name) -> storage)
}

const _VUIDS: () = {
    check_vuid_defs2!(EnumerateInstanceExtensionProperties
        pub const VUID_vkEnumerateInstanceExtensionProperties_pLayerName_parameter:
            &'static [u8] =
            "If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string"
                .as_bytes();
        CHECK {
            // verified by Option<VkStr>
        }
        pub const VUID_vkEnumerateInstanceExtensionProperties_pPropertyCount_parameter:
            &'static [u8] = "pPropertyCount must be a valid pointer to a uint32_t value".as_bytes();
        CHECK {
            // handled by enumerator_code!()
        }
        pub const VUID_vkEnumerateInstanceExtensionProperties_pProperties_parameter : & 'static [ u8 ] = "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures" . as_bytes ( ) ;
        CHECK {
            // handled by enumerator_code!()
        }
    )
};