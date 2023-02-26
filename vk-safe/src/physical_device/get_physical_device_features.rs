use super::*;
use vk_safe_sys as vk;
use krs_hlist::Get;
use crate::instance::InstanceConfig;

use std::mem::MaybeUninit;
use std::fmt;

impl<C: InstanceConfig> PhysicalDevice<'_, C> where C::InstanceCommands: vk::GetCommand<vk::GetPhysicalDeviceFeatures> {
    pub fn get_physical_device_features(&self) -> PhysicalDeviceFeatures {
        let mut features = MaybeUninit::uninit();
        unsafe {
            self.instance.feature_commands.get()(self.handle, features.as_mut_ptr());
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