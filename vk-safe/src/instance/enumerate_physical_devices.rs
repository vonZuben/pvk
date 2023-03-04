use super::command_impl_prelude::*;

use crate::enumerator_storage::EnumeratorStorage;
use crate::instance::InstanceConfig;
use crate::physical_device::PhysicalDevices;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html

VUID-vkEnumeratePhysicalDevices-instance-parameter
instance must be a valid VkInstance handle

- provided by vk_safe::Instance

VUID-vkEnumeratePhysicalDevices-pPhysicalDeviceCount-parameter
pPhysicalDeviceCount must be a valid pointer to a uint32_t value

- handled by enumerator_code2

VUID-vkEnumeratePhysicalDevices-pPhysicalDevices-parameter
If the value referenced by pPhysicalDeviceCount is not 0, and pPhysicalDevices is not NULL, pPhysicalDevices must be a valid pointer to an array of pPhysicalDeviceCount VkPhysicalDevice handles

- handled by enumerator_code2 and EnumeratorStorage
*/
impl_safe_instance_interface!{
EnumeratePhysicalDevices {
    pub fn enumerate_physical_devices<
        'a,
        S: EnumeratorStorage<vk::PhysicalDevice>,
    >(
        &'a self,
        mut storage: S
    ) -> Result<PhysicalDevices<'a, C, S>, vk::Result> {
        let handles = enumerator_code2!(self.handle, self.feature_commands; () -> storage);
        Ok(PhysicalDevices::new(handles, self))
    }
}}