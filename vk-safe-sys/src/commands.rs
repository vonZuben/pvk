use std::ffi::c_char;
use std::fmt;

use krs_hlist::{Cons, End, Hlist};
use crate::generated::VulkanCommand;

/// local type alias for vulkan void function
pub type VkVoidFunction = crate::generated::PFN_vkVoidFunction;

/// "trait alias" for a function that can load a vulkan command
pub trait FunctionLoader:
    Fn(*const c_char) -> Option<VkVoidFunction> + Copy
{
}
impl<F> FunctionLoader for F where
    F: Fn(*const c_char) -> Option<VkVoidFunction> + Copy
{
}

/// Error loading a command
///
/// ## Safety
/// 'command' must be set to a valid c string pointer
/// there is no check for this
#[derive(Debug)]
pub struct CommandLoadError {
    command: *const c_char,
}

impl std::error::Error for CommandLoadError {}

impl fmt::Display for CommandLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY : CommandLoadError can only be internally created, and we must ensure it is created with a pointer to a valid c string
        let command_name = unsafe { std::ffi::CStr::from_ptr(self.command) };
        write!(f, "failed to load {:?}", command_name)
    }
}

/// Load commands with a given function loader
///
/// 'loader' is an function that takes a c_string pointer to the name of the command to load
pub trait LoadCommands : Sized {
    fn load(loader: impl FunctionLoader) -> Result<Self, CommandLoadError>;
}

impl<C: VulkanCommand> LoadCommands for C {
    fn load(loader: impl FunctionLoader) -> Result<Self, CommandLoadError> {
        let fptr = loader(C::VK_NAME).ok_or(CommandLoadError { command: C::VK_NAME })?;
        // SAFETY : fptr should be the correct kind of pointer since we loaded it with H::VK_NAME
        unsafe { Ok(C::new(fptr)) }
    }
}

impl<H: LoadCommands, T: LoadCommands + Hlist> LoadCommands for Cons<H, T> {
    fn load(loader: impl FunctionLoader) -> Result<Self, CommandLoadError> {
       Ok(Cons::new(H::load(loader)?, T::load(loader)?))
    }
}

impl LoadCommands for End {
    fn load(loader: impl FunctionLoader) -> Result<Self, CommandLoadError> {
        Ok(End)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn command_load_test() {
        let mut instance = crate::generated::Instance { handle: std::ptr::null() };
        let loader = |name| {
            // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
            unsafe { crate::GetInstanceProcAddr(instance, name) }
        };

        // test command list with only CreateInstance since we can load it without an instance
        type ToLoad = krs_hlist::hlist_ty!(crate::generated::CreateInstance);

        let instance_commands = ToLoad::load(loader).unwrap();

        let mut info = unsafe { std::mem::MaybeUninit::<crate::generated::InstanceCreateInfo>::zeroed().assume_init() };
        info.s_type = crate::generated::StructureType::INSTANCE_CREATE_INFO;
        // info.p_application_info = &app_info;

        unsafe { (&instance_commands.head)(&info, std::ptr::null(), &mut instance) };

        println!("{:?}", instance);

        let loader = |name| {
            // SAFETY : this will only be used here where we trust the passed name is a proper c_string command name
            unsafe { crate::GetInstanceProcAddr(instance, name) }
        };

        let commands = <crate::generated::VERSION_1_0 as crate::generated::VulkanVersion>::InstanceCommands::load(loader).unwrap();

        println!("{commands:?}");
    }
}