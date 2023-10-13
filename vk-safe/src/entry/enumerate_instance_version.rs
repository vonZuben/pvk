use super::command_impl_prelude::*;

use std::mem::MaybeUninit;

use crate::pretty_version::VkVersion;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceVersion.html
*/
pub fn enumerate_instance_version() -> Result<VkVersion, vk::Result> {
    match super::entry_fn_loader::<vk::EnumerateInstanceVersion>() {
        Some(command) => {
            let mut version = MaybeUninit::uninit();
            unsafe {
                let res = command.get_fptr()(version.as_mut_ptr());
                check_raw_err!(res);
                Ok(VkVersion::from_raw(version.assume_init()))
            }
        }
        None => Ok(VkVersion::new(1, 0, 0)),
    }
}

const _VUIDS: () = {
    check_vuid_defs2!(EnumerateInstanceVersion
        pub const VUID_vkEnumerateInstanceVersion_pApiVersion_parameter: &'static [u8] =
            "pApiVersion must be a valid pointer to a uint32_t value".as_bytes();
            // using MaybeUninit::as_mut_ptr
    )
};
