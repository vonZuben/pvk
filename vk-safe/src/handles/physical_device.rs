use super::instance::Instance;
use super::{DispatchableHandle, Handle};

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::scope::{Captures, Tag};
use crate::structs::*;
use crate::vk_str::VkStr;

use std::marker::PhantomData;

use vk_safe_sys as vk;

pub_export_modules2!(
#[cfg(VK_VERSION_1_0)]
get_physical_device_properties;

#[cfg(VK_VERSION_1_0)]
get_physical_device_features;

#[cfg(VK_VERSION_1_0)]
enumerate_device_extension_properties;

#[cfg(VK_VERSION_1_0)]
enumerate_device_layer_properties;

#[cfg(VK_VERSION_1_0)]
get_physical_device_format_properties;
);

/// PhysicalDevice handle trait
///
/// Represents a *specific* PhysicalDevice which has been scoped.
///
/// Obtained by iterating over [`PhysicalDevices`], and then
/// tagging each PhysicalDevice with [`tag`](PhysicalDeviceTagger::tag).
///
/// You may note that there are no visible implementors of this trait.
/// You are only ever intended to use opaque implementors of this trait
/// as seen with the return type of [`tag`](PhysicalDeviceTagger::tag)
pub trait PhysicalDevice: DispatchableHandle<RawHandle = vk::PhysicalDevice> + Sized {
    type Instance: Instance;

    /// Query the properties of the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    /// #   (physical_device: P) {
    /// let physical_device_properties = physical_device.get_physical_device_properties();
    /// # }
    /// ```
    ///
    /// Vulkan docs:
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetPhysicalDeviceProperties.html>
    fn get_physical_device_properties(&self) -> PhysicalDeviceProperties<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceProperties,
    {
        get_physical_device_properties(self)
    }

    /// Query the device level layers supported by the PhysicalDevice
    ///
    /// Must provide [`ArrayStorage`] space to return the extension properties into.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    /// #   (physical_device: P) {
    /// let layer_properties = physical_device.enumerate_device_layer_properties(Vec::new());
    /// # }
    /// ```
    fn enumerate_device_layer_properties<A: ArrayStorage<LayerProperties<Self>>>(
        &self,
        storage: A,
    ) -> Result<A::InitStorage, Error>
    where
        Self::Commands: vk::has_command::EnumerateDeviceLayerProperties,
    {
        enumerate_device_layer_properties(self, storage)
    }

    /// Query the features supported by the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    /// #   (physical_device: P) {
    /// let features = physical_device.get_physical_device_features();
    /// # }
    /// ```
    fn get_physical_device_features(&self) -> PhysicalDeviceFeatures<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceFeatures,
    {
        get_physical_device_features(self)
    }

    /// Query the device level extensions supported by the PhysicalDevice
    ///
    /// If `layer_name` is `None`, only extensions provided by the Vulkan implementation.
    /// are returned. If `layer_name` is `Some(layer_name)`, device extensions provided
    /// by that layer are returned.
    ///
    /// Must provide [`ArrayStorage`] space to return the extension properties into.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    /// #   (physical_device: P) {
    /// let extension_properties =
    ///     physical_device.enumerate_device_extension_properties(None, Vec::new());
    /// # }
    /// ```
    fn enumerate_device_extension_properties<A: ArrayStorage<ExtensionProperties<Self>>>(
        &self,
        layer_name: Option<VkStr>,
        storage: A,
    ) -> Result<A::InitStorage, Error>
    where
        Self::Commands: vk::has_command::EnumerateDeviceExtensionProperties,
    {
        enumerate_device_extension_properties(self, layer_name, storage)
    }

    /// Query the format properties of the PhysicalDevice
    ///
    /// Provide the [`Format`](crate::vk::Format) to get the properties of that format
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # fn tst<C: vk::instance::VERSION_1_0, P: vk::PhysicalDevice<Context = C>>
    /// #   (physical_device: P) {
    /// let format_properties =
    ///     physical_device.get_physical_device_format_properties(vk::Format::R8G8B8A8_SRGB);
    /// # }
    /// ```
    fn get_physical_device_format_properties(&self, format: vk::Format) -> FormatProperties<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceFormatProperties,
    {
        get_physical_device_format_properties(self, format)
    }
}

/// Hidden type which implements PhysicalDevice
struct _PhysicalDevice<'a, I, T> {
    handle: vk::PhysicalDevice,
    instance: &'a I,
    tag: PhantomData<T>,
}

impl<'a, I, T> _PhysicalDevice<'a, I, T> {
    fn new(handle: vk::PhysicalDevice, instance: &'a I, _tag: T) -> Self {
        Self {
            handle,
            instance,
            tag: PhantomData,
        }
    }
}

