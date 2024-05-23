/*!
Query the format properties of the PhysicalDevice

use the [`get_physical_device_format_properties`](ScopedPhysicalDevice::get_physical_device_format_properties) method on a scoped PhysicalDevice

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFormatProperties.html>
*/

use super::concrete_type::ScopedPhysicalDevice;

use crate::dispatchable_handles::instance::Instance;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFormatProperties;

use std::mem::MaybeUninit;

impl<S, I: Instance> ScopedPhysicalDevice<S, I>
where
    I::Context: GetPhysicalDeviceFormatProperties,
{
    /**
    Query the format properties of the PhysicalDevice

    Provide the [`Format`](crate::vk::Format) to get the properties of that format

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    #   (physical_device: P) {
    let format_properties =
        physical_device.get_physical_device_format_properties(vk::Format::R8G8B8A8_SRGB);
    # }
    ```
    */
    pub fn get_physical_device_format_properties(&self, format: vk::Format) -> FormatProperties<S> {
        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.instance
                .context
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
