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

- checked with const Verify check

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

- TODO

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
    #[track_caller]
    pub fn get_physical_device_image_format_properties(
        &self,
        format: impl vk::FormatConst,
        image_type: impl vk::ImageTypeConst,
        image_tiling: impl vk::ImageTilingConst,
        usage_flags: impl vk::ImageUsageFlagsConst,
        create_flags: impl vk::ImageCreateFlagsConst,//vk::ImageCreateFlags,
    ) -> Result<ImageFormatProperties, vk::Result> {
        Params::verify(image_tiling);
        let mut properties = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.feature_commands.get().get_fptr()(
                self.handle,
                format.variant(),
                image_type.variant(),
                image_tiling.variant(),
                usage_flags.bitmask(),
                create_flags.bitmask(),
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties {
                inner: properties.assume_init(),
            })
        }
    }
}

verify_params!(Params(T: vk::ImageTilingConst) {
    use vk::VkEnumVariant;
    if T::VARIANT == vk::image_tiling::DRM_FORMAT_MODIFIER_EXT::VARIANT {
        panic!("image_tiling must not be VK_IMAGE_TILING_DRM_FORMAT_MODIFIER_EXT. (Use vkGetPhysicalDeviceImageFormatProperties2 instead)")
    }
});

simple_struct_wrapper!(ImageFormatProperties);

impl fmt::Debug for ImageFormatProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
