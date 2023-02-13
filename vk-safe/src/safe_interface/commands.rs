use std::ffi::CStr;
use vk_safe_sys as vk;

use super::Result;
use super::enumerator_storage::EnumeratorStorage;
use super::structs::*;

// Entry level interface
pub trait CreateInstance {
    fn create_instance<V: vk::VulkanVersion, E>(&self, create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>) -> Result<vk::Instance>;
}

pub trait EnumerateInstanceExtensionProperties {
    fn enumerate_instance_extension_properties<S: EnumeratorStorage<ExtensionProperties>>(&self, layer_name: Option<&CStr>, storage: S) -> Result<S::InitStorage>;
}

pub trait EnumerateInstanceLayerProperties {
    fn enumerate_instance_layer_properties<S: EnumeratorStorage<LayerProperties>>(&self, storage: S) -> Result<S::InitStorage>;
}

pub trait EnumerateInstanceVersion {
    fn enumerate_instance_version(&self) -> Result<crate::utils::VkVersion>;
}