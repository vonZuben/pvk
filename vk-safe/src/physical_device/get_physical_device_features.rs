use super::*;
use crate::instance::ScopedInstance;
use vk_safe_sys as vk;

use vk::has_command::GetPhysicalDeviceFeatures;

use std::mem::MaybeUninit;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFeatures.html
*/
impl<'scope, I: ScopedInstance> ScopedPhysicalDeviceType<'scope, I> {
    pub fn get_physical_device_features<P>(&self) -> PhysicalDeviceFeatures<'scope>
    where
        I::Commands: GetPhysicalDeviceFeatures<P>,
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
    check_vuid_defs2!( GetPhysicalDeviceFeatures
        pub const VUID_vkGetPhysicalDeviceFeatures_physicalDevice_parameter: &'static [u8] =
        "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
        // ensured by PhysicalDevice creation

        pub const VUID_vkGetPhysicalDeviceFeatures_pFeatures_parameter: &'static [u8] =
        "pFeatures must be a valid pointer to a VkPhysicalDeviceFeatures structure".as_bytes();
        // using MaybeUninit
    )
};
