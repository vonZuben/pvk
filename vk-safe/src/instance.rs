use crate::safe_interface::type_conversions::ToC;
use krs_hlist::Get;
use vk_safe_sys as vk;

use crate::pretty_version::VkVersion;

use std::marker::PhantomData;

use vk::commands::{CommandLoadError, LoadCommands};

pub trait InstanceConfig {
    const VERSION: VkVersion;
    type InstanceCommands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyInstance>;
    type InstanceExtensions : vk::commands::LoadCommands;
}

#[derive(Debug)]
pub struct Config<V, E> {
    _version: PhantomData<V>,
    _extension: PhantomData<E>,
}

impl<V, E> Config<V, E> {
    pub fn new(_version: V, _extensions: E) -> Self {
        Self { _version: PhantomData, _extension: PhantomData }
    }
}

impl<V: vk::VulkanVersion, E: vk::VulkanExtension> InstanceConfig for Config<V, E>
where
    V::InstanceCommands : vk::commands::LoadCommands + vk::GetCommand<vk::DestroyInstance>,
    E::InstanceCommands : vk::commands::LoadCommands,
{
    const VERSION: VkVersion = VkVersion::new(V::VersionTriple.0, V::VersionTriple.1, V::VersionTriple.2);

    type InstanceCommands = V::InstanceCommands;

    type InstanceExtensions = E::InstanceCommands;
}

#[derive(Debug)]
pub struct Instance<C: InstanceConfig> {
    handle: vk::Instance,
    feature_commands: C::InstanceCommands,
    extension_commands: C::InstanceExtensions,
}

impl<C: InstanceConfig> Instance<C> {
    pub(crate) fn new(handle: vk::Instance) -> Result<Self, CommandLoadError> {
        let loader = |command_name| unsafe { vk::GetInstanceProcAddr(handle, command_name) };
        Ok(Self {
            handle,
            feature_commands: C::InstanceCommands::load(loader)?,
            extension_commands: C::InstanceExtensions::load(loader)?,
        })
    }
}

/*
SAFETY (https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkDestroyInstance.html)

VUID-vkDestroyInstance-instance-00629
All child objects created using instance must have been destroyed prior to destroying instance

- child objects borrow the Instance, so they should all be dropped (destroyed) before it is possible to Drop the Instance

VUID-vkDestroyInstance-instance-00630
If VkAllocationCallbacks were provided when instance was created, a compatible set of callbacks must be provided here

- the allocation callbacks from creation should be held by Instance, and used here

VUID-vkDestroyInstance-instance-00631
If no VkAllocationCallbacks were provided when instance was created, pAllocator must be NULL

- this follows from holding the allocation callbacks from creation

VUID-vkDestroyInstance-instance-parameter
If instance is not NULL, instance must be a valid VkInstance handle

- taken by rust ref so valid, and creation of all safe interface types should only make valid types

VUID-vkDestroyInstance-pAllocator-parameter
If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks structure

- taken by rust ref so valid
*/
impl<C: InstanceConfig> Drop for Instance<C> {
    fn drop(&mut self) {
        unsafe { self.feature_commands.get()(self.handle, None.to_c()) }
    }
}

mod command_impl_prelude {
    pub use super::Instance;
    pub use crate::enumerator_storage::{EnumeratorStorage, VulkanLenType};
    pub use crate::safe_interface::type_conversions::*;
    pub use krs_hlist::Get;
    pub use vk_safe_sys as vk;
    pub use vk_safe_sys::{GetCommand, VulkanExtension, VulkanVersion};
}

// This is how each safe command can be implemented on top of each raw command
macro_rules! impl_safe_instance_interface {
    ( $interface:ident { $($code:tt)* }) => {
        impl<C: InstanceConfig> Instance<C>
        where
            C::InstanceCommands: GetCommand<vk::$interface> {
            $($code)*
        }
    };
}

mod enumerate_physical_devices;

pub use enumerate_physical_devices::*;