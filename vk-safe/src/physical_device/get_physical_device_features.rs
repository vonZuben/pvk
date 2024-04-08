use super::*;
use crate::instance_type::Instance;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFeatures;

use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFeatures.html
*/
impl<S, I: Instance> ScopedPhysicalDeviceType<S, I> {
    pub fn get_physical_device_features(&self) -> PhysicalDeviceFeatures<S>
    where
        I::Commands: GetPhysicalDeviceFeatures,
    {
        let mut features = MaybeUninit::uninit();
        unsafe {
            self.instance
                .commands
                .GetPhysicalDeviceFeatures()
                .get_fptr()(self.handle, features.as_mut_ptr());
            PhysicalDeviceFeatures::new(features.assume_init())
        }
    }
}

simple_struct_wrapper_scoped!(PhysicalDeviceFeatures impl Debug);

const _VUID: () = {
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
};
