use super::*;
use vk_safe_sys as vk;
use krs_hlist::Get;
use crate::instance::InstanceConfig;

use std::mem::MaybeUninit;
use std::fmt;

use vk_safe_sys::validation::GetPhysicalDeviceFormatProperties::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C> where C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceFormatProperties> {
    pub fn get_physical_device_format_properties(&self, format: impl vk::FormatConst) -> FormatProperties {
        validate(Validation);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance.feature_commands.get().get_fptr()(self.handle, format.variant(), properties.as_mut_ptr());
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

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkGetPhysicalDeviceFormatProperties_physicalDevice_parameter: () = {
        // ensured by PhysicalDevice creation
    };

    const VUID_vkGetPhysicalDeviceFormatProperties_format_parameter: () = {
        // ensured by FormatConst
    };

    const VUID_vkGetPhysicalDeviceFormatProperties_pFormatProperties_parameter: () = {
        // using MaybeUninit
    };
}

check_vuid_defs!(
    pub const VUID_vkGetPhysicalDeviceFormatProperties_physicalDevice_parameter:
            &'static [u8] = "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        pub const VUID_vkGetPhysicalDeviceFormatProperties_format_parameter: &'static [u8] =
            "format must be a valid VkFormat value".as_bytes();
        pub const VUID_vkGetPhysicalDeviceFormatProperties_pFormatProperties_parameter:
            &'static [u8] =
            "pFormatProperties must be a valid pointer to a VkFormatProperties structure"
                .as_bytes();
);