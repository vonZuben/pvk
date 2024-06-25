//! representation of a Vulkan implementation on the system
//!
//! A [`PhysicalDevice`] lets you query details about the Vulkan implementation (e.g.
//! memory properties). A logical [`Device`](crate::dispatchable_handles::device::Device)
//! can be created from a `PhysicalDevice` with [`create_device`].
//!
//! Vulkan doc:
//! <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkPhysicalDevice.html>

use vk_safe_sys as vk;

use crate::array_storage::ArrayStorage;
use crate::dispatchable_handles::instance::Instance;

use crate::scope::{Captures, Scope, Tag};

use super::ScopedDispatchableHandle;

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

use concrete_type::PhysicalDeviceConfig;

/// PhysicalDevice handle trait
///
/// Represents a *specific* PhysicalDevice which has been scoped.
///
/// See the available methods on [`_PhysicalDevice`]
///
/// Obtained by iterating over [`PhysicalDevices`], and then
/// tagging each PhysicalDevice with [`tag`](PhysicalDeviceTagger::tag).
///
/// You may note that there are no visible implementors of this trait.
/// You are only ever intended to use opaque implementors of this trait
/// as seen with the return type of [`tag`](PhysicalDeviceTagger::tag)
pub trait PhysicalDevice:
    ScopedDispatchableHandle<concrete_type::PhysicalDevice<Self::Config>> + Send + Sync
{
    #[doc(hidden)]
    type Config: PhysicalDeviceConfig<Context = Self::Context>;
    /// The *specific* Instance to which this PhysicalDevice belongs
    type Instance: Instance<Context = Self::Context>;
    /// shortcut to the Instance context such as the Version and Extensions being used
    type Context;
}

#[cfg(doc)]
/// Example of concrete PhysicalDevice
///
/// Given some <code>Pd: [PhysicalDevice]</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _PhysicalDevice<S, C> = crate::scope::SecretScope<S, concrete_type::PhysicalDevice<C>>;

pub struct PhysicalDevices<C: PhysicalDeviceConfig, A: ArrayStorage<vk::PhysicalDevice>> {
    config: C,
    handles: A::InitStorage,
}

unsafe impl<C: PhysicalDeviceConfig, A: ArrayStorage<vk::PhysicalDevice>> Send
    for PhysicalDevices<C, A>
{
}
unsafe impl<C: PhysicalDeviceConfig, A: ArrayStorage<vk::PhysicalDevice>> Sync
    for PhysicalDevices<C, A>
{
}

impl<C: PhysicalDeviceConfig, A: ArrayStorage<vk::PhysicalDevice>> PhysicalDevices<C, A> {
    pub(crate) fn new(handles: A::InitStorage, config: C) -> Self {
        Self { config, handles }
    }

    pub fn iter<'s>(&'s self) -> PhysicalDeviceIter<'s, C> {
        self.into_iter()
    }
}

pub struct PhysicalDeviceIter<'s, C: PhysicalDeviceConfig> {
    config: C,
    iter: std::iter::Copied<std::slice::Iter<'s, vk::PhysicalDevice>>,
}

impl<C: PhysicalDeviceConfig> Iterator for PhysicalDeviceIter<'_, C> {
    type Item = PhysicalDeviceTagger<C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|pd| PhysicalDeviceTagger::new(self.config, pd))
    }
}

/// Provides the means to add unique tags to PhysicalDevices
///
/// Obtained by iterating over the PhysicalDevices returned by
/// [`enumerate_physical_devices`](crate::scope::SecretScope::enumerate_physical_devices).
///
/// Provides the means to add unique tag to each individual PhysicalDevice with
/// the [`tag`](PhysicalDeviceTagger::tag) method. See documentation regarding [`Tag`] for
/// more details.
pub struct PhysicalDeviceTagger<C: PhysicalDeviceConfig> {
    config: C,
    handle: vk::PhysicalDevice,
}

unsafe impl<C: PhysicalDeviceConfig> Send for PhysicalDeviceTagger<C> {}
unsafe impl<C: PhysicalDeviceConfig> Sync for PhysicalDeviceTagger<C> {}

impl<C: PhysicalDeviceConfig> PhysicalDeviceTagger<C> {
    fn new(config: C, handle: vk::PhysicalDevice) -> Self {
        Self { config, handle }
    }

    /// Tag a Physical device
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
    pub fn tag<'t>(
        self,
        tag: Tag<'t>,
    ) -> impl PhysicalDevice<Context = C::Context> + Captures<Tag<'t>> {
        Scope::from_tag(
            concrete_type::PhysicalDevice::new(self.config, self.handle),
            tag,
        )
    }
}

impl<'s, C: PhysicalDeviceConfig, A: ArrayStorage<vk::PhysicalDevice>> IntoIterator
    for &'s PhysicalDevices<C, A>
{
    type Item = PhysicalDeviceTagger<C>;

    type IntoIter = PhysicalDeviceIter<'s, C>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalDeviceIter {
            config: self.config,
            iter: self.handles.as_ref().into_iter().copied(),
        }
    }
}

pub(crate) mod concrete_type {
    use crate::scope::{Scope, SecretScope};

    use vk_safe_sys as vk;

    use std::fmt;

    use crate::array_storage::ArrayStorage;
    use crate::dispatchable_handles::instance::Instance;

    pub type ScopedPhysicalDevice<S, C> = SecretScope<S, PhysicalDevice<C>>;

    impl<C: PhysicalDeviceConfig> super::PhysicalDevice for Scope<'_, PhysicalDevice<C>> {
        type Config = C;
        type Instance = C::Instance;
        type Context = <C::Instance as Instance>::Context;
    }

    pub trait PhysicalDeviceConfig: Copy {
        type Instance: Instance<Context = Self::Context>;
        type Context;
        fn instance(&self) -> &Self::Instance;
    }

    pub struct Config<'a, I> {
        instance: &'a I,
    }

    impl<'a, I> Config<'a, I> {
        pub(crate) fn new(instance: &'a I) -> Self {
            Self { instance }
        }
    }

    impl<I> Clone for Config<'_, I> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<I> Copy for Config<'_, I> {}

    impl<I: Instance> PhysicalDeviceConfig for Config<'_, I> {
        type Instance = I;
        type Context = I::Context;

        fn instance(&self) -> &Self::Instance {
            &self.instance
        }
    }

    /// A PhysicalDevice handle that is limited to the scope of the associated Instance
    pub struct PhysicalDevice<C: PhysicalDeviceConfig> {
        config: C,
        pub(crate) handle: vk::PhysicalDevice,
    }

    unsafe impl<C: PhysicalDeviceConfig> Send for PhysicalDevice<C> {}
    unsafe impl<C: PhysicalDeviceConfig> Sync for PhysicalDevice<C> {}

    impl<C: PhysicalDeviceConfig> PhysicalDevice<C> {
        pub(crate) fn new(config: C, handle: vk::PhysicalDevice) -> Self {
            Self { config, handle }
        }

        pub(crate) fn instance(&self) -> &C::Instance {
            self.config.instance()
        }
    }

    impl<C: PhysicalDeviceConfig> fmt::Debug for PhysicalDevice<C> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.handle.fmt(f)
        }
    }

    impl<C: PhysicalDeviceConfig, A: ArrayStorage<vk::PhysicalDevice>> fmt::Debug
        for super::PhysicalDevices<C, A>
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PhysicalDevices")?;
            f.debug_list()
                .entries(self.handles.as_ref().iter())
                .finish()
        }
    }
}
