use super::command_impl_prelude::*;

use std::mem::MaybeUninit;

use crate::pretty_version::VkVersion;

/*
https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceVersion.html
*/
impl_safe_entry_interface! {
EnumerateInstanceVersion {
    pub fn enumerate_instance_version(&self) -> Result<VkVersion, vk::Result> {
        let mut version = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get().get_fptr()(version.as_mut_ptr());
            check_raw_err!(res);
            Ok(VkVersion::from_raw(version.assume_init()))
        }
    }
}}

mod enumerate_instance_version_validation {
    use vk_safe_sys::validation::EnumerateInstanceVersion::*;

    pub struct Validation;

    #[allow(non_upper_case_globals)]
    impl Vuids for Validation {
        const VUID_vkEnumerateInstanceVersion_pApiVersion_parameter: () = {
            // using MaybeUninit::as_mut_ptr
        };
    }

    check_vuid_defs!(
        pub const VUID_vkEnumerateInstanceVersion_pApiVersion_parameter: &'static [u8] =
            "pApiVersion must be a valid pointer to a uint32_t value".as_bytes();
    );
}