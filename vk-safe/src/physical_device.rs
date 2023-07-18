use crate::scope::*;

use vk_safe_sys as vk;

use std::fmt;

use crate::enumerator_storage::EnumeratorStorage;
use crate::instance::{InstanceConfig, ScopedInstance};

pub struct PhysicalDevices<'i, C: InstanceConfig, S: EnumeratorStorage<vk::PhysicalDevice>> {
    instance: ScopedInstance<'i, C>,
    handles: S::InitStorage,
}

impl<'i, C: InstanceConfig, S: EnumeratorStorage<vk::PhysicalDevice>> PhysicalDevices<'i, C, S> {
    pub(crate) fn new(handles: S::InitStorage, instance: ScopedInstance<'i, C>) -> Self {
        Self { instance, handles }
    }

    pub fn iter<'s>(
        &'s self,
    ) -> PhysicalDeviceIter<'i, 's, C>
    {
        self.into_iter()
    }
}

/// A scoped PhysicalDevice
///
/// when you want to start using a PhysicalDevice, the PhysicalDevice defines a new scope
/// the PhysicalDevice new scope is itself limited with respect to the associated Instance scope
pub type ScopedPhysicalDevice<'pd, 'i, C> = Scope<'pd, &'pd PhysicalDevice<'i, C>>;

/// A PhysicalDevice handle that is limited to the scope of the associated Instance
pub struct PhysicalDevice<'i, C: InstanceConfig> {
    instance: ScopedInstance<'i, C>,
    handle: vk::PhysicalDevice,
}

impl<'i, C: InstanceConfig> PhysicalDevice<'i, C> {
    pub(crate) fn new(instance: ScopedInstance<'i, C>, handle: vk::PhysicalDevice) -> Self {
        Self { instance, handle }
    }
}

impl<C: InstanceConfig> fmt::Debug for PhysicalDevice<'_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(f)
    }
}

pub struct PhysicalDeviceIter<'i, 's, C: InstanceConfig> {
    instance: ScopedInstance<'i, C>,
    iter: std::iter::Copied<std::slice::Iter<'s, vk::PhysicalDevice>>,
}

impl<C: InstanceConfig, S: EnumeratorStorage<vk::PhysicalDevice>> fmt::Debug
    for PhysicalDevices<'_, C, S>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalDevices")?;
        f.debug_list()
            .entries(self.handles.as_ref().iter())
            .finish()
    }
}

impl<'i, C: InstanceConfig> Iterator
    for PhysicalDeviceIter<'i, '_, C>
{
    type Item = PhysicalDevice<'i, C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|pd| {
            PhysicalDevice::new(self.instance, pd)
        })
    }
}

impl<'s, 'i, C: InstanceConfig, S: EnumeratorStorage<vk::PhysicalDevice>> IntoIterator
    for &'s PhysicalDevices<'i, C, S>
{
    type Item = PhysicalDevice<'i, C>;

    type IntoIter =
        PhysicalDeviceIter<'i, 's, C>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalDeviceIter {
            instance: self.instance,
            iter: self.handles.as_ref().into_iter().copied(),
        }
    }
}

mod get_physical_device_features;
mod get_physical_device_format_properties;
mod get_physical_device_image_format_properties;
mod get_physical_device_properties;
mod get_physical_device_queue_family_properties;
mod get_physical_device_memory_properties;
mod create_device;

// use get_physical_device_features::*;
// use get_physical_device_format_properties::*;
// use get_physical_device_image_format_properties::*;
// use get_physical_device_properties::*;
use get_physical_device_queue_family_properties::*;
// use get_physical_device_memory_properties::*;
use create_device::*;
pub use create_device::{DeviceCreateInfo, QueuePriorities};