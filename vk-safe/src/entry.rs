use vk_safe_sys as vk;

use vk::{
    commands::{CommandLoadError, LoadCommands},
    VulkanVersion,
};

/// Entry
///
/// provides a means for accessing global vulkan commands
#[derive(Debug)]
pub struct Entry<V: VulkanVersion> {
    commands: V::EntryCommands,
}

impl<V: VulkanVersion> Entry<V> {
    pub fn from_version(_v: V) -> std::result::Result<Self, CommandLoadError>
    where
        V::EntryCommands: LoadCommands,
    {
        let loader = |command_name| unsafe {
            vk::GetInstanceProcAddr(
                vk::Instance {
                    handle: std::ptr::null(),
                },
                command_name,
            )
        };

        Ok(Self {
            commands: V::EntryCommands::load(loader)?,
        })
    }
}

// The following is imported by each command impl module
mod command_impl_prelude {
    pub use super::Entry;
    pub use crate::enumerator_storage::{EnumeratorStorage, VulkanLenType};
    pub use crate::safe_interface::type_conversions::*;
    pub use crate::safe_interface::structs;
    pub use krs_hlist::Get;
    pub use vk_safe_sys as vk;
    pub use vk_safe_sys::{GetCommand, VulkanExtension, VulkanVersion};
}

// This is how each safe command can be implemented on top of each raw command
macro_rules! impl_safe_entry_interface {
    ( $interface:ident { $($code:tt)* }) => {
        impl<EntryVersion: VulkanVersion> $interface for Entry<EntryVersion> where EntryVersion::EntryCommands : GetCommand<vk::$interface> {
            $($code)*
        }
    };
}

mod create_instance;
mod enumerate_instance_extension_properties;
mod enumerate_instance_layer_properties;
mod enumerate_instance_version;

pub use create_instance::CreateInstance;
pub use enumerate_instance_extension_properties::EnumerateInstanceExtensionProperties;
pub use enumerate_instance_layer_properties::EnumerateInstanceLayerProperties;
pub use enumerate_instance_version::EnumerateInstanceVersion;
