//! Vulkan Queue object
//!
//! Queue are created when creating the logical `Device`. The Queues of the Device that
//! were created can be obtained from the device (🚧 TODO how to get Queues is under
//! consideration for revision).
//!
//! Vulkan doc:
//! <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap5.html#devsandqueues-queues>

use crate::dispatchable_handles::device::Device;
use crate::flags::Flags;

use super::DispatchableHandle;

use vk_safe_sys as vk;

/** Queue handle trait

Represents a Queue

*currently* Queue does not need to be scoped
*/
pub trait Queue: DispatchableHandle<concrete_type::Queue<Self::Config>> {
    #[doc(hidden)]
    type Config: concrete_type::QueueConfig<Device = Self::Device>;
    /// The *specific* Device to which this Queue belongs
    type Device: Device<Context = Self::Context>;
    /// Flags representing the capabilities of the Queue (e.g. if the Queue supports graphics operations)
    type Capability: QueueCapability;
    /// shortcut for the Device context such as the Version and Extensions being used
    type Context;
}

/// Represents what kind of work can be submitted to the Queue
pub trait QueueCapability: Flags<Type = vk::QueueFlags> {}
impl<T> QueueCapability for T where T: Flags<Type = vk::QueueFlags> {}

pub(crate) mod concrete_type {
    use std::marker::PhantomData;

    use vk_safe_sys as vk;

    use crate::dispatchable_handles::device::Device;

    pub trait QueueConfig: Send + Sync {
        type Device: Device;
        type Capability: super::QueueCapability;
        fn device(&self) -> &Self::Device;
    }

    pub struct Config<'a, D, Q> {
        device: &'a D,
        capability: PhantomData<Q>,
    }

    impl<'a, D, Q> Config<'a, D, Q> {
        pub(crate) fn new(device: &'a D) -> Self {
            Self {
                device,
                capability: PhantomData,
            }
        }
    }

    impl<'a, D: Device, Q: super::QueueCapability> QueueConfig for Config<'a, D, Q> {
        type Device = D;
        type Capability = Q;

        fn device(&self) -> &Self::Device {
            &self.device
        }
    }

    impl<C: QueueConfig> super::Queue for Queue<C> {
        type Config = C;
        type Device = C::Device;
        type Capability = C::Capability;
        type Context = <C::Device as Device>::Context;
    }

    pub struct Queue<C: QueueConfig> {
        handle: vk::Queue,
        config: C,
    }

    unsafe impl<C: QueueConfig> Send for Queue<C> {}
    unsafe impl<C: QueueConfig> Sync for Queue<C> {}

    impl<C: QueueConfig> Queue<C> {
        pub(crate) fn new(handle: vk::Queue, config: C) -> Self {
            Self { handle, config }
        }
    }

    impl<C: QueueConfig> std::ops::Deref for Queue<C> {
        type Target = Self;

        fn deref(&self) -> &Self::Target {
            self
        }
    }

    impl<C: QueueConfig> std::ops::DerefMut for Queue<C> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self
        }
    }

    impl<C: QueueConfig> super::DispatchableHandle<Queue<C>> for Queue<C> {}

    impl<C: QueueConfig> std::fmt::Debug for Queue<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("QueueType")
                .field("handle", &self.handle)
                .field("device", &self.config.device().deref())
                .finish()
        }
    }
}
