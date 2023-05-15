use super::*;
use vk_safe_sys as vk;
use krs_hlist::Get;
use crate::instance::InstanceConfig;

use std::mem::MaybeUninit;
use std::fmt;

use vk_safe_sys::validation::GetPhysicalDeviceFeatures::*;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceFeatures.html
*/
impl<C: InstanceConfig> PhysicalDevice<'_, C> where C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceFeatures> {
    pub fn get_physical_device_features(&self) -> PhysicalDeviceFeatures {
        validate(Validation);
        let mut features = MaybeUninit::uninit();
        unsafe {
            self.instance.feature_commands.get().get_fptr()(self.handle, features.as_mut_ptr());
            PhysicalDeviceFeatures { inner: features.assume_init() }
        }
    }
}

simple_struct_wrapper!(PhysicalDeviceFeatures);

impl fmt::Debug for PhysicalDeviceFeatures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

struct Validation;

#[allow(non_upper_case_globals)]
impl Vuids for Validation {
    const VUID_vkGetPhysicalDeviceFeatures_physicalDevice_parameter: () = {
        // ensured by PhysicalDevice creation
    };

    const VUID_vkGetPhysicalDeviceFeatures_pFeatures_parameter: () = {
        // using MaybeUninit
    };
}

check_vuid_defs!(
    pub const VUID_vkGetPhysicalDeviceFeatures_physicalDevice_parameter: &'static [u8] =
    "physicalDevice must be a valid VkPhysicalDevice handle".as_bytes();
    pub const VUID_vkGetPhysicalDeviceFeatures_pFeatures_parameter: &'static [u8] =
    "pFeatures must be a valid pointer to a VkPhysicalDeviceFeatures structure".as_bytes();
);