use super::PhysicalDevice;

use std::mem::MaybeUninit;

use crate::structs::FormatProperties;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFormatProperties;

pub(crate) fn get_physical_device_format_properties<
    P: PhysicalDevice<Commands: GetPhysicalDeviceFormatProperties>,
>(
    physical_device: &P,
    format: vk::Format,
) -> FormatProperties<P> {
    let mut properties = MaybeUninit::uninit();
    unsafe {
        physical_device
            .commands()
            .GetPhysicalDeviceFormatProperties()
            .get_fptr()(
            physical_device.raw_handle(),
            format,
            properties.as_mut_ptr(),
        );
        FormatProperties::new(properties.assume_init())
    }
}
