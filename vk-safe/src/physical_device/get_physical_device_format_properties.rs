use super::*;
use crate::instance::InstanceConfig;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFormatProperties;

use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html
*/
impl<'scope, C: InstanceConfig> ScopedPhysicalDevice<'scope, '_, C> {
    pub fn get_physical_device_format_properties<P>(
        &self,
        format: vk::Format,
    ) -> FormatProperties<'scope>
    where
        C::Commands: GetPhysicalDeviceFormatProperties<P>,
    {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance
                .commands
                .GetPhysicalDeviceFormatProperties()
                .get_fptr()(self.handle, format, properties.as_mut_ptr());
            FormatProperties::new(properties.assume_init())
        }
    }
}

simple_struct_wrapper_scoped!(FormatProperties impl Debug);

const _VUID: () = {
    check_vuid_defs2!( GetPhysicalDeviceFormatProperties
        pub const VUID_vkGetPhysicalDeviceFormatProperties_physicalDevice_parameter:
            &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        // ensured by PhysicalDevice creation
        pub const VUID_vkGetPhysicalDeviceFormatProperties_format_parameter: &'static [u8] =
            "format must be a valid VkFormat value".as_bytes();
        // ensured by Format (can only make valid values)
        pub const VUID_vkGetPhysicalDeviceFormatProperties_pFormatProperties_parameter:
            &'static [u8] =
            "pFormatProperties must be a valid pointer to a VkFormatProperties structure"
                .as_bytes();
        // using MaybeUninit
    )
};
