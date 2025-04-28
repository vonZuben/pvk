use crate::error::Error;
use crate::handles::DispatchableHandle;
use crate::structs::{ImageFormatProperties, ImageParameters::ImageParameters};
use crate::type_conversions::ConvertWrapper;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

pub trait GetPhysicalDeviceImageFormatProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    /// Query the image format properties of the PhysicalDevice
    ///
    /// Provide [`ImageParameters`] with the parameters of an image,
    /// to get the format properties of an image created with such parameters
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let image_params = vk::ImageParameters::new(
    ///     vk::Format::R8G8B8A8_SRGB,
    ///     vk::ImageType::TYPE_2D,
    ///     vk::ImageTiling::OPTIMAL,
    ///     vk::flags!(ImageUsageFlags + COLOR_ATTACHMENT_BIT + TRANSFER_DST_BIT),
    ///     (),
    /// );
    ///
    /// let image_format_properties =
    /// physical_device.get_physical_device_image_format_properties(image_params);
    /// # }
    /// ```
    fn get_physical_device_image_format_properties<Params: ImageParameters, X>(
        &self,
        _params: Params,
    ) -> Result<ImageFormatProperties<Self, Params>, Error>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceImageFormatProperties<X>,
    {
        use vk::has_command::GetPhysicalDeviceImageFormatProperties;
        // *************Regarding VUID checks**************
        // please see the checks for [ImageParameters]

        let mut properties = MaybeUninit::uninit();
        let command = self
            .commands()
            .GetPhysicalDeviceImageFormatProperties()
            .get_fptr();
        unsafe {
            let res = command(
                self.raw_handle(),
                Params::format(),
                Params::image_type(),
                Params::image_tiling(),
                Params::image_usage_flags(),
                Params::image_create_flags(),
                properties.as_mut_ptr(),
            );
            check_raw_err!(res);
            Ok(ImageFormatProperties::from_c(properties.assume_init()))
        }
    }
}
