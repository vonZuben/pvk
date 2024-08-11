use super::device::Device;
use super::Handle;

use std::fmt;
use std::marker::PhantomData;

use crate::type_conversions::ToC;

use vk_safe_sys as vk;

use vk::flag_traits::CommandPoolCreateFlags;
use vk::has_command::DestroyCommandPool;

/// A memory object for allocating CommandBuffers
///
/// These objects are [`Send`] byt not [`Sync`]. This the usage of this object
/// and CommandBuffers allocated from this object but be synchronized. Since CommandBuffers
/// will borrow the CommandPool, this effectively forces the CommandPool and all CommandBuffers
/// allocated therefrom to be locked to the same unit of execution, and they cannot be
/// **individually** sent across different threads. Since they will all be on the same thread,
/// synchronization is guaranteed without locking.
///
/// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkCommandPool.html>
pub trait CommandPool: Handle<RawHandle = vk::CommandPool> + Send {
    type Device;

    type Flags: CommandPoolCreateFlags;

    type QueueFamily;
}

pub(crate) fn make_command_pool<'a, D: Device<Commands: DestroyCommandPool>, F, Q>(
    handle: vk::CommandPool,
    device: &'a D,
) -> _CommandPool<'a, D, F, Q> {
    _CommandPool {
        handle,
        device,
        flags: PhantomData,
        queue_family: PhantomData,
    }
}

/// [`CommandPool`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [CommandPool]</code>
/// will be returned.
pub struct _CommandPool<'a, D: Device<Commands: DestroyCommandPool>, F, Q> {
    handle: vk::CommandPool,
    device: &'a D,
    flags: PhantomData<F>,
    queue_family: PhantomData<Q>,
}

impl<D: Device<Commands: DestroyCommandPool>, F, Q> fmt::Debug for _CommandPool<'_, D, F, Q> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_CommandPool")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<D: Device<Commands: DestroyCommandPool>, F, Q> Handle for _CommandPool<'_, D, F, Q> {
    type RawHandle = vk::CommandPool;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<D: Sync + Device<Commands: DestroyCommandPool>, F: Send + CommandPoolCreateFlags, Q: Send>
    CommandPool for _CommandPool<'_, D, F, Q>
{
    type Device = D;

    type Flags = F;

    type QueueFamily = Q;
}

impl<'a, D: Device<Commands: DestroyCommandPool>, F, Q> Drop for _CommandPool<'a, D, F, Q> {
    fn drop(&mut self) {
        check_vuids::check_vuids!(DestroyCommandPool);

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_commandPool_00041: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "All VkCommandBuffer objects allocated from commandPool must not be in the pending"
            "state"
            }

            // **************TODO*****************************
            // after CommandBuffer submission to Queues is possible
            // it will be necessary to consider how we ensure that
            // Queues have finished with the CommandBuffers
            // before we start destroying everything
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_commandPool_00042: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If VkAllocationCallbacks were provided when commandPool was created, a compatible"
            "set of callbacks must be provided here"
            }

            // TODO
            // no AllocationCallbacks for now
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_commandPool_00043: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If no VkAllocationCallbacks were provided when commandPool was created, pAllocator"
            "must be NULL"
            }

            // TODO
            // no AllocationCallbacks for now
            // always null set below
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_device_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "device must be a valid VkDevice handle"
            }

            // ensured by device creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_commandPool_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If commandPool is not VK_NULL_HANDLE, commandPool must be a valid VkCommandPool handle"
            }

            // ensured by CommandPool creation
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_pAllocator_parameter: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
            "structure"
            }

            // TODO
            // no AllocationCallbacks for now
            // always null set below
        }

        #[allow(unused_labels)]
        'VUID_vkDestroyCommandPool_commandPool_parent: {
            check_vuids::version! {"1.3.268"}
            check_vuids::description! {
            "If commandPool is a valid handle, it must have been created, allocated, or retrieved"
            "from device"
            }

            // the Device and CommandPool handles are held together
        }

        unsafe {
            self.device.commands().DestroyCommandPool().get_fptr()(
                self.device.raw_handle(),
                self.handle,
                None.to_c(),
            );
        }
    }
}
