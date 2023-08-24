use super::*;
use crate::instance::InstanceConfig;
use vk::GetCommand;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html
*/
impl<'scope, C: InstanceConfig> ScopedPhysicalDevice<'scope, '_, C>
where
    C::Commands: GetCommand<vk::GetPhysicalDeviceProperties>,
{
    pub fn get_physical_device_properties(&self) -> PhysicalDeviceProperties<'scope> {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance.commands.get_command().get_fptr()(
                self.handle,
                properties.as_mut_ptr()
            );
            PhysicalDeviceProperties::new(properties.assume_init())
        }
    }
}

const _VUID: () = {
    check_vuid_defs2!( GetPhysicalDeviceProperties
        pub const VUID_vkGetPhysicalDeviceProperties_physicalDevice_parameter: &'static [u8] =
            "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        // ensured by PhysicalDevice creation
        pub const VUID_vkGetPhysicalDeviceProperties_pProperties_parameter: &'static [u8] =
            "pProperties must be a valid pointer to a VkPhysicalDeviceProperties structure"
                .as_bytes();
        // MaybeUninit
    )
};

simple_struct_wrapper_scoped!(PhysicalDeviceProperties impl Deref);

impl PhysicalDeviceProperties<'_> {
    pretty_version!(api_version);
    get_str!(device_name);
}

impl fmt::Debug for PhysicalDeviceProperties<'_> {
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