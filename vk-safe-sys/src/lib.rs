#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused)]
#![recursion_limit = "1000"]

// trace_macros!(true);

#[macro_use]
mod generated_vulkan; // TODO I do not think I want this public, but need type defs for now

#[link(name = "vulkan")]
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

pub mod commands;

pub use generated_vulkan::command as has_command;
pub use generated_vulkan::*;
