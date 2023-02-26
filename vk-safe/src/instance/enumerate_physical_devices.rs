use super::command_impl_prelude::*;

use crate::enumerator_storage::EnumeratorStorage;
use crate::instance::InstanceConfig;
use crate::physical_device::PhysicalDevices;

impl_safe_instance_interface!{
EnumeratePhysicalDevices {
    pub fn enumerate_physical_devices<
        'a,
        S: EnumeratorStorage<vk::PhysicalDevice>,
    >(
        &'a self,
        mut storage: S
    ) -> Result<PhysicalDevices<'a, C, S>, vk::Result> {
        let handles = enumerator_code2!(self.handle, self.feature_commands; () -> storage);
        Ok(PhysicalDevices::new(handles, self))
    }
}}