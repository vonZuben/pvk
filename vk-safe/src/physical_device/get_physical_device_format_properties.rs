use super::*;
use crate::instance_type::Instance;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFormatProperties;

use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html
*/
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn get_physical_device_format_properties(&self, format: vk::Format) -> FormatProperties<S>
    where
        I::Commands: GetPhysicalDeviceFormatProperties,
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
    check_vuids::check_vuids!(GetPhysicalDeviceFormatProperties);

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFormatProperties_physicalDevice_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "physicalDevice must be a valid VkPhysicalDevice handle"
        }

        // valid from creation
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFormatProperties_format_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "format must be a valid VkFormat value"
        }

        // vk::Format
    }

    #[allow(unused_labels)]
    'VUID_vkGetPhysicalDeviceFormatProperties_pFormatProperties_parameter: {
        check_vuids::version! {"1.3.268"}
        check_vuids::description! {
        "pFormatProperties must be a valid pointer to a VkFormatProperties structure"
        }

        // MaybeUninit
    }
};
