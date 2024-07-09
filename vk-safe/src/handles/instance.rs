use super::physical_device::PhysicalDevices;
use super::DispatchableHandle;

use crate::array_storage::ArrayStorage;
use crate::error::Error;

use vk_safe_sys as vk;

pub_export_modules2!(
#[cfg(VK_VERSION_1_0)]
enumerate_physical_devices;
);

/// Main Vulkan object
///
/// [`Instance`] is the main object you create ([`create_instance`](crate::vk::create_instance))
/// in Vulkan that stores all application state. The primary thing you will want to do with
/// an Instance is enumerate the PhysicalDevices on the system ([`Instance::enumerate_physical_devices`])
///
/// Vulkan doc:
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkInstance.html>
pub trait Instance: DispatchableHandle<RawHandle = vk::Instance> + Sized {
    #[cfg(VK_VERSION_1_0)]
    /// Enumerate PhysicalDevices on the system
    ///
    /// # Usage
    /// Provide an [`ArrayStorage`] implementor to store the PhysicalDevices.
    /// Then you can iterate over the PhysicalDevices and tag each one that
    /// you want to use with a [`Tag`](crate::scope::Tag).
    ///
    /// # Example
    /// ```
    /// # use vk_safe::vk;
    /// # fn tst(instance: impl vk::Instance<Context: vk::instance::VERSION_1_0>) {
    /// let physical_devices = instance
    ///     .enumerate_physical_devices(Vec::new())
    ///     .unwrap();
    ///
    /// for physical_device in physical_devices.iter() {
    ///     vk::tag!(tag);
    ///     let physical_device = physical_device.tag(tag);
    /// }
    /// # }
    /// ```
    ///
    /// Vulkan docs:
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumeratePhysicalDevices.html>
    fn enumerate_physical_devices<A: ArrayStorage<vk::PhysicalDevice>>(
        &self,
        storage: A,
    ) -> Result<impl PhysicalDevices<Self>, Error>
    where
        Self::Commands: vk::has_command::EnumeratePhysicalDevices,
    {
        enumerate_physical_devices::enumerate_physical_devices(self, storage)
    }
}
