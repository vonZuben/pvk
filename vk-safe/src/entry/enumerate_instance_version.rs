use super::command_impl_prelude::*;

use std::mem::MaybeUninit;

pub trait EnumerateInstanceVersion {
    fn enumerate_instance_version(&self) -> Result<crate::utils::VkVersion, vk::Result>;
}

impl_safe_entry_interface! {
EnumerateInstanceVersion {
    fn enumerate_instance_version(&self) -> Result<crate::utils::VkVersion, vk::Result> {
        let mut version = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(version.as_mut_ptr());
            check_raw_err!(res);
            Ok(crate::utils::VkVersion::from_raw(version.assume_init()))
        }
    }
}}
