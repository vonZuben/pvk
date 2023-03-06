use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceImageFormatProperties.html

VUID-vkGetPhysicalDeviceImageFormatProperties-tiling-02248
tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)

- currently ensured by an internal runtime check - TODO can we provide type level safety for this?

VUID-vkGetPhysicalDeviceImageFormatProperties-physicalDevice-parameter
physicalDevice must be a valid VkPhysicalDevice handle

- provided by vk_safe::PhysicalDevice

VUID-vkGetPhysicalDeviceImageFormatProperties-format-parameter
format must be a valid VkFormat value

- provided by vk::Format

VUID-vkGetPhysicalDeviceImageFormatProperties-type-parameter
type must be a valid VkImageType value

- provided by vk::ImageType

VUID-vkGetPhysicalDeviceImageFormatProperties-tiling-parameter
tiling must be a valid VkImageTiling value

- provided by vk::ImageTiling

VUID-vkGetPhysicalDeviceImageFormatProperties-usage-parameter
usage must be a valid combination of VkImageUsageFlagBits values

- provided by vk::ImageUsageFlags (*NOTE* I understand there is no "invalid" combination of bits, as long as only the defined bits are used)

VUID-vkGetPhysicalDeviceImageFormatProperties-usage-requiredbitmask
usage must not be 0

- TODO - make bitflags safe to use by being more limiting on what can be

VUID-vkGetPhysicalDeviceImageFormatProperties-flags-parameter
flags must be a valid combination of VkImageCreateFlagBits values

- TODO

VUID-vkGetPhysicalDeviceImageFormatProperties-pImageFormatProperties-parameter
pImageFormatProperties must be a valid pointer to a VkImageFormatProperties structure

- TODO
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C>
where
    C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceImageFormatProperties>,
{
    pub fn get_physical_device_image_format_properties(
        &self,
        format: vk::Format,
        image_type: vk::ImageType,
        image_tiling: vk::ImageTiling,
        usage_flags: vk::ImageUsageFlags,
        create_flags: vk::ImageCreateFlags,
    ) -> Result<ImageFormatProperties, vk::Result> {
        if image_tiling == vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT {
            panic!("tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)");
        }
        let mut properties = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.feature_commands.get().get_fptr()(
                self.handle,
                format,
                image_type,
                image_tiling,
                usage_flags,
                create_flags,
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties {
                inner: properties.assume_init(),
            })
        }
    }
}

simple_struct_wrapper!(ImageFormatProperties);

impl fmt::Debug for ImageFormatProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
