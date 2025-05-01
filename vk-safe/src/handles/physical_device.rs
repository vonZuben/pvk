use super::instance::Instance;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::scope::{Captures, Tag};

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::Support;

handle_command_collection_trait!(
    NAME(PhysicalDeviceCommands)
    IMPL( _PhysicalDevice['a, I, T, V, E] where I: Instance )
    #[cfg(VK_VERSION_1_0)] {
        GetPhysicalDeviceProperties,
        EnumerateDeviceLayerProperties,
        GetPhysicalDeviceFeatures,
        EnumerateDeviceExtensionProperties,
        GetPhysicalDeviceFormatProperties,
        GetPhysicalDeviceImageFormatProperties,
        GetPhysicalDeviceSparseImageFormatProperties,
        GetPhysicalDeviceQueueFamilyProperties,
        GetPhysicalDeviceMemoryProperties,
        CreateDevice,
    }
    #[cfg(VK_VERSION_1_1)] {
        GetPhysicalDeviceProperties2,
    }
);

/// PhysicalDevice handle trait
///
/// Represents a *specific* PhysicalDevice which has been scoped.
///
/// Obtained using [`PhysicalDeviceHandle::tag`].
pub trait PhysicalDevice:
    DispatchableHandle<RawHandle = vk::PhysicalDevice, Commands: vk::Version>
    + ThreadSafeHandle
    + Support<Version = Self::DeviceVersion, Extensions = Self::DeviceExtensions>
    + Copy // should this be Copy???
    + PhysicalDeviceCommands
{
    type Instance: Instance;
    type DeviceVersion;
    type DeviceExtensions;

    // /// Check if the actual Device supports the requested Vulkan version
    // ///
    // /// returns a new `impl PhysicalDevice` which indicates support for
    // /// the requested Vulkan version if it is actually supported.
    // fn check_device_version<C: vk::context::Context<Commands: vk::Version>>(
    //     self,
    //     properties: &PhysicalDeviceProperties<Self>,
    //     context: C,
    // ) -> Option<
    //     impl PhysicalDevice<
    //             Instance = Self::Instance,
    //             Commands = Self::Commands,
    //             DeviceVersion = C,
    //             DeviceExtensions = Self::DeviceExtensions,
    //         > + Captures<Self>,
    // > {
    //     let _ = context;
    //     if properties.api_version() > <C::Commands as vk::Version>::VERSION {
    //         Some(unsafe { self.convert_device_version(context) })
    //     } else {
    //         None
    //     }
    // }

    /// Check if the actual Device supports the requested Vulkan extensions
    ///
    /// returns a new `impl PhysicalDevice` which indicates support for
    /// the requested Vulkan extensions if it is actually supported.
    fn check_device_extensions<C: vk::context::Extensions>(
        self,
        extension_properties: &[crate::vk::ExtensionProperties<Self>],
        context: C,
    ) -> Option<
        impl PhysicalDevice<
                Instance = Self::Instance,
                Commands = Self::Commands,
                DeviceVersion = Self::DeviceVersion,
                DeviceExtensions = C,
            > + use<Self, C>,
    >;
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
    ) -> impl PhysicalDevice<
        Instance = I,
        Commands = I::Commands,
        DeviceVersion = (),
        DeviceExtensions = (),
    > + Captures<(&'a I, Tag<'t>)> {
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
struct _PhysicalDevice<'a, I, T, V, E> {
    handle: vk::PhysicalDevice,
    instance: &'a I,
    tag: PhantomData<T>,
    device_version: PhantomData<V>,
    device_extensions: PhantomData<E>,
}

impl<'a, I, T, V, E> Copy for _PhysicalDevice<'a, I, T, V, E> {}

impl<'a, I, T, V, E> Clone for _PhysicalDevice<'a, I, T, V, E> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<'a, I, T, V, E> Send for _PhysicalDevice<'a, I, T, V, E> {}
unsafe impl<'a, I, T, V, E> Sync for _PhysicalDevice<'a, I, T, V, E> {}
impl<'a, I, T, V, E> ThreadSafeHandle for _PhysicalDevice<'a, I, T, V, E> {}

impl<'a, I, T, V, E> _PhysicalDevice<'a, I, T, V, E> {
    fn new(handle: vk::PhysicalDevice, instance: &'a I, _tag: T) -> Self {
        Self {
            handle,
            instance,
            tag: PhantomData,
            device_version: PhantomData,
            device_extensions: PhantomData,
        }
    }
}

impl<'a, I, T, V, E> fmt::Debug for _PhysicalDevice<'a, I, T, V, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_PhysicalDevice")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<'a, I, T, V, E> Handle for _PhysicalDevice<'a, I, T, V, E> {
    type RawHandle = vk::PhysicalDevice;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<'a, I, T, V, E> vk::Commands for _PhysicalDevice<'a, I, T, V, E>
where
    I: Instance,
{
    type Commands = I::Commands;

    fn commands(&self) -> &Self::Commands {
        self.instance.commands()
    }
}

impl<'a, I, T, V, E> DispatchableHandle for _PhysicalDevice<'a, I, T, V, E> where I: Instance {}

impl<'a, I, T, V, E> PhysicalDevice for _PhysicalDevice<'a, I, T, V, E>
where
    I: Instance,
{
    type Instance = I;
    type DeviceVersion = V;
    type DeviceExtensions = E;

    fn check_device_extensions<C: vk_safe_sys::context::Extensions>(
        self,
        extension_properties: &[crate::vk::ExtensionProperties<Self>],
        context: C,
    ) -> Option<
        impl PhysicalDevice<
                Instance = Self::Instance,
                Commands = Self::Commands,
                DeviceVersion = Self::DeviceVersion,
                DeviceExtensions = C,
            > + use<'a, I, T, V, E, C>,
    > {
        let _ = context;

        let requested_extensions = C::list_of_extensions();
        let requested_extensions = requested_extensions.as_ref();

        // TODO - FIX PERFORMANCE
        // This is O(n^2) since we can end up comparing every element to every element, of each list
        // It would be nice if there is a standard about the order that extension names are
        // listed, so we can reduce to O(n) ordered searching
        for extension in requested_extensions {
            if extension_properties
                .iter()
                .find(|e| {
                    let e = unsafe { vk_safe_sys::VkStrRaw::new(e.extension_name.as_ptr()) };
                    e == *extension
                })
                .is_none()
            {
                return None;
            }
        }

        Some(_PhysicalDevice::new(self.handle, self.instance, self.tag))
    }
}

unsafe impl<'a, I, T, V, E> Support for _PhysicalDevice<'a, I, T, V, E> {
    type Version = V;
    type Extensions = E;
}
