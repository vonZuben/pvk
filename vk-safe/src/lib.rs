#[macro_use]
mod error;

mod pretty_version;

#[macro_use]
mod helper_macros;

mod array_storage;
mod vk_str;

pub mod safe_interface;
pub mod scope;
pub mod entry; // not finalized on if this should be pub
pub mod instance;
pub mod physical_device;
pub mod device;

pub use pretty_version::VkVersion;
pub use vk_str::VkStr;

pub use vk_safe_sys::{instance_context, device_context};

pub use entry::*;