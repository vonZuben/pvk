use super::command_impl_prelude::*;

use crate::array_storage::ArrayStorage;
use crate::instance::InstanceConfig;
use crate::physical_device::PhysicalDevices;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html
*/
impl_safe_instance_interface!{
EnumeratePhysicalDevices {
    pub fn enumerate_physical_devices<
        S: ArrayStorage<vk::PhysicalDevice>,
    >(
        &self,
        mut storage: S
    ) -> Result<PhysicalDevices<'scope, C, S>, vk::Result> {

        // handled by enumerator_code2!() and instance creation
        check_vuid_defs2!( EnumeratePhysicalDevices
            pub const VUID_vkEnumeratePhysicalDevices_instance_parameter: &'static [u8] = "instance must be a valid VkInstance handle".as_bytes();
            pub const VUID_vkEnumeratePhysicalDevices_pPhysicalDeviceCount_parameter: &'static [u8] = "pPhysicalDeviceCount must be a valid pointer to a uint32_t value".as_bytes();
            pub const VUID_vkEnumeratePhysicalDevices_pPhysicalDevices_parameter : &'static [u8] = "If the value referenced by pPhysicalDeviceCount is not 0, and pPhysicalDevices is not NULL, pPhysicalDevices must be a valid pointer to an array of pPhysicalDeviceCount VkPhysicalDevice handles".as_bytes();
        );

        let handles = enumerator_code2!(self.handle, self.commands; () -> storage);
        Ok(PhysicalDevices::new(handles, *self))
    }
}}