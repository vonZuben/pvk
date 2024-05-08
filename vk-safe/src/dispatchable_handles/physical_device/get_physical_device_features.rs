use super::concrete_type::ScopedPhysicalDeviceType;

use crate::dispatchable_handles::instance::Instance;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFeatures;

use std::mem::MaybeUninit;

impl<S, I: Instance> ScopedPhysicalDeviceType<S, I>
where
    I::Context: GetPhysicalDeviceFeatures,
{
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFeatures.html>
    pub fn get_physical_device_features(&self) -> PhysicalDeviceFeatures<S> {
        let mut features = MaybeUninit::uninit();
        unsafe {
            self.instance.context.GetPhysicalDeviceFeatures().get_fptr()(
                self.handle,
                features.as_mut_ptr(),
            );
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
