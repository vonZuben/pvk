/*!
Query the features supported by the PhysicalDevice

use the [`get_physical_device_features`](ScopedPhysicalDevice::get_physical_device_features) method on a scoped PhysicalDevice

Vulkan docs:
<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFeatures.html>
*/

use super::concrete_type::ScopedPhysicalDevice;

use crate::dispatchable_handles::instance::Instance;

use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFeatures;

use std::mem::MaybeUninit;

impl<S, I: Instance> ScopedPhysicalDevice<S, I>
where
    I::Context: GetPhysicalDeviceFeatures,
{
    /**
    Query the features supported by the PhysicalDevice

    ```rust
    # use vk_safe::vk;
    # vk::device_context!(D: VERSION_1_0);
    # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    #   (physical_device: P) {
    let features = physical_device.get_physical_device_features();
    # }
    ```
    */
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
