use vk_safe_sys as vk;

use std::fmt;

use crate::instance::{Instance, InstanceConfig};

pub struct PhysicalDevice<'a, C: InstanceConfig> {
    handle: vk::PhysicalDevice,
    instance: &'a Instance<C>,
}

impl<'a, C: InstanceConfig> PhysicalDevice<'a, C> {
    pub fn new(handle: vk::PhysicalDevice, instance: &'a Instance<C>) -> Self {
        Self { handle, instance }
    }
}

impl<C: InstanceConfig> fmt::Debug for PhysicalDevice<'_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDevice").field("handle", &self.handle).finish()
    }
}