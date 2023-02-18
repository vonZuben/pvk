use super::command_impl_prelude::*;

use crate::instance as safe_instance;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct TempError;

pub trait CreateInstance {
    fn create_instance<V: vk::VulkanVersion, E: vk::VulkanExtension>(
        &self,
        create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>,
    ) -> std::result::Result<safe_instance::Instance<V, E>, TempError>
    where
        V::InstanceCommands: vk::commands::LoadCommands + vk::GetCommand<vk::DestroyInstance>,
        E::InstanceCommands: vk::commands::LoadCommands;
}

/*
SAFETY (https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateInstance.html)

VUID-vkCreateInstance-ppEnabledExtensionNames-01388
All required extensions for each extension in the VkInstanceCreateInfo::ppEnabledExtensionNames list must also be present in that list

- TODO should ensure safety by creation of the create_info

VUID-vkCreateInstance-pCreateInfo-parameter
pCreateInfo must be a valid pointer to a valid VkInstanceCreateInfo structure

- taken by rust ref so valid, and creation of all safe interface types should only make valid types

VUID-vkCreateInstance-pAllocator-parameter
If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure

- taken by rust ref so valid, and creation of all safe interface types should only make valid types

VUID-vkCreateInstance-pInstance-parameter
pInstance must be a valid pointer to a VkInstance handle

- we pass a valid pointer to the location where the function will return the instance handle
*/

impl_safe_entry_interface! {
CreateInstance {
    fn create_instance<V: vk::VulkanVersion, E: vk::VulkanExtension>(
        &self,
        create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>,
    ) -> std::result::Result<safe_instance::Instance<V, E>, TempError>
    where
        V::InstanceCommands: vk::commands::LoadCommands + vk::GetCommand<vk::DestroyInstance>,
        E::InstanceCommands: vk::commands::LoadCommands
    {
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(&create_info.inner, None.to_c(), instance.as_mut_ptr());
            if res.is_err() {
                return Err(TempError);
            }
            Ok(safe_instance::Instance::new(instance.assume_init()).map_err(|_|TempError)?)
        }
    }
}}
