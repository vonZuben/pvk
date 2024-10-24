use super::command_impl_prelude::*;

use crate::enumerator::Enumerator;
use crate::scope::Captures;
use crate::vk_str::VkStr;

use crate::structs::ExtensionProperties;

/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceExtensionProperties.html>
pub fn enumerate_instance_extension_properties(
    layer_name: Option<VkStr>,
) -> impl Enumerator<ExtensionProperties<()>> + Captures<VkStr> {
    let command = super::entry_fn_loader::<vk::EnumerateInstanceExtensionProperties>()
        .unwrap()
        .get_fptr();
    make_enumerator!(command; (layer_name))
}

const _VUIDS: () = {
    check_vuids::check_vuids!(EnumerateInstanceExtensionProperties);

    #[allow(unused_labels)]
    'VUID_vkEnumerateInstanceExtensionProperties_pLayerName_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If pLayerName is not NULL, pLayerName must be a null-terminated UTF-8 string"
        }

        // Option<VkStr>
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateInstanceExtensionProperties_pPropertyCount_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pPropertyCount must be a valid pointer to a uint32_t value"
        }

        // enumerator_code2!
    }

    #[allow(unused_labels)]
    'VUID_vkEnumerateInstanceExtensionProperties_pProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "If the value referenced by pPropertyCount is not 0, and pProperties is not NULL, pProperties"
        "must be a valid pointer to an array of pPropertyCount VkExtensionProperties structures"
        }

        // enumerator_code2!
    }
};
