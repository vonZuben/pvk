use super::device_memory::_DeviceMemory;
use super::physical_device::PhysicalDevice;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::flags::Flags;
use crate::scope::Tag;
use crate::structs::MemoryAllocateInfo;
use crate::type_conversions::ToC;
use crate::VkVersion;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::DestroyDevice;
use vk::Version;

pub_use_modules!(
#[cfg(VK_VERSION_1_0)]
allocate_memory;
);

pub trait Device: DispatchableHandle<RawHandle = vk::Device> + ThreadSafeHandle {
    const VERSION: VkVersion;

    type PhysicalDevice: PhysicalDevice;

    /// Allocate memory on the Device
    ///
    /// Provide a [`MemoryAllocateInfo`] structure with the information
    /// about amount and type of memory you want to allocate.
    ///
    /// ```rust
    /// # use vk_safe::vk;
    /// # vk::device_context!(D: VERSION_1_0);
    /// # fn tst<C: vk::device::VERSION_1_0, D: vk::Device<Context = C>, P: vk::Flags, H: vk::Flags>
    /// #   (device: D, alloc_info: vk::MemoryAllocateInfo<D::PhysicalDevice, P, H>) {
    /// let memory = device.allocate_memory(&alloc_info);
    /// # }
    /// ```
    fn allocate_memory<P: Flags, H: Flags>(
        &self,
        info: &MemoryAllocateInfo<Self::PhysicalDevice, P, H>,
    ) -> Result<
        // impl DeviceMemory<Device = S, PropertyFlags = P, HeapFlags = H> + Captures<&Self>,
        _DeviceMemory<Self>,
        vk::Result,
    >
    where
        Self::Commands: vk::has_command::AllocateMemory + vk::has_command::FreeMemory,
    {
        allocate_memory(self, info)
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
pub struct _Device<C: DestroyDevice, P, T> {
    handle: vk::Device,
    commands: C,
    tag: PhantomData<T>,
    physical_device: PhantomData<P>,
}

impl<'t, C: DestroyDevice, P> _Device<C, P, Tag<'t>> {
    pub(crate) fn new(handle: vk::Device, commands: C, _tag: Tag<'t>) -> Self {
        Self {
            handle,
            commands,
            tag: PhantomData,
            physical_device: PhantomData,
        }
    }
}

unsafe impl<C: DestroyDevice, P, T> Send for _Device<C, P, T> {}
unsafe impl<C: DestroyDevice, P, T> Sync for _Device<C, P, T> {}
impl<C: DestroyDevice, P, T> ThreadSafeHandle for _Device<C, P, T> {}

impl<C: DestroyDevice, P, T> fmt::Debug for _Device<C, P, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_Device")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<C: DestroyDevice, P, T> Handle for _Device<C, P, T> {
    type RawHandle = vk::Device;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<C: DestroyDevice, P, T> DispatchableHandle for _Device<C, P, T> {
    type Commands = C;

    fn commands(&self) -> &Self::Commands {
        &self.commands
    }
}

impl<C: DestroyDevice + Version, P: PhysicalDevice, T> Device for _Device<C, P, T> {
    const VERSION: VkVersion = C::VERSION;

    type PhysicalDevice = P;
}

impl<C: DestroyDevice, P, T> Drop for _Device<C, P, T> {
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
