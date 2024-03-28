#[macro_use]
mod error;

#[macro_use]
mod helper_macros;
#[macro_use]
mod error_macros;

mod array_storage;
mod flags;
mod type_conversions;
mod vk_str;

mod device_type;
mod entry; // not finalized on if this should be pub
mod instance_type;
mod physical_device;
mod queue_type;
mod scope;

pub use vk_safe_sys::VkVersion;
pub use vk_str::VkStr;

pub use vk_safe_sys::generated_vulkan::bitmasks::*;
pub use vk_safe_sys::generated_vulkan::enum_variants::*;
pub use vk_safe_sys::generated_vulkan::enumerations::*;
pub use vk_safe_sys::{device_context, instance_context, queue_capabilities};

pub use device_type::device_exports::*;
pub use entry::*;
pub use instance_type::instance_exports::*;
pub use physical_device::physical_device_exports::*;

pub use scope::scope;

pub use flags::*;

pub mod instance {
    pub use vk_safe_sys::extension::instance::traits::*;
    pub use vk_safe_sys::version::instance::traits::*;
}

pub mod device {
    pub use vk_safe_sys::extension::device::traits::*;
    pub use vk_safe_sys::version::device::traits::*;
}
