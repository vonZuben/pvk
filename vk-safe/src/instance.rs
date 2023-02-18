use crate::safe_interface::type_conversions::ToC;
use krs_hlist::Get;
use vk_safe_sys as vk;

use vk::commands::{CommandLoadError, LoadCommands};

#[derive(Debug)]
pub struct Instance<V: vk::VulkanVersion, E: vk::VulkanExtension>
where
    V::InstanceCommands: vk::GetCommand<vk::DestroyInstance>,
{
    handle: vk::Instance,
    feature_commands: V::InstanceCommands,
    extension_commands: E::InstanceCommands,
}

impl<V: vk::VulkanVersion, E: vk::VulkanExtension> Instance<V, E>
where
    V::InstanceCommands: LoadCommands,
    E::InstanceCommands: LoadCommands,
    V::InstanceCommands: vk::GetCommand<vk::DestroyInstance>,
{
    pub(crate) fn new(handle: vk::Instance) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetInstanceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            feature_commands: V::InstanceCommands::load(loader)?,
            extension_commands: E::InstanceCommands::load(loader)?,
        })
    }
}

/*
SAFETY (https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyInstance.html)

VUID-vkDestroyInstance-instance-00629
All child objects created using instance must have been destroyed prior to destroying instance

- child objects borrow the Instance, so they should all be dropped (destroyed) before it is possible to Drop the Instance

VUID-vkDestroyInstance-instance-00630
If VkAllocationCallbacks were provided when instance was created, a compatible set of callbacks must be provided here

- the allocation callbacks from creation should be held by Instance, and used here

VUID-vkDestroyInstance-instance-00631
If no VkAllocationCallbacks were provided when instance was created, pAllocator must be NULL

- this follows from holding the allocation callbacks from creation

VUID-vkDestroyInstance-instance-parameter
If instance is not NULL, instance must be a valid VkInstance handle

- taken by rust ref so valid, and creation of all safe interface types should only make valid types

VUID-vkDestroyInstance-pAllocator-parameter
If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure

- taken by rust ref so valid
*/
impl<V: vk::VulkanVersion, E: vk::VulkanExtension> Drop for Instance<V, E>
where
    V::InstanceCommands: vk::GetCommand<vk::DestroyInstance>,
{
    fn drop(&mut self) {
        unsafe { self.feature_commands.get()(self.handle, None.to_c()) }
    }
}
