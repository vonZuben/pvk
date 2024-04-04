use std::marker::PhantomData;

use vk_safe_sys as vk;

use crate::device_type::Device;
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

pub trait Queue: std::ops::Deref<Target = QueueType<Self::Config>> {
    type Config: QueueConfig<Device = Self::Device>;
    type Device: Device<Commands = Self::Commands>;
    type Capability: QueueCapability;
    type Commands;
}

impl<C: QueueConfig> Queue for QueueType<C> {
    type Config = C;
    type Device = C::Device;
    type Capability = C::Capability;
    type Commands = <C::Device as Device>::Commands;
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
