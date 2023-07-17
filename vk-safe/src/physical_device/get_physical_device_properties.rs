use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

use vk_safe_sys::validation::GetPhysicalDeviceProperties::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C>
where
    C::Commands: vk::GetCommand<vk::GetPhysicalDeviceProperties>,
{
    pub fn get_physical_device_properties(&self) -> PhysicalDeviceProperties {
        validate(Validation);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance.commands.get().get_fptr()(
                self.handle,
                properties.as_mut_ptr()
            );
            PhysicalDeviceProperties { inner:  properties.assume_init() }
        }
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkGetPhysicalDeviceProperties_physicalDevice_parameter: () = {
        // ensured by PhysicalDevice creation
    };

    const VUID_vkGetPhysicalDeviceProperties_pProperties_parameter: () = {
        // MaybeUninit
    };
}

check_vuid_defs!(
    pub const VUID_vkGetPhysicalDeviceProperties_physicalDevice_parameter: &'static [u8] =
            "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        pub const VUID_vkGetPhysicalDeviceProperties_pProperties_parameter: &'static [u8] =
            "pProperties must be a valid pointer to a VkPhysicalDeviceProperties structure"
                .as_bytes();
);

simple_struct_wrapper!(PhysicalDeviceProperties);

impl PhysicalDeviceProperties {
    pretty_version!(api_version);
    get_str!(device_name);
}

impl fmt::Debug for PhysicalDeviceProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceProperties")
            .field("api_version", &self.api_version())
            .field("driver_version", &self.driver_version)
            .field("vender_id", &self.vendor_id)
            .field("device_id", &self.device_id)
            .field("device_type", &self.device_type)
            .field("device_name", &self.device_name())
            .field("pipeline_cache_id", &self.pipeline_cache_uuid)
            .field("limits", &self.limits)
            .field("sparse_properties", &self.sparse_properties)
            .finish()
    }
}