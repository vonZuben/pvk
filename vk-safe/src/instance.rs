
use vk_safe_sys as vk;

use vk::commands::{LoadCommands, CommandLoadError};

#[derive(Debug)]
pub struct Instance<V: vk::VulkanVersion, E: vk::VulkanExtension> {
    handle: vk::Instance,
    feature_commands: V::InstanceCommands,
    extension_commands: E::InstanceCommands,
}

impl<V: vk::VulkanVersion, E: vk::VulkanExtension> Instance<V, E> where V::InstanceCommands: LoadCommands, E::InstanceCommands: LoadCommands {
    pub(crate) fn new(handle: vk::Instance) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe {
            vk::GetInstanceProcAddr(handle, command_name)
        };
        Ok(Self {
            handle,
            feature_commands: V::InstanceCommands::load(loader)?,
            extension_commands: E::InstanceCommands::load(loader)?,
        })
    }
}