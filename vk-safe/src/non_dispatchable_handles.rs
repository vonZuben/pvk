//! Vulkan non dispatchable handles
//!
//! 🚧 docs in progress

pub mod device_memory;

pub(crate) mod exports {
    use super::*;

    pub use device_memory::{DeviceMemory, MappedMemory};
}
