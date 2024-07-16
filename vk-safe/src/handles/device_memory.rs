use super::{Handle, ThreadSafeHandle};

use std::fmt;

use crate::handles::device::Device;
use crate::type_conversions::ToC;

use vk_safe_sys as vk;

use vk::has_command::FreeMemory;

pub trait DeviceMemory: Handle + ThreadSafeHandle {
    type Device;
}

/// [`DeviceMemory`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [Device]</code>
/// will be returned.
pub struct _DeviceMemory<'a, D: Device<Commands: FreeMemory>> {
    handle: vk::DeviceMemory,
    device: &'a D,
}

impl<'a, D: Device<Commands: FreeMemory>> _DeviceMemory<'a, D> {
    pub(crate) fn new(handle: vk::DeviceMemory, device: &'a D) -> Self {
        Self { handle, device }
    }
}

unsafe impl<D: Device<Commands: FreeMemory>> Send for _DeviceMemory<'_, D> {}
unsafe impl<D: Device<Commands: FreeMemory>> Sync for _DeviceMemory<'_, D> {}
impl<D: Device<Commands: FreeMemory>> ThreadSafeHandle for _DeviceMemory<'_, D> {}

impl<D: Device<Commands: FreeMemory>> fmt::Debug for _DeviceMemory<'_, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceMemory")
            .field("handle", &self.handle)
            // .field("device", &self.device)
            .finish()
    }
}

impl<D: Device<Commands: FreeMemory>> Handle for _DeviceMemory<'_, D> {
    type RawHandle = vk::DeviceMemory;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<D: Device<Commands: FreeMemory>> DeviceMemory for _DeviceMemory<'_, D> {
    type Device = D;
}

impl<D: Device<Commands: FreeMemory>> Drop for _DeviceMemory<'_, D> {
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
                None.to_c(),
            )
        }
    }
}
