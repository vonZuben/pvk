#[macro_use]
mod error;

mod pretty_version;

#[macro_use]
mod helper_macros;

mod array_storage;
mod vk_str;

pub mod device;
pub mod entry; // not finalized on if this should be pub
pub mod instance;
pub mod physical_device;
pub mod safe_interface;
pub mod scope;

pub use pretty_version::VkVersion;
pub use vk_str::VkStr;

pub use vk_safe_sys::{device_context, instance_context};

pub use entry::*;
