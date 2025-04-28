use std::mem::MaybeUninit;

use crate::handles::DispatchableHandle;
use crate::structs::PhysicalDeviceMemoryProperties;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

pub trait GetPhysicalDeviceMemoryProperties:
    DispatchableHandle<RawHandle = vk::PhysicalDevice>
{
    /// Query the memory properties of the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let memory_properties = physical_device.get_physical_device_memory_properties();
    /// # }
    /// ```
    fn get_physical_device_memory_properties<X>(&self) -> PhysicalDeviceMemoryProperties<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceMemoryProperties<X>,
    {
        use vk::has_command::GetPhysicalDeviceMemoryProperties;

        check_vuids::check_vuids!(GetPhysicalDeviceMemoryProperties);

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceMemoryProperties_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceMemoryProperties_pMemoryProperties_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pMemoryProperties must be a valid pointer to a VkPhysicalDeviceMemoryProperties structure"
            }

            // MaybeUninit
        }

        let mut properties = MaybeUninit::uninit();
        unsafe {
            self.commands()
                .GetPhysicalDeviceMemoryProperties()
                .get_fptr()(self.raw_handle(), properties.as_mut_ptr());
            PhysicalDeviceMemoryProperties::from_c(properties.assume_init())
        }
    }
}
