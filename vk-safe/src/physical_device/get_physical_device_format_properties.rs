use super::*;
use vk_safe_sys as vk;
use krs_hlist::Get;
use crate::instance::InstanceConfig;

use std::mem::MaybeUninit;
use std::fmt;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html

VUID-vkGetPhysicalDeviceFormatProperties-physicalDevice-parameter
physicalDevice must be a valid VkPhysicalDevice handle

- provided by vk_safe::PhysicalDevice

VUID-vkGetPhysicalDeviceFormatProperties-format-parameter
format must be a valid VkFormat value

- provided by vk::Format

VUID-vkGetPhysicalDeviceFormatProperties-pFormatProperties-parameter
pFormatProperties must be a valid pointer to a VkFormatProperties structure

- provided with MaybeUninit
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C> where C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceFormatProperties> {
    pub fn get_physical_device_format_properties(&self, format: vk::Format) -> FormatProperties {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance.feature_commands.get()(self.handle, format, properties.as_mut_ptr());
            FormatProperties { inner: properties.assume_init() }
        }
    }
}

simple_struct_wrapper!(FormatProperties);

impl fmt::Debug for FormatProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}