use super::PhysicalDevice;

use crate::error::Error;
use crate::structs::{ImageFormatProperties, ImageParameters::ImageParameters};

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceImageFormatProperties;

pub(crate) fn get_physical_device_image_format_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceImageFormatProperties>,
    Params: ImageParameters,
>(
    physical_device: &P,
    params: Params,
) -> Result<ImageFormatProperties<P, Params>, Error> {
    // *************Regarding VUID checks**************
    // please see the checks for [ImageParameters]

    let mut properties = MaybeUninit::uninit();
    let command = physical_device
        .commands()
        .GetPhysicalDeviceImageFormatProperties()
        .get_fptr();
    unsafe {
        let res = command(
            physical_device.raw_handle(),
            Params::format(),
            Params::image_type(),
            Params::image_tiling(),
            Params::image_usage_flags(),
            Params::image_create_flags(),
            properties.as_mut_ptr(),
        );
        check_raw_err!(res);
        Ok(ImageFormatProperties::new(properties.assume_init(), params))
    }
}
