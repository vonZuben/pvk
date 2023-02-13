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
            vk::GetInstanceProcAddr(vk::Instance{handle: std::ptr::null()}, command_name)
        };

        Ok(Self {
            commands: V::EntryCommands::load(loader)?
        })
    }
}

// This is how each safe command can be implemented on top of each raw command
macro_rules! impl_safe_entry_interface {
    ( $interface:ident { $($code:tt)* }) => {
        impl<EntryVersion: VulkanVersion> safe_interface::$interface for Entry<EntryVersion> where EntryVersion::EntryCommands : GetCommand<vk::$interface> {
            $($code)*
        }
    };
}

macro_rules! result_getter {
    ( $fn_name:ident $(<$generic:ident>)? ( $($param:ident : $param_t:ty),* ) -> $getting:ty ) => {
        fn $fn_name(&self, $($param : $param_t ,)*) -> Result<$getting> {
            let mut get = MaybeUninit::uninit();
            unsafe {
                let res = self.commands.get()($($param.to_c(),)* None.to_c(), get.as_mut_ptr());
                check_raw_err!(res);
                Ok(get.assume_init())
            }
        }
    };
}

// enumerators are all very similar, so why repeat ourselves
macro_rules! enumerator_code {
    ( $fn_name:ident ( $($param:ident : $param_t:ty),* ) -> $getting:ty ) => {
        fn $fn_name<S: EnumeratorStorage<$getting>>(&self, $($param : $param_t ,)* mut storage: S) -> Result<S::InitStorage> {
            let query_len = || {
                let mut num = 0;
                let res;
                unsafe {
                    res = self.commands.get()($($param.to_c(),)* &mut num, std::ptr::null_mut());
                    check_raw_err!(res);
                }
                Ok(num.try_into().expect("error: vk_safe_interface internal error, can't convert len as usize"))
            };
            storage.query_len(query_len)?;
            let uninit_slice = storage.uninit_slice();
            let mut len = VulkanLenType::from_usize(uninit_slice.len());
            let res;
            unsafe {
                res = self.commands.get()($($param.to_c(),)* &mut len, uninit_slice.as_mut_ptr().cast());
                check_raw_err!(res);
            }
            Ok(storage.finalize(len.to_usize()))
        }
    };
}

impl_safe_entry_interface!{
CreateInstance {
    fn create_instance<V: VulkanVersion, E>(&self, create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>) -> Result<vk::Instance> {
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(&create_info.inner, None.to_c(), instance.as_mut_ptr());
            check_raw_err!(res);
            Ok(instance.assume_init())
        }
    }
}}

impl_safe_entry_interface!{
EnumerateInstanceExtensionProperties {
    enumerator_code!(enumerate_instance_extension_properties(layer_name: Option<&CStr>) -> ExtensionProperties);
}}

impl_safe_entry_interface!{
EnumerateInstanceLayerProperties {
    enumerator_code!(enumerate_instance_layer_properties() -> LayerProperties);
}}

impl_safe_entry_interface!{
EnumerateInstanceVersion {
    fn enumerate_instance_version(&self) -> Result<crate::utils::VkVersion> {
        let mut version = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(version.as_mut_ptr());
            check_raw_err!(res);
            Ok(crate::utils::VkVersion::from_raw(version.assume_init()))
        }
    }
}}