use super::device_memory::{DeviceMemory, MappedMemory, _DeviceMemory};
use super::physical_device::PhysicalDevice;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::error::Error;
use crate::flags::{Excludes, Flags, Includes};
use crate::scope::Tag;
use crate::structs::*;
use crate::type_conversions::ToC;
use crate::vk::QueueCapability;
use crate::VkVersion;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::DestroyDevice;
use vk::Version;

use vk::flag_types::MemoryHeapFlags::MULTI_INSTANCE_BIT;
use vk::flag_types::MemoryPropertyFlags::HOST_VISIBLE_BIT;

pub_use_modules!(
#[cfg(VK_VERSION_1_0)]
allocate_memory;

#[cfg(VK_VERSION_1_0)]
map_memory;

#[cfg(VK_VERSION_1_0)]
flush_mapped_memory_ranges;

#[cfg(VK_VERSION_1_0)]
unmap_memory;

#[cfg(VK_VERSION_1_0)]
wait_idle;

#[cfg(VK_VERSION_1_0)]
get_queue_family;
);

pub trait Device: DispatchableHandle<RawHandle = vk::Device> + ThreadSafeHandle {
    const VERSION: VkVersion;

    type PhysicalDevice: PhysicalDevice;
    type QueueConfig;

    #[cfg(VK_VERSION_1_0)]
    /// Allocate memory on the Device
    ///
    /// Provide a [`MemoryAllocateInfo`] structure with the information
    /// about amount and type of memory you want to allocate.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::traits::*;
    /// # fn tst<D: vk::Device<Commands: vk::device::VERSION_1_0>, P: vk::Flags, H: vk::Flags>
    /// #   (device: D, alloc_info: vk::MemoryAllocateInfo<D::PhysicalDevice, P, H>) {
    /// let memory = device.allocate_memory(&alloc_info);
    /// # }
    /// ```
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkAllocateMemory.html>
    fn allocate_memory<P: Flags, H: Flags>(
        &self,
        info: &MemoryAllocateInfo<Self::PhysicalDevice, P, H>,
    ) -> Result<
        // impl DeviceMemory<Device = S, PropertyFlags = P, HeapFlags = H> + Captures<&Self>,
        _DeviceMemory<Self, P, H>,
        vk::Result,
    >
    where
        Self::Commands: vk::has_command::AllocateMemory + vk::has_command::FreeMemory,
    {
        allocate_memory(self, info)
    }

    #[cfg(VK_VERSION_1_0)]
    /// Map memory for host access
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # use vk::flag_types::MemoryHeapFlags::MULTI_INSTANCE_BIT;
    /// # use vk::flag_types::MemoryPropertyFlags::HOST_VISIBLE_BIT;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// #    P: vk::Includes<HOST_VISIBLE_BIT>,
    /// #    H: vk::Excludes<MULTI_INSTANCE_BIT>
    /// # >
    /// #   (device: D, memory: impl vk::DeviceMemory<Device = D, PropertyFlags = P, HeapFlags = H>) {
    /// let mapped_memory = device.map_memory(memory);
    /// # }
    /// ```
    ///
    /// ### Note
    /// *currently this can only be used to map the whole memory range. There may be breaking change in
    /// future to make the API more inline with the real `vkMapMemory`, which allows mapping sub ranges*
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkMapMemory.html>
    fn map_memory<
        M: DeviceMemory<
            Device = Self,
            PropertyFlags: Includes<HOST_VISIBLE_BIT>,
            HeapFlags: Excludes<MULTI_INSTANCE_BIT>,
        >,
    >(
        &self,
        memory: M,
    ) -> Result<MappedMemory<M>, Error>
    where
        Self::Commands: vk::has_command::MapMemory,
    {
        map_memory(self, memory)
    }

    #[cfg(VK_VERSION_1_0)]
    /// Flush memory to make host writes visible to the device
    ///
    /// ```
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// #    M: vk::DeviceMemory<Device = D>
    /// # >
    /// #   (device: D, mapped_memory: vk::MappedMemory<M>) {
    /// let ranges = [vk::MappedMemoryRange::whole_range(&mapped_memory)];
    /// device.flush_mapped_memory_ranges(&ranges).unwrap();
    /// # }
    /// ```
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkFlushMappedMemoryRanges.html>
    fn flush_mapped_memory_ranges(&self, ranges: &[MappedMemoryRange<Self>]) -> Result<(), Error>
    where
        Self::Commands: vk::has_command::FlushMappedMemoryRanges,
    {
        flush_mapped_memory_ranges(self, ranges)
    }

    #[cfg(VK_VERSION_1_0)]
    /// Unmap memory for host access
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// #    M: vk::DeviceMemory<Device = D>,
    /// # >
    /// #   (device: D, mapped_memory: vk::MappedMemory<M>) {
    /// let memory = device.unmap_memory(mapped_memory);
    /// # }
    /// ```
    fn unmap_memory<M: DeviceMemory<Device = Self>>(&self, mapped_memory: MappedMemory<M>) -> M
    where
        Self::Commands: vk::has_command::UnmapMemory,
    {
        unmap_memory(self, mapped_memory)
    }

