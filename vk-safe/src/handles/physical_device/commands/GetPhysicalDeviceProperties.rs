use crate::handles::DispatchableHandle;
use crate::structs::PhysicalDeviceProperties;
use crate::type_conversions::ConvertWrapper;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

// use vk::has_command::GetPhysicalDeviceProperties;

pub trait GetPhysicalDeviceProperties: DispatchableHandle<RawHandle = vk::PhysicalDevice> {
    /// Query the properties of the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let physical_device_properties = physical_device.get_physical_device_properties();
    /// # }
    /// ```
    ///
    /// Vulkan docs:
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html>
    fn get_physical_device_properties<X>(&self) -> PhysicalDeviceProperties<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceProperties<X>,
    {
        use vk::has_command::GetPhysicalDeviceProperties;

        let mut properties = MaybeUninit::uninit();

        check_vuids::check_vuids!(GetPhysicalDeviceProperties);

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceProperties_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceProperties_pProperties_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pProperties must be a valid pointer to a VkPhysicalDeviceProperties structure"
            }

            // MaybeUninit
        }

        unsafe {
            self.commands().GetPhysicalDeviceProperties().get_fptr()(
                self.raw_handle(),
                properties.as_mut_ptr(),
            );
            PhysicalDeviceProperties::from_c(properties.assume_init())
        }
    }
}
