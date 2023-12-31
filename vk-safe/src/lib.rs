#[macro_use]
mod error;

mod pretty_version;

#[macro_use]
mod helper_macros;

mod array_storage;
mod type_conversions;
mod vk_str;

mod device_type;
mod entry; // not finalized on if this should be pub
mod instance_type;
mod physical_device;
mod scope;

pub use pretty_version::VkVersion;
pub use vk_str::VkStr;

pub use vk_safe_sys::generated_vulkan::bitmasks::*;
pub use vk_safe_sys::generated_vulkan::enum_variants::*;
pub use vk_safe_sys::generated_vulkan::enumerations::*;
pub use vk_safe_sys::{device_context, instance_context};

pub use device_type::device_exports::*;
pub use entry::*;
pub use physical_device::physical_device_exports::*;

pub use scope::scope;

pub mod instance {
    pub use vk_safe_sys::extension::instance::*;
    pub use vk_safe_sys::version::instance::*;
}

pub mod device {
    pub use vk_safe_sys::extension::device::*;
    pub use vk_safe_sys::version::device::*;
}
