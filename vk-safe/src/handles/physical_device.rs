use super::instance::Instance;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::array_storage::ArrayStorage;
use crate::error::Error;
use crate::scope::{Captures, Tag};
use crate::structs::*;
use crate::vk_str::VkStr;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

pub_use_modules!(
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

#[cfg(VK_VERSION_1_0)]
get_physical_device_image_format_properties;

#[cfg(VK_VERSION_1_0)]
get_physical_device_sparse_image_format_properties;

#[cfg(VK_VERSION_1_0)]
get_physical_device_queue_family_properties;
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
pub trait PhysicalDevice:
    DispatchableHandle<RawHandle = vk::PhysicalDevice> + ThreadSafeHandle
{
    type Instance: Instance;

    #[cfg(VK_VERSION_1_0)]
    /// Query the properties of the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
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

    #[cfg(VK_VERSION_1_0)]
    /// Query the device level layers supported by the PhysicalDevice
    ///
    /// Must provide [`ArrayStorage`] space to return the extension properties into.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
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

    #[cfg(VK_VERSION_1_0)]
    /// Query the features supported by the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
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

    #[cfg(VK_VERSION_1_0)]
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
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
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

    #[cfg(VK_VERSION_1_0)]
    /// Query the format properties of the PhysicalDevice
    ///
    /// Provide the [`Format`](crate::vk::Format) to get the properties of that format
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
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

    #[cfg(VK_VERSION_1_0)]
    /// Query the image format properties of the PhysicalDevice
    ///
    /// Provide [`ImageParameters`] with the parameters of an image,
    /// to get the format properties of an image created with such parameters
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// const PARAMS: vk::ImageParameters =
    ///     vk::ImageParameters::new(
    ///     vk::Format::R8G8B8A8_SRGB,
    ///     vk::ImageType::TYPE_2D,
    ///     vk::ImageTiling::OPTIMAL,
    ///     vk::ImageUsageFlags::COLOR_ATTACHMENT_BIT.or(vk::ImageUsageFlags::TRANSFER_DST_BIT),
    ///     vk::ImageCreateFlags::empty(),
    /// );
    ///
    /// let image_format_properties =
    ///     physical_device.get_physical_device_image_format_properties(PARAMS);
    /// # }
    /// ```
    fn get_physical_device_image_format_properties(
        &self,
        params: ImageParameters,
    ) -> Result<ImageFormatProperties<Self>, Error>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceImageFormatProperties,
    {
        get_physical_device_image_format_properties(self, params)
    }

    #[cfg(VK_VERSION_1_0)]
    /// Query the sparse image format properties of the PhysicalDevice
    ///
    /// ### Note
    /// This requires [`ImageFormatProperties`] from
    /// [`get_physical_device_image_format_properties`](PhysicalDevice::get_physical_device_image_format_properties()),
    /// which provides [`ImageParameters`] and ensures that the sample count you choose is supported
    /// by an image with such parameters.
    ///
    /// Must provide the storage space to return the properties to.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P, image_format_properties: vk::ImageFormatProperties<P>) {
    /// let sparse_image_format_properties =
    ///     physical_device.get_physical_device_sparse_image_format_properties(
    ///         vk::SampleCountFlags::TYPE_1_BIT,
    ///         image_format_properties,
    ///         Vec::new(),
    ///     ).unwrap();
    /// # }
    /// ```
    fn get_physical_device_sparse_image_format_properties<
        A: ArrayStorage<SparseImageFormatProperties<Self>>,
    >(
        &self,
        samples: vk::SampleCountFlags,
        image_format_properties: ImageFormatProperties<Self>,
        storage: A,
    ) -> Result<A::InitStorage, Error>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceSparseImageFormatProperties,
    {
        get_physical_device_sparse_image_format_properties(
            self,
            samples,
            image_format_properties,
            storage,
        )
    }

    #[cfg(VK_VERSION_1_0)]
    /// Query the queue family properties of the PhysicalDevice
    ///
    /// Must provide the storage space to return the properties to.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let queue_family_properties =
    ///     physical_device.get_physical_device_queue_family_properties(Vec::new());
    /// # }
    /// ```
    fn get_physical_device_queue_family_properties<A: ArrayStorage<vk::QueueFamilyProperties>>(
        &self,
        storage: A,
    ) -> Result<QueueFamilies<Self, A>, Error>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceQueueFamilyProperties,
    {
        get_physical_device_queue_family_properties(self, storage)
    }
}

/// Hidden type which implements PhysicalDevice
struct _PhysicalDevice<'a, I, T> {
    handle: vk::PhysicalDevice,
    instance: &'a I,
    tag: PhantomData<T>,
}

unsafe impl<I, T> Send for _PhysicalDevice<'_, I, T> {}
unsafe impl<I, T> Sync for _PhysicalDevice<'_, I, T> {}
impl<I, T> ThreadSafeHandle for _PhysicalDevice<'_, I, T> {}

impl<'a, I, T> _PhysicalDevice<'a, I, T> {
    fn new(handle: vk::PhysicalDevice, instance: &'a I, _tag: T) -> Self {
        Self {
            handle,
            instance,
            tag: PhantomData,
        }
    }
}

impl<I, T> fmt::Debug for _PhysicalDevice<'_, I, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_PhysicalDevice")
            .field("handle", &self.handle)
            .finish()
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

impl<I: fmt::Debug> fmt::Debug for _PhysicalDeviceTagger<'_, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceTagger")
            .field("instance", &self.instance)
            .field("physical_device", &self.physical_device)
            .finish()
    }
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
    fn tag<'t>(self, tag: Tag<'t>) -> impl PhysicalDevice<Instance = I, Commands = I::Commands> {
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
/// [`enumerate_physical_devices`](crate::vk::Instance::enumerate_physical_devices).
pub trait PhysicalDeviceTagger<I: Instance>: Sized + fmt::Debug + Send + Sync {
    /// Tag an enumerated PhysicalDevice
    ///
    /// See [`Instance::enumerate_physical_devices`] for
    /// example use.
    fn tag<'t>(self, tag: Tag<'t>) -> impl PhysicalDevice<Instance = I, Commands = I::Commands>;
}

/// Provide access to PhysicalDevices enumerated on the system
///
/// Can be consumed via [`IntoIterator`] implementation, or
/// you can iterator without consuming with [`PhysicalDevices::iter`].
pub trait PhysicalDevices<I: Instance>: IntoIterator<Item = Self::Tagger> + fmt::Debug {
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

impl<I: fmt::Debug, A: AsRef<[vk::PhysicalDevice]>> fmt::Debug for _PhysicalDevices<'_, I, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysicalDevices")?;
        f.debug_list().entries(self.array.as_ref().iter()).finish()
    }
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