impl<I, T> Handle for _PhysicalDevice<'_, I, T> {
    type RawHandle = vk::PhysicalDevice;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<I: Instance, T> DispatchableHandle for _PhysicalDevice<'_, I, T> {
    type Commands = I::Commands;

    fn commands(&self) -> &Self::Commands {
        self.instance.commands()
    }
}

impl<I: Instance, T> PhysicalDevice for _PhysicalDevice<'_, I, T> {
    type Instance = I;
}

pub(crate) fn make_physical_devices<'a, I: Instance, A: AsRef<[vk::PhysicalDevice]>>(
    instance: &'a I,
    array: A,
) -> impl PhysicalDevices<I> + Captures<&'a I> {
    _PhysicalDevices { instance, array }
}

/// Hidden type which implements PhysicalDeviceTagger
struct _PhysicalDeviceTagger<'a, I> {
    instance: &'a I,
    physical_device: vk::PhysicalDevice,
}

unsafe impl<I: Send> Send for _PhysicalDeviceTagger<'_, I> {}
unsafe impl<I: Sync> Sync for _PhysicalDeviceTagger<'_, I> {}

impl<'a, I> _PhysicalDeviceTagger<'a, I> {
    fn new(instance: &'a I, physical_device: vk::PhysicalDevice) -> Self {
        Self {
            instance,
            physical_device,
        }
    }
}

impl<'a, I: Instance> PhysicalDeviceTagger<I> for _PhysicalDeviceTagger<'a, I> {
    fn tag<'t>(self, tag: Tag<'t>) -> impl PhysicalDevice<Instance = I> {
        _PhysicalDevice::new(self.physical_device, self.instance, tag)
    }
}

/// Provides the means to add unique tags to PhysicalDevices
///
/// Provides the means to add unique tag to each individual PhysicalDevice with
/// the [`tag`](PhysicalDeviceTagger::tag) method. See documentation regarding [`Tag`] for
/// more details.
///
/// Obtained by iterating over the PhysicalDevices returned by
/// [`enumerate_physical_devices`](crate::scope::SecretScope::enumerate_physical_devices).
pub trait PhysicalDeviceTagger<I: Instance>: Sized {
    /// Tag an enumerated PhysicalDevice
    ///
    /// See [`Instance::enumerate_physical_devices`] for
    /// example use.
    fn tag<'t>(self, tag: Tag<'t>) -> impl PhysicalDevice<Instance = I>;
}

/// Provide access to PhysicalDevices enumerated on the system
///
/// Can be consumed via [`IntoIterator`] implementation, or
/// you can iterator without consuming with [`PhysicalDevices::iter`].
pub trait PhysicalDevices<I: Instance>: IntoIterator<Item = Self::Tagger> {
    type Tagger: PhysicalDeviceTagger<I>;
    /// Provide an iterator over PhysicalDevice taggers without consuming
    /// self.
    fn iter(&self) -> impl Iterator<Item = Self::Tagger>;
}

/// Hidden type which implements PhysicalDevices
struct _PhysicalDevices<'a, I, A> {
    instance: &'a I,
    array: A,
}

struct _PhysicalDeviceIter<'a, I, A> {
    pds: _PhysicalDevices<'a, I, A>,
    next: usize,
}

impl<'a, I, A: AsRef<[vk::PhysicalDevice]>> Iterator for _PhysicalDeviceIter<'a, I, A> {
    type Item = _PhysicalDeviceTagger<'a, I>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = self.pds.array.as_ref();
        if self.next >= array.len() {
            None
        } else {
            let ret = unsafe { array.get_unchecked(self.next) };
            self.next += 1;
            Some(_PhysicalDeviceTagger::new(self.pds.instance, *ret))
        }
    }
}

impl<'a, I, A: AsRef<[vk::PhysicalDevice]>> IntoIterator for _PhysicalDevices<'a, I, A> {
    type Item = _PhysicalDeviceTagger<'a, I>;
    type IntoIter = _PhysicalDeviceIter<'a, I, A>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

impl<'a, I: Instance, A: AsRef<[vk::PhysicalDevice]>> PhysicalDevices<I>
    for _PhysicalDevices<'a, I, A>
{
    type Tagger = _PhysicalDeviceTagger<'a, I>;
    fn iter(&self) -> impl Iterator<Item = Self::Tagger> {
        _PhysicalDeviceIterRef {
            instance: self.instance,
            iter: self.array.as_ref().iter().copied(),
        }
    }
}

struct _PhysicalDeviceIterRef<'a, 's, I> {
    instance: &'a I,
    iter: std::iter::Copied<std::slice::Iter<'s, vk::PhysicalDevice>>,
}

impl<'a, I> Iterator for _PhysicalDeviceIterRef<'a, '_, I> {
    type Item = _PhysicalDeviceTagger<'a, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|pd| _PhysicalDeviceTagger::new(self.instance, pd))
    }
}
