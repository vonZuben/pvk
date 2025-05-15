use crate::handles::DispatchableHandle;
use crate::struct_extension::{LinkMut, Pnext};
use crate::structs::PhysicalDeviceFeatures2;

use vk_safe_sys as vk;

pub trait GetPhysicalDeviceFeatures2: DispatchableHandle<RawHandle = vk::PhysicalDevice> {
    /// Reports capabilities of a physical device (extendable version of get_physical_device_features)
    ///
    /// <https://registry.khronos.org/vulkan/specs/latest/man/html/vkGetPhysicalDeviceFeatures2.html>
    fn get_physical_device_features2<X, Pn: Pnext<PhysicalDeviceFeatures2<Self>, Self>>(
        &self,
        _features: Pn,
    ) -> Pn::Output<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceFeatures2<X>,
    {
        use vk::has_command::GetPhysicalDeviceFeatures2;

        check_vuids::check_vuids!(GetPhysicalDeviceFeatures2);

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceFeatures2_physicalDevice_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "physicalDevice must be a valid VkPhysicalDevice handle"
            }

            // ensured by PhysicalDevice creation
        }

        #[allow(unused_labels)]
        'VUID_vkGetPhysicalDeviceFeatures2_pFeatures_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "pFeatures must be a valid pointer to a VkPhysicalDeviceFeatures2 structure"
            }

            // Pnext::p_next_uninit
        }

        let mut features = Pn::p_next_uninit();

        unsafe {
            self.commands().GetPhysicalDeviceFeatures2().get_fptr()(
                self.raw_handle(),
                LinkMut::link_mut(features.as_mut_ptr()),
            );
            features.assume_init()
        }
    }
}
