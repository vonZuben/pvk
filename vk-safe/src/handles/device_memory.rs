use super::{Handle, ThreadSafeHandle};

use std::fmt;
use std::marker::PhantomData;

use crate::handles::device::Device;

use vk_safe_sys as vk;

use vk::flag_traits::{MemoryHeapFlags, MemoryPropertyFlags};
use vk::has_command::FreeMemory;

pub trait DeviceMemory: Handle<RawHandle = vk::DeviceMemory> + ThreadSafeHandle {
    /// The *specific* Device to which this DeviceMemory belongs
    type Device;
    /// Properties of the memory type this DeviceMemory was allocated with
    type PropertyFlags: MemoryPropertyFlags;
    /// Properties of the memory heap from which this DeviceMemory was allocated
    type HeapFlags: MemoryHeapFlags;
}

/// [`DeviceMemory`] implementor
struct _DeviceMemory<'a, D: Device<Commands: FreeMemory>, P, H> {
    handle: vk::DeviceMemory,
    device: &'a D,
    property_flags: PhantomData<P>,
    heap_flags: PhantomData<H>,
}

pub(crate) fn make_device_memory<
    'a,
    D: Device<Commands: FreeMemory>,
    P: MemoryPropertyFlags,
    H: MemoryHeapFlags,
>(
    handle: vk::DeviceMemory,
    device: &'a D,
) -> impl DeviceMemory<Device = D, PropertyFlags = P, HeapFlags = H> + use<'a, D, P, H> {
    _DeviceMemory {
        handle,
        device,
        property_flags: PhantomData,
        heap_flags: PhantomData,
    }
}

unsafe impl<D: Device<Commands: FreeMemory>, P, H> Send for _DeviceMemory<'_, D, P, H> {}
unsafe impl<D: Device<Commands: FreeMemory>, P, H> Sync for _DeviceMemory<'_, D, P, H> {}
impl<D: Device<Commands: FreeMemory>, P, H> ThreadSafeHandle for _DeviceMemory<'_, D, P, H> {}

impl<D: Device<Commands: FreeMemory>, P, H> fmt::Debug for _DeviceMemory<'_, D, P, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceMemory")
            .field("handle", &self.handle)
            // .field("device", &self.device)
            .finish()
    }
}

impl<D: Device<Commands: FreeMemory>, P, H> Handle for _DeviceMemory<'_, D, P, H> {
    type RawHandle = vk::DeviceMemory;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<D: Device<Commands: FreeMemory>, P: MemoryPropertyFlags, H: MemoryHeapFlags> DeviceMemory
    for _DeviceMemory<'_, D, P, H>
{
    type Device = D;
    type PropertyFlags = P;
    type HeapFlags = H;
}

impl<D: Device<Commands: FreeMemory>, P, H> Drop for _DeviceMemory<'_, D, P, H> {
    fn drop(&mut self) {
        check_vuids::check_vuids!(FreeMemory);

        #[allow(unused_labels)]
        'VUID_vkFreeMemory_memory_00677: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "All submitted commands that refer to memory (via images or buffers) must have completed"
            "execution"
            }

            // everything should take the memory by reference and be done borrowing it before this can happen
        }

        #[allow(unused_labels)]
        'VUID_vkFreeMemory_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkFreeMemory_memory_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If memory is not VK_NULL_HANDLE, memory must be a valid VkDeviceMemory handle"
            }

            // ensured by memory allocation
        }

        #[allow(unused_labels)]
        'VUID_vkFreeMemory_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO always null for now
        }

        #[allow(unused_labels)]
        'VUID_vkFreeMemory_memory_parent: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If memory is a valid handle, it must have been created, allocated, or retrieved from"
            "device"
            }

            // ensured by memory allocation
        }

        unsafe {
            self.device.commands().FreeMemory().get_fptr()(
                self.device.raw_handle(),
                self.raw_handle(),
                std::ptr::null(),
            )
        }
    }
}

/// DeviceMemory which has been mapped for host access
#[derive(Debug)]
pub struct MappedMemory<M> {
    memory: M,
    ptr: *const std::ffi::c_void,
}

impl<M: DeviceMemory> MappedMemory<M> {
    pub(crate) fn handle(&self) -> vk::DeviceMemory {
        self.memory.raw_handle()
    }

    /// Must have actually unmapped the memory
    pub(crate) unsafe fn take(self) -> M {
        self.memory
    }
}

impl<M> MappedMemory<M> {
    pub(crate) fn new(memory: M, ptr: *const std::ffi::c_void) -> Self {
        Self { memory, ptr }
    }
}
