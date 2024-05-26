use std::marker::PhantomData;

use vk_safe_sys as vk;

use crate::dispatchable_handles::device::Device;
use crate::flags::Flags;

pub trait QueueCapability: Flags<Type = vk::QueueFlags> {}
impl<T> QueueCapability for T where T: Flags<Type = vk::QueueFlags> {}

pub trait QueueConfig {
    type Device: Device;
    type Capability: QueueCapability;
}

pub struct Config<D, Q> {
    device: PhantomData<D>,
    capability: PhantomData<Q>,
}

impl<D: Device, Q: QueueCapability> QueueConfig for Config<D, Q> {
    type Device = D;
    type Capability = Q;
}

/** Queue handle trait

Represents a Queue

*currently* Queue does not need to be scoped
*/
pub trait Queue: std::ops::Deref<Target = QueueType<Self::Config>> {
    #[doc(hidden)]
    type Config: QueueConfig<Device = Self::Device>;
    /// The *specific* Device to which this Queue belongs
    type Device: Device<Context = Self::Context>;
    /// Flags representing the capabilities of the Queue (e.g. if the Queue supports graphics operations)
    type Capability: QueueCapability;
    /// shortcut for the Device context such as the Version and Extensions being used
    type Context;
}

impl<C: QueueConfig> Queue for QueueType<C> {
    type Config = C;
    type Device = C::Device;
    type Capability = C::Capability;
    type Context = <C::Device as Device>::Context;
}

pub struct QueueType<C: QueueConfig> {
    handle: vk::Queue,
    device: C::Device,
}

impl<C: QueueConfig> QueueType<C> {
    pub(crate) fn new(handle: vk::Queue, device: C::Device) -> Self {
        Self { handle, device }
    }
}

impl<C: QueueConfig> std::ops::Deref for QueueType<C> {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl<C: QueueConfig> std::fmt::Debug for QueueType<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueType")
            .field("handle", &self.handle)
            .field("device", &self.device.deref())
            .finish()
    }
}

pub mod queue_exports {
    pub use super::Queue;
}
