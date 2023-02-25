use super::command_impl_prelude::*;

use std::mem::MaybeUninit;

/*
SAFETY (https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceVersion.html)

VUID-vkEnumerateInstanceVersion-pApiVersion-parameter
pApiVersion must be a valid pointer to a uint32_t value

- internally handled with a &mut u32
*/
impl_safe_entry_interface! {
EnumerateInstanceVersion {
    pub fn enumerate_instance_version(&self) -> Result<crate::utils::VkVersion, vk::Result> {
        let mut version = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(version.as_mut_ptr());
            check_raw_err!(res);
            Ok(crate::utils::VkVersion::from_raw(version.assume_init()))
        }
    }
}}
