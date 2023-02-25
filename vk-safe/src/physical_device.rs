use vk_safe_sys as vk;

use std::fmt;

use crate::enumerator_storage::EnumeratorStorage;
use crate::instance::{Instance, InstanceConfig};

pub struct PhysicalDevices<'i, C: InstanceConfig, S: EnumeratorStorage<vk::PhysicalDevice>> {
    instance: &'i Instance<C>,
    handles: S::InitStorage,
}

impl<'i, C: InstanceConfig, S: EnumeratorStorage<vk::PhysicalDevice>> PhysicalDevices<'i, C, S> {
    pub fn new(handles: S::InitStorage, instance: &'i Instance<C>) -> Self {
        Self { instance, handles }
    }

    pub fn iter<'s>(
        &'s self,
    ) -> PhysicalDeviceIter<'i, 's, C>
    {
        self.into_iter()
    }
}

pub struct PhysicalDevice<'i, C: InstanceConfig> {
    instance: &'i Instance<C>,
    handle: vk::PhysicalDevice,
}

impl<C: InstanceConfig> fmt::Debug for PhysicalDevice<'_, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(f)
    }
}

pub struct PhysicalDeviceIter<'i, 's, C: InstanceConfig> {
    instance: &'i Instance<C>,
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
        self.iter.next().map(|pd| PhysicalDevice {
            instance: self.instance,
            handle: pd,
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
