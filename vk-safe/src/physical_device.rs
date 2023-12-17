use crate::scope::{RefScope, Scope};

use vk_safe_sys as vk;

use std::fmt;

use crate::array_storage::ArrayStorage;
use crate::instance::Instance;

pub struct PhysicalDevices<I: Instance, A: ArrayStorage<vk::PhysicalDevice>> {
    instance: I,
    handles: A::InitStorage,
}

impl<I: Instance, A: ArrayStorage<vk::PhysicalDevice>> PhysicalDevices<I, A> {
    pub(crate) fn new(handles: A::InitStorage, instance: I) -> Self {
        Self { instance, handles }
    }

    pub fn iter<'s>(&'s self) -> PhysicalDeviceIter<'s, I> {
        self.into_iter()
    }
}

/// A scoped PhysicalDevice
///
/// when you want to start using a PhysicalDevice, the PhysicalDevice defines a new scope
/// the PhysicalDevice new scope is itself limited with respect to the associated Instance scope
pub type ScopedPhysicalDeviceType<S, I> = RefScope<S, PhysicalDeviceType<I>>;

pub trait PhysicalDevice:
    std::ops::Deref<Target = ScopedPhysicalDeviceType<Self, Self::Instance>> + Copy
{
    type Instance: Instance;
}

impl<'scope, I: Instance> PhysicalDevice for Scope<'scope, PhysicalDeviceType<I>> {
    type Instance = I;
}

/// A PhysicalDevice handle that is limited to the scope of the associated Instance
pub struct PhysicalDeviceType<I: Instance> {
    instance: I,
    handle: vk::PhysicalDevice,
}

impl<I: Instance> PhysicalDeviceType<I> {
    pub(crate) fn new(instance: I, handle: vk::PhysicalDevice) -> Self {
        Self { instance, handle }
    }
}

impl<I: Instance> fmt::Debug for PhysicalDeviceType<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.handle.fmt(f)
    }
}

pub struct PhysicalDeviceIter<'s, I: Instance> {
    instance: I,
    iter: std::iter::Copied<std::slice::Iter<'s, vk::PhysicalDevice>>,
}

impl<I: Instance, S: ArrayStorage<vk::PhysicalDevice>> fmt::Debug for PhysicalDevices<I, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalDevices")?;
        f.debug_list()
            .entries(self.handles.as_ref().iter())
            .finish()
    }
}

impl<I: Instance> Iterator for PhysicalDeviceIter<'_, I> {
    type Item = PhysicalDeviceType<I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|pd| PhysicalDeviceType::new(self.instance, pd))
    }
}

impl<'s, I: Instance, S: ArrayStorage<vk::PhysicalDevice>> IntoIterator
    for &'s PhysicalDevices<I, S>
{
    type Item = PhysicalDeviceType<I>;

    type IntoIter = PhysicalDeviceIter<'s, I>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalDeviceIter {
            instance: self.instance,
            iter: self.handles.as_ref().into_iter().copied(),
        }
    }
}

mod create_device;
mod enumerate_device_extension_properties;
mod enumerate_device_layer_properties;
mod get_physical_device_features;
mod get_physical_device_format_properties;
mod get_physical_device_image_format_properties;
mod get_physical_device_memory_properties;
mod get_physical_device_properties;
mod get_physical_device_queue_family_properties;
mod get_physical_device_sparse_image_format_properties;

// use get_physical_device_features::*;
// use get_physical_device_format_properties::*;
pub use get_physical_device_image_format_properties::GetPhysicalDeviceImageFormatPropertiesParameters;
// use get_physical_device_properties::*;
use create_device::*;
pub use create_device::{DeviceCreateInfo, QueuePriorities};
pub use get_physical_device_memory_properties::*;
use get_physical_device_queue_family_properties::*;
