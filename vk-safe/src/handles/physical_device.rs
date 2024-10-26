use super::device::_Device; // ⚠️ hidden type exposed until precise capturing in RPITIT is possible
use super::instance::Instance;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::enumerator::Enumerator;
use crate::error::Error;
use crate::scope::{Captures, HasScope, Tag};
use crate::structs::*;
use crate::vk_str::VkStr;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::context::{Context, InstanceDependencies, LoadCommands};
use vk::has_command::DestroyDevice;
use vk::Version;

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

#[cfg(VK_VERSION_1_0)]
get_physical_device_memory_properties;

#[cfg(VK_VERSION_1_0)]
create_device;
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
    fn enumerate_device_layer_properties(&self) -> impl Enumerator<LayerProperties<Self>>
    where
        Self::Commands: vk::has_command::EnumerateDeviceLayerProperties,
    {
        enumerate_device_layer_properties(self)
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
    fn enumerate_device_extension_properties(
        &self,
        layer_name: Option<VkStr>,
    ) -> impl Enumerator<ExtensionProperties<Self>>
    where
        Self::Commands: vk::has_command::EnumerateDeviceExtensionProperties,
    {
        enumerate_device_extension_properties(self, layer_name)
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
    fn get_physical_device_format_properties<F: vk::enum_traits::Format>(
        &self,
        format: F,
    ) -> FormatProperties<Self, F>
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
    /// let image_params = vk::ImageParameters::new(
    ///     vk::Format::R8G8B8A8_SRGB,
    ///     vk::ImageType::TYPE_2D,
    ///     vk::ImageTiling::OPTIMAL,
    ///     vk::flags!(ImageUsageFlags + COLOR_ATTACHMENT_BIT + TRANSFER_DST_BIT),
    ///     (),
    /// );
    ///
    /// let image_format_properties =
    /// physical_device.get_physical_device_image_format_properties(image_params);
    /// # }
    /// ```
    fn get_physical_device_image_format_properties<Params: ImageParameters::ImageParameters>(
        &self,
        params: Params,
    ) -> Result<ImageFormatProperties<Self, Params>, Error>
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
    /// # fn tst<
    /// #   P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>,
    /// #   Params: vk::ImageParameters::ImageParameters,
    /// # >
    /// #   (physical_device: P, image_format_properties: vk::ImageFormatProperties<P, Params>) {
    /// let sparse_image_format_properties =
    ///     physical_device.get_physical_device_sparse_image_format_properties(
    ///         vk::SampleCountFlags::TYPE_1_BIT,
    ///         image_format_properties,
    ///         Vec::new(),
    ///     ).unwrap();
    /// # }
    /// ```
    fn get_physical_device_sparse_image_format_properties<
        Params: ImageParameters::ImageParameters,
        SampleCount: vk::flag_traits::SampleCountFlags,
    >(
        &self,
        samples: SampleCount,
        image_format_properties: ImageFormatProperties<Self, Params>,
    ) -> Result<impl Enumerator<SparseImageFormatProperties<Self>>, Error>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceSparseImageFormatProperties,
    {
        get_physical_device_sparse_image_format_properties(self, samples, image_format_properties)
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
    fn get_physical_device_queue_family_properties(
        &self,
    ) -> impl Enumerator<vk::QueueFamilyProperties, QueueFamiliesTarget<Self>>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceQueueFamilyProperties,
    {
        get_physical_device_queue_family_properties(self)
    }

    #[cfg(VK_VERSION_1_0)]
    /// Query the memory properties of the PhysicalDevice
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>>
    /// #   (physical_device: P) {
    /// let memory_properties = physical_device.get_physical_device_memory_properties();
    /// # }
    /// ```
    fn get_physical_device_memory_properties(&self) -> PhysicalDeviceMemoryProperties<Self>
    where
        Self::Commands: vk::has_command::GetPhysicalDeviceMemoryProperties,
    {
        get_physical_device_memory_properties(self)
    }

    /// Create a device from the PhysicalDevice
    ///
    /// In order to create a Device, you first define the Version and Extensions you will
    /// use with [`vk::device_context!`]. You can then create an [`DeviceCreateInfo`]
    /// structure along with an array of [`DeviceQueueCreateInfo`].
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # use vk::traits::*;
    /// # fn tst<P: vk::PhysicalDevice<Commands: vk::instance::VERSION_1_0>, T>
    /// #   (physical_device: P, create_info: &vk::DeviceCreateInfo<D, (P, T)>, queue_properties: &vk::QueueFamiliesRef<P>) {
    /// vk::tag!(tag);
    /// let device = physical_device.create_device(create_info, tag).unwrap();
    /// # }
    /// ```
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCreateDevice.html>
    fn create_device<'t, C, O, Z: HasScope<Self>>(
        &self,
        create_info: &DeviceCreateInfo<C, Z>,
        tag: Tag<'t>,
    ) -> Result<
        // impl Device<Context = D::Commands, PhysicalDevice = S, QueueConfig = Z> + Captures<Tag<'t>>,
        _Device<C::Commands, Self, Z, Tag<'t>>,
        Error,
    >
    where
        Self::Commands:
            vk::has_command::CreateDevice + vk::has_command::EnumerateDeviceExtensionProperties,
        C: Context + InstanceDependencies<Self::Commands, O> + Send + Sync,
        C::Commands:
            DestroyDevice + LoadCommands + Version + VersionCheck<Self::Commands> + Send + Sync,
    {
        create_device(self, create_info, tag)
    }
}

/// Handle for a PhysicalDevice
///
/// This is just the handle. It must be tagged using the
/// [`tag()`](PhysicalDeviceHandle::tag) method in order
/// to obtain a [`PhysicalDevice`] implementation.
#[repr(transparent)]
pub struct PhysicalDeviceHandle<I> {
    handle: vk::PhysicalDevice,
    instance: PhantomData<I>,
}

impl<I: Instance> PhysicalDeviceHandle<I> {
    /// Tag the handle to make a [`PhysicalDevice`] implementation
    ///
    /// Create an `impl PhysicalDevice` using the handle, and the
    /// same [`Instance`] that it was created from, and a new [`Tag`].
    ///
    /// Note: it is possible to tag the same handle multiple times
    /// with different tags. This is NOT something you should do.
    /// Even if different [`PhysicalDevice`] implementations use the same
    /// PhysicalDevice, they will been seen as different and cannot
    /// be used together.
    pub fn tag<'a, 't>(
        self,
        instance: &'a I,
        tag: Tag<'t>,
    ) -> impl PhysicalDevice<Instance = I, Commands = I::Commands> + Captures<(&'a I, Tag<'t>)>
    {
        _PhysicalDevice::new(self.handle, instance, tag)
    }
}

unsafe impl<I> crate::type_conversions::ConvertWrapper<vk::PhysicalDevice>
    for PhysicalDeviceHandle<I>
{
}

unsafe impl<I> Send for PhysicalDeviceHandle<I> {}
unsafe impl<I> Sync for PhysicalDeviceHandle<I> {}

impl<I> Clone for PhysicalDeviceHandle<I> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            instance: self.instance.clone(),
        }
    }
}

impl<I> Copy for PhysicalDeviceHandle<I> {}

impl<I> fmt::Debug for PhysicalDeviceHandle<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDeviceHandle")
            .field("handle", &self.handle)
            .finish()
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
