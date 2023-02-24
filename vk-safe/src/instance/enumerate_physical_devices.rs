use super::command_impl_prelude::*;

use crate::enumerator_storage::EnumeratorStorage;
use crate::instance::InstanceConfig;
use crate::physical_device::PhysicalDevice;

pub trait EnumeratePhysicalDevices {
    type Config: InstanceConfig;
    // Self is intended to be Instance<C>
    fn enumerate_physical_devices<
        'a,
        S: EnumeratorStorage<PhysicalDevice<'a, Self::Config>>,
    >(
        &'a self,
        storage: S
    ) -> Result<S::InitStorage, vk::Result>;
}

impl<C: InstanceConfig> EnumeratePhysicalDevices for Instance<C> where C::InstanceCommands: GetCommand<vk::EnumeratePhysicalDevices> {
    type Config = C;

    fn enumerate_physical_devices<
        'a,
        S: EnumeratorStorage<PhysicalDevice<'a, Self::Config>>,
    >(
        &'a self,
        mut storage: S
    ) -> Result<S::InitStorage, vk::Result> {
        use std::convert::TryInto;
        let query_len = || {
            let mut num = 0;
            let res;
            unsafe {
                res = self.feature_commands.get()(self.handle, &mut num, std::ptr::null_mut());
                check_raw_err!(res);
            }
            Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
        };
        storage.query_len(query_len)?;
        let uninit_slice = storage.uninit_slice();
        let mut len = VulkanLenType::from_usize(uninit_slice.len());
        let res;
        unsafe {
            res = self.feature_commands.get()(self.handle, &mut len, uninit_slice.as_mut_ptr().cast());
            check_raw_err!(res);
        }
        Ok(storage.finalize(len.to_usize()))
    }
}