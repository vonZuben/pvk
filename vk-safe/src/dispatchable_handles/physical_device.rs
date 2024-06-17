//! representation of a Vulkan implementation on the system
//!
//! A `PhysicalDevice` lets you query details about the Vulkan implementation (e.g.
//! memory properties). A logical [`Device`](crate::dispatchable_handles::device::ConcreteDevice)
//! can be created from a `PhysicalDevice` with [`create_device`].
//!
//! Vulkan doc:
//! <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDevice.html>

use vk_safe_sys as vk;

use crate::array_storage::ArrayStorage;
use crate::dispatchable_handles::instance::Instance;

// pub mod enumerate_device_extension_properties;
// pub mod enumerate_device_layer_properties;
// pub mod get_physical_device_features;
// pub mod get_physical_device_format_properties;
// pub mod get_physical_device_image_format_properties;
// pub mod get_physical_device_memory_properties;
// pub mod get_physical_device_properties;
// pub mod get_physical_device_queue_family_properties;
// pub mod get_physical_device_sparse_image_format_properties;

pub_export_modules!(create_device;
enumerate_device_extension_properties;
enumerate_device_layer_properties;
get_physical_device_features;
get_physical_device_format_properties;
get_physical_device_image_format_properties;
get_physical_device_memory_properties;
get_physical_device_properties;
get_physical_device_queue_family_properties;
get_physical_device_sparse_image_format_properties;
);

use crate::scope::{HandleScope, Shared};

/** PhysicalDevice handle trait

Represents a *specific* PhysicalDevice which has been scoped.
*/
pub trait PhysicalDevice: HandleScope<concrete_type::PhysicalDevice<Self::Instance>> {
    /// The *specific* Instance to which this PhysicalDevice belongs
    type Instance: Instance<Context = Self::Context>;
    /// shortcut to the Instance context such as the Version and Extensions being used
    type Context;
}

pub use concrete_type::PhysicalDevice as ConcretePhysicalDevice;

pub struct PhysicalDevices<I: Instance, A: ArrayStorage<vk::PhysicalDevice>> {
    instance: Shared<I>,
    handles: A::InitStorage,
}

unsafe impl<I: Instance, A: ArrayStorage<vk::PhysicalDevice>> Send for PhysicalDevices<I, A> {}
unsafe impl<I: Instance, A: ArrayStorage<vk::PhysicalDevice>> Sync for PhysicalDevices<I, A> {}

impl<I: Instance, A: ArrayStorage<vk::PhysicalDevice>> PhysicalDevices<I, A> {
    pub(crate) fn new(handles: A::InitStorage, instance: Shared<I>) -> Self {
        Self { instance, handles }
    }

    pub fn iter<'s>(&'s self) -> PhysicalDeviceIter<'s, I> {
        self.into_iter()
    }
}

pub struct PhysicalDeviceIter<'s, I: Instance> {
    instance: Shared<I>,
    iter: std::iter::Copied<std::slice::Iter<'s, vk::PhysicalDevice>>,
}

impl<I: Instance> Iterator for PhysicalDeviceIter<'_, I> {
    type Item = concrete_type::PhysicalDevice<I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|pd| concrete_type::PhysicalDevice::new(self.instance, pd))
    }
}

impl<'s, I: Instance, S: ArrayStorage<vk::PhysicalDevice>> IntoIterator
    for &'s PhysicalDevices<I, S>
{
    type Item = concrete_type::PhysicalDevice<I>;

    type IntoIter = PhysicalDeviceIter<'s, I>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalDeviceIter {
            instance: self.instance,
            iter: self.handles.as_ref().into_iter().copied(),
        }
    }
}

pub(crate) mod concrete_type {
    use crate::scope::{Scope, SecretScope, Shared};

    use vk_safe_sys as vk;

    use std::fmt;

    use crate::array_storage::ArrayStorage;
    use crate::dispatchable_handles::instance::Instance;

    pub type ScopedPhysicalDevice<S, I> = SecretScope<S, PhysicalDevice<I>>;

    impl<I: Instance> super::PhysicalDevice for Scope<'_, PhysicalDevice<I>> {
        type Instance = I;
        type Context = I::Context;
    }

    /// A PhysicalDevice handle that is limited to the scope of the associated Instance
    pub struct PhysicalDevice<I: Instance> {
        pub(crate) instance: Shared<I>,
        pub(crate) handle: vk::PhysicalDevice,
    }

    unsafe impl<I: Instance> Send for PhysicalDevice<I> {}
    unsafe impl<I: Instance> Sync for PhysicalDevice<I> {}

    impl<I: Instance> PhysicalDevice<I> {
        pub(crate) fn new(instance: Shared<I>, handle: vk::PhysicalDevice) -> Self {
            Self { instance, handle }
        }
    }

    impl<I: Instance> fmt::Debug for PhysicalDevice<I> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.handle.fmt(f)
        }
    }

    impl<I: Instance, S: ArrayStorage<vk::PhysicalDevice>> fmt::Debug for super::PhysicalDevices<I, S> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PhysicalDevices")?;
            f.debug_list()
                .entries(self.handles.as_ref().iter())
                .finish()
        }
    }
}
