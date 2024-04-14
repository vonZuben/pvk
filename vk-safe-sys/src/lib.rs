#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused)]
#![recursion_limit = "1000"]

//! Vulkan bindings for rust
//!
//! generated using the generator crate

#[macro_use]
pub mod generated_vulkan;

#[cfg_attr(target_os = "linux", link(name = "vulkan"))]
#[cfg_attr(target_os = "windows", link(name = "vulkan-1"))]
extern "system" {
    #[link_name = "vkGetInstanceProcAddr"]
    pub fn GetInstanceProcAddr(
        instance: generated_vulkan::Instance,
        p_name: *const std::ffi::c_char,
    ) -> Option<generated_vulkan::PFN_vkVoidFunction>;

    #[link_name = "vkGetDeviceProcAddr"]
    pub fn GetDeviceProcAddr(
        instance: generated_vulkan::Device,
        p_name: *const std::ffi::c_char,
    ) -> Option<generated_vulkan::PFN_vkVoidFunction>;
}

pub mod context;

pub use generated_vulkan::has_command;
pub use generated_vulkan::*;
