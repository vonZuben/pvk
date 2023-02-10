use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::convert::TryInto;

use vk_safe_sys as vk;
use krs_hlist::Get;

use crate::safe_interface::{
    self,
    Result,
    structs::*,
    enumerator_storage::EnumeratorStorage,
    enumerator_storage::VulkanLenType,
    type_conversions::ToC
};

use vk::{
    commands::{LoadCommands, CommandLoadError},
    VulkanVersion,
    GetCommand,
};

#[link(name = "vulkan")]
extern "system" {
    /// This is the very first point of entry that is internally used to load all other functions
    #[link_name = "vkGetInstanceProcAddr"]
    fn GetInstanceProcAddr(instance: vk::Instance, p_name: *const std::os::raw::c_char)
             -> Option<vk::PFN_vkVoidFunction>;
}

/// Entry
///
/// provides a means for accessing global vulkan commands
#[derive(Debug)]
pub struct Entry<V: VulkanVersion> {
    commands: V::EntryCommands,
}

impl<V: VulkanVersion> Entry<V> {
    pub fn from_version(_v: V) -> std::result::Result<Self, CommandLoadError> where V::EntryCommands: LoadCommands {

        let loader = |command_name| unsafe {
            GetInstanceProcAddr(vk::Instance{handle: std::ptr::null()}, command_name)
        };

        Ok(Self {
            commands: V::EntryCommands::load(loader)?
        })
    }
}

// This is how each safe command can be implemented on top of each raw command
macro_rules! impl_safe_entry_interface {
    ( $interface:ident { $($code:tt)* }) => {
        impl<V: VulkanVersion> safe_interface::$interface for Entry<V> where V::EntryCommands : GetCommand<vk::$interface> {
            $($code)*
        }
    };
}

// enumerators are all very similar, so why repeat ourselves
// macro_rules! enumerator_code {
//     (
//         $len_name:ident,
//         $main_name:ident,
//         $getting:ty =>
//         ( $($param:ident : $param_t:ty),* )
//     ) => {
//         fn $len_name(&self, $($param : $param_t),*) -> Result<usize> {
//             let mut num = 0;
//             let res;
//             unsafe {
//                 res = self.commands.fptr()($($param.to_c(),)* &mut num, std::ptr::null_mut());
//                 check_raw_err!(res);
//             }
//             Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
//         }
//         fn $main_name<S: EnumeratorStorage<$getting>>(&self, $($param : $param_t ,)* mut storage: S) -> Result<S::InitStorage> {
//             let query_len = || self.$len_name($($param,)*);
//             storage.query_len(query_len)?;
//             let uninit_slice = storage.uninit_slice();
//             let mut len = VulkanLenType::from_usize(uninit_slice.len());
//             let res;
//             unsafe {
//                 res = self.commands.fptr()($($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
//                 check_raw_err!(res);
//             }
//             Ok(storage.finalize(len.to_usize()))
//         }
//     };
// }

impl_safe_entry_interface!{
CreateInstance {
    fn create_instance(&self, create_info: &vk::InstanceCreateInfo) -> Result<vk::Instance> {
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(create_info, None.to_c(), instance.as_mut_ptr());
            check_raw_err!(res);
            Ok(instance.assume_init())
        }
    }
}}

// impl<V: VulkanVersion> safe_interface::CreateInstance for Entry<V> where V::EntryCommands : GetCommand<vk::CreateInstance> {
//     fn create_instance(&self, create_info: &vk::InstanceCreateInfo) -> Result<vk::Instance> {
//         let mut instance = MaybeUninit::uninit();
//         unsafe {
//             let res = self.commands.get()(create_info, None.to_c(), instance.as_mut_ptr());
//             check_raw_err!(res);
//             Ok(instance.assume_init())
//         }
//     }
// }

// impl_safe_entry_interface!{
// EnumerateInstanceExtensionProperties {
//     enumerator_code!(
//         enumerate_instance_extension_properties_len,
//         enumerate_instance_extension_properties,
//         ExtensionProperties =>
//         (layer_name: Option<&CStr>)
//     );
// }}

// impl_safe_entry_interface!{
// EnumerateInstanceLayerProperties {
//     enumerator_code!(
//         enumerate_instance_layer_properties_len,
//         enumerate_instance_layer_properties,
//         LayerProperties =>
//         ()
//     );
// }}

// impl_safe_entry_interface!{
// EnumerateInstanceVersion {
//     fn enumerate_instance_version(&self) -> Result<crate::utils::VkVersion> {
//         let mut version = MaybeUninit::uninit();
//         unsafe {
//             let res = self.commands.fptr()(version.as_mut_ptr());
//             check_raw_err!(res);
//             Ok(crate::utils::VkVersion::from_raw(version.assume_init()))
//         }
//     }
// }}