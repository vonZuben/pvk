use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::scope::Tag;
use crate::VkVersion;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::DestroyDevice;
use vk::Version;

handle_command_collection_trait!(
    NAME(DeviceCommands)
    IMPL(_Device[C, P, Q, T, X] where C: DestroyDevice<X>)
    #[cfg(VK_VERSION_1_0)] {
        AllocateMemory,
        MapMemory,
        FlushMappedMemoryRanges,
        UnmapMemory,
        DeviceWaitIdle,
        AllocateCommandBuffers,
        GetDeviceQueue,
        CreateCommandPool,
        CreateShaderModule,
    }
);

pub trait DeviceAttributes {
    type PhysicalDevice;
    type QueueConfig;
}

pub trait Device:
    DispatchableHandle<RawHandle = vk::Device> + ThreadSafeHandle + DeviceAttributes + DeviceCommands
{
    const VERSION: VkVersion;
}

/// [`Device`] implementor
struct _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X>,
{
    handle: vk::Device,
    commands: C,
    tag: PhantomData<T>,
    physical_device: PhantomData<P>,
    queue_config: PhantomData<Q>,
    destroy: PhantomData<X>,
}

pub(crate) fn make_device<
    't,
    C: DestroyDevice<X> + Version,
    P: DispatchableHandle<RawHandle = vk::PhysicalDevice>,
    Q,
    X,
>(
    handle: vk::Device,
    commands: C,
    _tag: Tag<'t>,
) -> impl Device<Commands = C, QueueConfig = Q, PhysicalDevice = P> + use<'t, C, P, Q, X> {
    _Device {
        handle,
        commands,
        tag: PhantomData::<Tag<'t>>,
        physical_device: PhantomData,
        queue_config: PhantomData,
        destroy: PhantomData,
    }
}

unsafe impl<C, P, Q, T, X> Send for _Device<C, P, Q, T, X> where C: DestroyDevice<X> {}
unsafe impl<C, P, Q, T, X> Sync for _Device<C, P, Q, T, X> where C: DestroyDevice<X> {}
impl<C, P, Q, T, X> ThreadSafeHandle for _Device<C, P, Q, T, X> where C: DestroyDevice<X> {}

impl<C, P, Q, T, X> fmt::Debug for _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_Device")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<C, P, Q, T, X> Handle for _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X>,
{
    type RawHandle = vk::Device;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<C, P, Q, T, X> vk::Commands for _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X>,
{
    type Commands = C;

    fn commands(&self) -> &Self::Commands {
        &self.commands
    }
}

impl<C, P, Q, T, X> DispatchableHandle for _Device<C, P, Q, T, X> where C: DestroyDevice<X> {}

impl<C, P, Q, T, X> DeviceAttributes for _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X>,
{
    type PhysicalDevice = P;
    type QueueConfig = Q;
}

impl<C, P, Q, T, X> Device for _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X> + Version,
    P: DispatchableHandle<RawHandle = vk::PhysicalDevice>,
{
    const VERSION: VkVersion = C::VERSION;
}

impl<C, P, Q, T, X> Drop for _Device<C, P, Q, T, X>
where
    C: DestroyDevice<X>,
{
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

        unsafe { self.commands.DestroyDevice().get_fptr()(self.handle, std::ptr::null()) }
    }
}
