use super::command_impl_prelude::*;

use std::ffi::CStr;

pub trait EnumerateInstanceExtensionProperties {
    fn enumerate_instance_extension_properties<S: EnumeratorStorage<structs::ExtensionProperties>>(
        &self,
        layer_name: Option<&CStr>,
        storage: S,
    ) -> Result<S::InitStorage, vk::Result>;
}

impl_safe_entry_interface! {
EnumerateInstanceExtensionProperties {
    enumerator_code!(enumerate_instance_extension_properties(layer_name: Option<&CStr>) -> structs::ExtensionProperties);
}}
