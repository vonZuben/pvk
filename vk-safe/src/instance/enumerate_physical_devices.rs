use super::command_impl_prelude::*;

use crate::enumerator_storage::EnumeratorStorage;
use crate::instance::InstanceConfig;
use crate::physical_device::PhysicalDevices;

use vk::validation::EnumeratePhysicalDevices::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html
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
        validate(Validation);
        let handles = enumerator_code2!(self.handle, self.commands; () -> storage);
        Ok(PhysicalDevices::new(handles, self))
    }
}}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkEnumeratePhysicalDevices_instance_parameter: () = {
        // This is guaranteed by the creation of the Instance which will have a valid handle
    };

    const VUID_vkEnumeratePhysicalDevices_pPhysicalDeviceCount_parameter: () = {
        // This is handled by enumerator_code2!() which ensure the proper count is provided
    };

    const VUID_vkEnumeratePhysicalDevices_pPhysicalDevices_parameter: () = {
        // This is handled by enumerator_code2!() and EnumeratorStorage which ensure proper storage space is provided;
    };
}

check_vuid_defs!(
    pub const VUID_vkEnumeratePhysicalDevices_instance_parameter: &'static [u8] = "instance must be a valid VkInstance handle".as_bytes();
    pub const VUID_vkEnumeratePhysicalDevices_pPhysicalDeviceCount_parameter: &'static [u8] = "pPhysicalDeviceCount must be a valid pointer to a uint32_t value".as_bytes();
    pub const VUID_vkEnumeratePhysicalDevices_pPhysicalDevices_parameter : &'static [u8] = "If the value referenced by pPhysicalDeviceCount is not 0, and pPhysicalDevices is not NULL, pPhysicalDevices must be a valid pointer to an array of pPhysicalDeviceCount VkPhysicalDevice handles".as_bytes();
);