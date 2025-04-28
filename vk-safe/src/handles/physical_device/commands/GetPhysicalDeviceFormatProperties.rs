use std::mem::MaybeUninit;

use crate::handles::DispatchableHandle;
use crate::structs::FormatProperties;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

pub trait GetPhysicalDeviceFormatProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    /// Query the format properties of the PhysicalDevice
    ///
    /// Provide the [`Format`](crate::vk::Format) to get the properties of that format
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let format_properties =
    ///     physical_device.get_physical_device_format_properties(vk::Format::R8G8B8A8_SRGB);
    /// # }
    /// ```
    fn get_physical_device_format_properties<F: vk::enum_traits::Format, X>(
        &self,
        _format: F,
    ) -> FormatProperties<Self, F>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceFormatProperties<X>,
    {
        use vk::has_command::GetPhysicalDeviceFormatProperties;

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

        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.commands()
                .GetPhysicalDeviceFormatProperties()
                .get_fptr()(self.raw_handle(), F::VALUE, properties.as_mut_ptr());
            FormatProperties::from_c(properties.assume_init())
        }
    }
}
