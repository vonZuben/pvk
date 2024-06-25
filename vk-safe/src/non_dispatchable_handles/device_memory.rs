//! Memory usable by a device
//!
//! [`DeviceMemory`] is allocated with a [`Device`](crate::vk::Device). You can choose the kind of memory
//! you want to allocate after determining the supported memory types of the
//! [`PhysicalDevice`](crate::vk::PhysicalDevice) with
//! [`get_physical_device_memory_properties`](crate::dispatchable_handles::physical_device::concrete_type::ScopedPhysicalDevice::get_physical_device_memory_properties).
//!
//! After choosing a memory type, you can allocate it with
//! [`allocate_memory`](crate::dispatchable_handles::device::concrete_type::ScopedDevice::allocate_memory).

use std::ops::DerefMut;

use crate::flags::Flags;

/** DeviceMemory handle trait

Represents a DeviceMemory

*currently* DeviceMemory does not need to be tagged (it is not clear if this will change in future for now)
*/
pub trait DeviceMemory:
    DerefMut<Target = concrete_type::DeviceMemory<Self::Config>> + std::fmt::Debug
{
    #[doc(hidden)]
    type Config: concrete_type::DeviceMemoryConfig<Device = Self::Device>;
    /// The *specific* Device to which this DeviceMemory belongs
    type Device;
    /// Properties of the memory type this DeviceMemory was allocated with
    type PropertyFlags: Flags;
    /// Properties of the memory heap from which this DeviceMemory was allocated
    type HeapFlags: Flags;
}

/// DeviceMemory which has been mapped for host access
#[derive(Debug)]
pub struct MappedMemory<M> {
    pub(crate) memory: M,
    ptr: *const std::ffi::c_void,
}

impl<M> MappedMemory<M> {
    pub(crate) fn new(memory: M, ptr: *const std::ffi::c_void) -> Self {
        Self { memory, ptr }
    }
}

pub(crate) mod concrete_type {
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};

    use vk_safe_sys as vk;

    use vk::has_command::FreeMemory;

    use crate::flags::Flags;
    use crate::vk::Device;

    pub trait DeviceMemoryConfig {
        type Commands: FreeMemory;
        type Device: Device<Context = Self::Commands>;
        type PropertyFlags: Flags;
        type HeapFlags: Flags;
        fn device(&self) -> &Self::Device;
    }

    pub struct Config<'a, D, P, H> {
        device: &'a D,
        property_flags: PhantomData<P>,
        heap_flags: PhantomData<H>,
    }

    impl<'a, D, P, H> Config<'a, D, P, H> {
        pub(crate) fn new(device: &'a D) -> Self {
            Self {
                device,
                property_flags: PhantomData,
                heap_flags: PhantomData,
            }
        }
    }

    impl<'a, D: Device, P: Flags, H: Flags> DeviceMemoryConfig for Config<'a, D, P, H>
    where
        D::Context: FreeMemory,
    {
        type Commands = D::Context;
        type Device = D;
        type PropertyFlags = P;
        type HeapFlags = H;

        fn device(&self) -> &Self::Device {
            &self.device
        }
    }

    pub struct DeviceMemory<C: DeviceMemoryConfig> {
        pub(crate) handle: vk::DeviceMemory,
        config: C,
    }

    impl<C: DeviceMemoryConfig> Deref for DeviceMemory<C> {
        type Target = Self;

        fn deref(&self) -> &Self::Target {
            self
        }
    }

    impl<C: DeviceMemoryConfig> DerefMut for DeviceMemory<C> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self
        }
    }

    impl<C: DeviceMemoryConfig> super::DeviceMemory for DeviceMemory<C> {
        type Config = C;
        type Device = C::Device;
        type PropertyFlags = C::PropertyFlags;
        type HeapFlags = C::HeapFlags;
    }

    impl<C: DeviceMemoryConfig> DeviceMemory<C> {
        pub(crate) fn new(handle: vk::DeviceMemory, config: C) -> Self {
            Self { handle, config }
        }
    }

    impl<D: DeviceMemoryConfig> std::fmt::Debug for DeviceMemory<D> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.handle.fmt(f)
        }
    }

    impl<D: DeviceMemoryConfig> Drop for DeviceMemory<D> {
        fn drop(&mut self) {
            let fptr = self.config.device().context.FreeMemory().get_fptr();
            check_vuids::check_vuids!(FreeMemory);

            #[allow(unused_labels)]
            'VUID_vkFreeMemory_memory_00677: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "All submitted commands that refer to memory (via images or buffers) must have completed"
                "execution"
                }

                // the memory will be borrowed by objects using the memory, such that the Memory cannot be dropped until done being used
            }

            #[allow(unused_labels)]
            'VUID_vkFreeMemory_device_parameter: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "device must be a valid VkDevice handle"
                }

                // valid from creation
            }

            #[allow(unused_labels)]
            'VUID_vkFreeMemory_memory_parameter: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If memory is not VK_NULL_HANDLE, memory must be a valid VkDeviceMemory handle"
                }

                // valid from creation
            }

            #[allow(unused_labels)]
            'VUID_vkFreeMemory_pAllocator_parameter: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If pAllocator is not NULL, pAllocator must be a valid pointer to a valid VkAllocationCallbacks"
                "structure"
                }

                // TODO: VkAllocationCallbacks not currently supported
            }

            #[allow(unused_labels)]
            'VUID_vkFreeMemory_memory_parent: {
                check_vuids::version! {"1.3.268"}
                check_vuids::description! {
                "If memory is a valid handle, it must have been created, allocated, or retrieved from"
                "device"
                }

                // DeviceMemoryType knows what device it came from
            }
            unsafe {
                fptr(self.config.device().handle, self.handle, std::ptr::null());
            }
        }
    }
}
