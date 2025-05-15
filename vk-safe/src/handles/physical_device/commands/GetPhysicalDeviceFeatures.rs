use crate::handles::DispatchableHandle;
use crate::structs::PhysicalDeviceFeatures;
use crate::type_conversions::ConvertWrapper;

use std::mem::MaybeUninit;

use vk_safe_sys as vk;

pub trait GetPhysicalDeviceFeatures: DispatchableHandle<RawHandle = vk::PhysicalDevice> {
    /// Reports capabilities of a physical device
    ///
    /// <https://registry.khronos.org/vulkan/specs/latest/man/html/vkGetPhysicalDeviceFeatures.html>
    fn get_physical_device_features<X>(&self) -> PhysicalDeviceFeatures<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceFeatures<X>,
    {
        use vk::has_command::GetPhysicalDeviceFeatures;

        check_vuids::check_vuids!(GetPhysicalDeviceFeatures);

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceFeatures_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // valid from creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceFeatures_pFeatures_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pFeatures must be a valid pointer to a VkPhysicalDeviceFeatures structure"
            }

            // MaybeUninit
        }

        let mut features = MaybeUninit::uninit();
        unsafe {
            self.commands().GetPhysicalDeviceFeatures().get_fptr()(
                self.raw_handle(),
                features.as_mut_ptr(),
            );
            PhysicalDeviceFeatures::from_c(features.assume_init())
        }
    }
}
