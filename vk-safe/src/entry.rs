use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::convert::TryInto;

use vk_safe_sys as vk;
use krs_hlist::Get;

use crate::instance as safe_instance;

use crate::safe_interface::{
    self,
    Result,
    structs::*,
    enumerator_storage::EnumeratorStorage,
    enumerator_storage::VulkanLenType,
    type_conversions::ToC,
    TempError,
};

use vk::{
    commands::{LoadCommands, CommandLoadError},
    VulkanVersion,
    VulkanExtension,
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

impl_safe_entry_interface!{
CreateInstance {
    fn create_instance<V: VulkanVersion, E: VulkanExtension>(&self, create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>) -> std::result::Result<safe_instance::Instance<V, E>, TempError>
    where V::InstanceCommands: LoadCommands, E::InstanceCommands: LoadCommands
    {
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(&create_info.inner, None.to_c(), instance.as_mut_ptr());
            // check_raw_err!(res);
            if res.is_err() {
                return Err(TempError);
            }
            Ok(safe_instance::Instance::new(instance.assume_init()).map_err(|_|TempError)?)
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