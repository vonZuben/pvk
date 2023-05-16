use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;

use vk_safe_sys::validation::GetPhysicalDeviceQueueFamilyProperties::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C>
where
    C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceQueueFamilyProperties>,
{
    pub fn get_physical_device_queue_family_properties<S: EnumeratorStorage<QueueFamilyProperties>>(&self, mut storage: S) -> S::InitStorage {
        enumerator_code_non_fail!(self.handle, self.instance.feature_commands; () -> storage)
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter: () = {
        // PhysicalDevice
    };

    const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter : ( ) = {
        // enumerator_code2
    };

    const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter: () = {
        // enumerator_code2
    };
}

check_vuid_defs!(
    pub const VUID_vkGetPhysicalDeviceQueueFamilyProperties_physicalDevice_parameter:
            &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        pub const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyPropertyCount_parameter : & 'static [ u8 ] = "pQueueFamilyPropertyCount must be a valid pointer to a uint32_t value" . as_bytes ( ) ;
        pub const VUID_vkGetPhysicalDeviceQueueFamilyProperties_pQueueFamilyProperties_parameter : & 'static [ u8 ] = "If the value referenced by pQueueFamilyPropertyCount is not 0, and pQueueFamilyProperties is not NULL, pQueueFamilyProperties must be a valid pointer to an array of pQueueFamilyPropertyCount VkQueueFamilyProperties structures" . as_bytes ( ) ;
);

simple_struct_wrapper!(QueueFamilyProperties);

impl fmt::Debug for QueueFamilyProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}