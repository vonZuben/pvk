use super::command_impl_prelude::*;

use crate::error::Error;

use crate::structs::LayerProperties;

/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceLayerProperties.html>
pub fn enumerate_instance_layer_properties<S: ArrayStorage<LayerProperties<()>>>(
    mut storage: S,
) -> Result<S::InitStorage, Error> {
    let command = super::entry_fn_loader::<vk::EnumerateInstanceLayerProperties>()
        .unwrap()
        .get_fptr();
    enumerator_code2!(command; () -> storage)
}

// all verified by enumerator_code!()
const _VUIDS: () = {
    check_vuids::check_vuids!(EnumerateInstanceLayerProperties);

    #[allow(unused_labels)]
    'VUID_vkEnumerateInstanceLayerProperties_pPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateInstanceLayerProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
        "must be a valid pointer to an array of pPropertyCount VkLayerProperties structures"
        }

        // enumerator_code2!
    }
};
