use super::*;
use crate::instance::InstanceConfig;
use krs_hlist::Get;
use vk_safe_sys as vk;

use std::fmt;
use std::mem::MaybeUninit;

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
        let mut properties = MaybeUninit::uninit();
        unsafe {
            let res = self.instance.feature_commands.get()(
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