    #[cfg(VK_VERSION_1_0)]
    /// Wait for all queue operations on the device to complete.
    ///
    /// Blocks until **all** operations on **all** `Queue`s belonging to this `Device` are
    /// complete.
    ///
    /// *Can fail in exceptional situations. Will return Ok(()) on success.*
    ///
    /// # SAFETY
    /// You **must not** call any methods on any [`Queue`](TODO) object
    /// created from this Device, on any other threads at the same time as calling
    /// this method.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # fn tst<
    /// #    D: vk::Device<Commands: vk::device::VERSION_1_0>,
    /// # >
    /// #   (mut device: D) {
    /// let result = unsafe { device.wait_idle() };
    /// # }
    /// ```
    unsafe fn wait_idle(&self) -> Result<(), Error>
    where
        Self::Commands: vk::has_command::DeviceWaitIdle,
    {
        wait_idle(self)
    }

    /// Get a QueueFamily which should have specific capabilities
    ///
    /// In vk-safe, you do not directly get queues from the Device. Rather,
    /// you first get a type that represents a [`QueueFamily`] that you already
    /// configured by by passing in the same queue configuration and properties
    /// parameters used when creating the Device.
    ///
    /// From the returned [`QueueFamily`], you can obtain individual
    /// [`Queue`](crate::vk::Queue) objects.
    ///
    /// Returns the [`QueueFamily`] if the [`QueueCapability`] is supported.
    /// Otherwise returns [`UnsupportedCapability`].
    fn get_queue_family<'a, 't, Q: QueueCapability>(
        &'a self,
        queue_config: &DeviceQueueCreateInfo<Self::QueueConfig>,
        queue_family_properties: &QueueFamiliesRef<Self::PhysicalDevice>,
        capability: Q,
        tag: Tag<'t>,
    ) -> Result<_QueueFamily<'a, Self, Q, Tag<'t>>, UnsupportedCapability> {
        get_queue_family(self, queue_config, queue_family_properties, capability, tag)
    }
}

// #[allow(unused)]
// // ⚠️ return impl Device after precise capturing in RPITIT is possible
// pub(crate) fn make_device<C: DestroyDevice + Version, Tag>(
//     handle: vk::Device,
//     commands: C,
//     _tag: Tag,
//     // ) -> impl Device<Commands = C> + Captures<Tag> {
// ) -> _Device<C, Tag> {
//     _Device::<C, Tag> {
//         handle,
//         commands,
//         tag: PhantomData,
//     }
// }

/// [`Device`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [Device]</code>
/// will be returned.
pub struct _Device<C: DestroyDevice, P, Q, T> {
    handle: vk::Device,
    commands: C,
    tag: PhantomData<T>,
    physical_device: PhantomData<P>,
    queue_config: PhantomData<Q>,
}

impl<'t, C: DestroyDevice, P, Q> _Device<C, P, Q, Tag<'t>> {
    pub(crate) fn new(handle: vk::Device, commands: C, _tag: Tag<'t>) -> Self {
        Self {
            handle,
            commands,
            tag: PhantomData,
            physical_device: PhantomData,
            queue_config: PhantomData,
        }
    }
}

unsafe impl<C: DestroyDevice, P, Q, T> Send for _Device<C, P, Q, T> {}
unsafe impl<C: DestroyDevice, P, Q, T> Sync for _Device<C, P, Q, T> {}
impl<C: DestroyDevice, P, Q, T> ThreadSafeHandle for _Device<C, P, Q, T> {}

impl<C: DestroyDevice, P, Q, T> fmt::Debug for _Device<C, P, Q, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_Device")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<C: DestroyDevice, P, Q, T> Handle for _Device<C, P, Q, T> {
    type RawHandle = vk::Device;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<C: DestroyDevice, P, Q, T> DispatchableHandle for _Device<C, P, Q, T> {
    type Commands = C;

    fn commands(&self) -> &Self::Commands {
        &self.commands
    }
}

impl<C: DestroyDevice + Version, P: PhysicalDevice, Q, T> Device for _Device<C, P, Q, T> {
    const VERSION: VkVersion = C::VERSION;

    type PhysicalDevice = P;
    type QueueConfig = Q;
}

impl<C: DestroyDevice, P, Q, T> Drop for _Device<C, P, Q, T> {
    fn drop(&mut self) {
        check_vuids::check_vuids!(DestroyDevice);

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_05137: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "All child objects created on device must have been destroyed prior to destroying device"
            }

            // everything borrowing device should be doe before this can happen
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_00379: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkAllocationCallbacks were provided when device was created, a compatible set of"
            "callbacks must be provided here"
            }

            // TODO always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_00380: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If no VkAllocationCallbacks were provided when device was created, pAllocator must"
            "be NULL"
            }

            // TODO always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If device is not NULL, device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyDevice_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO always null for now
        }

        unsafe { self.commands.DestroyDevice().get_fptr()(self.handle, None.to_c()) }
    }
}
