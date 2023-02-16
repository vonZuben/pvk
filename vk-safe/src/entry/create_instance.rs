use super::command_impl_prelude::*;

use crate::instance as safe_instance;
use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct TempError;

pub trait CreateInstance {
    fn create_instance<V: vk::VulkanVersion, E: vk::VulkanExtension>(
        &self,
        create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>,
    ) -> std::result::Result<safe_instance::Instance<V, E>, TempError>
    where
        V::InstanceCommands: vk::commands::LoadCommands,
        E::InstanceCommands: vk::commands::LoadCommands;
}

impl_safe_entry_interface! {
CreateInstance {
    fn create_instance<V: vk::VulkanVersion, E: vk::VulkanExtension>(
        &self,
        create_info: &crate::safe_interface::structs::InstanceCreateInfo<V, E>,
    ) -> std::result::Result<safe_instance::Instance<V, E>, TempError>
    where
        V::InstanceCommands: vk::commands::LoadCommands,
        E::InstanceCommands: vk::commands::LoadCommands
    {
        let mut instance = MaybeUninit::uninit();
        unsafe {
            let res = self.commands.get()(&create_info.inner, None.to_c(), instance.as_mut_ptr());
            if res.is_err() {
                return Err(TempError);
            }
            Ok(safe_instance::Instance::new(instance.assume_init()).map_err(|_|TempError)?)
        }
    }
}}
