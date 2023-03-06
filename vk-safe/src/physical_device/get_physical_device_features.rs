use super::*;
use vk_safe_sys as vk;
use krs_hlist::Get;
use crate::instance::InstanceConfig;

use std::mem::MaybeUninit;
use std::fmt;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFeatures.html

VUID-vkGetPhysicalDeviceFeatures-physicalDevice-parameter
physicalDevice must be a valid VkPhysicalDevice handle

- provided by vk_safe::PhysicalDevice

VUID-vkGetPhysicalDeviceFeatures-pFeatures-parameter
pFeatures must be a valid pointer to a VkPhysicalDeviceFeatures structure

- provided with MaybeUninit
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C> where C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceFeatures> {
    pub fn get_physical_device_features(&self) -> PhysicalDeviceFeatures {
        let mut features = MaybeUninit::uninit();
        unsafe {
            self.instance.feature_commands.get().get_fptr()(self.handle, features.as_mut_ptr());
            PhysicalDeviceFeatures { inner: features.assume_init() }
        }
    }
}

simple_struct_wrapper!(PhysicalDeviceFeatures);

impl fmt::Debug for PhysicalDeviceFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}