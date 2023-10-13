use super::command_impl_prelude::*;

use std::fmt;

use crate::error::Error;

//===========LayerProperties
simple_struct_wrapper!(LayerProperties);

impl LayerProperties {
    get_str!(layer_name);
    get_str!(description);
    pretty_version!(spec_version);
}

impl fmt::Debug for LayerProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayerProperties")
            .field("Name", &self.layer_name())
            .field("Spec Version", &self.spec_version())
            .field("Implementation Version", &self.implementation_version)
            .field("Description", &self.description())
            .finish()
    }
}

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceLayerProperties.html
*/
pub fn enumerate_instance_layer_properties<S: ArrayStorage<LayerProperties>>(
    mut storage: S,
) -> Result<S::InitStorage, Error> {
    let command = super::entry_fn_loader::<vk::EnumerateInstanceLayerProperties>()
        .unwrap()
        .get_fptr();
    enumerator_code2!(command; () -> storage)
}

// all verified by enumerator_code!()
const _VUIDS: () = {
    check_vuid_defs2!(EnumerateInstanceLayerProperties
        pub const VUID_vkEnumerateInstanceLayerProperties_pPropertyCount_parameter:
            &'static [u8] = "pPropertyCount must be a valid pointer to a uint32_t value".as_bytes();
        pub const VUID_vkEnumerateInstanceLayerProperties_pProperties_parameter : & 'static [ u8 ] = "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties must be a valid pointer to an array of pPropertyCount VkLayerProperties structures" . as_bytes ( ) ;
    )
};
