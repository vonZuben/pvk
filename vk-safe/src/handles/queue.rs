use super::device::Device;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use std::fmt;
use std::marker::PhantomData;

use crate::flags::Flags;
use crate::scope::Captures;

use vk_safe_sys as vk;

pub trait Queue: DispatchableHandle<RawHandle = vk::Queue> + ThreadSafeHandle {
    type Device;
    type Capability: QueueCapability;
}

/// Represents what kind of work can be submitted to the Queue
pub trait QueueCapability: Flags<Type = vk::QueueFlags> {}
impl<T> QueueCapability for T where T: Flags<Type = vk::QueueFlags> {}

pub(crate) fn make_queue<'a, D: Device, C: QueueCapability, T>(
    handle: vk::Queue,
    device: &'a D,
    tag: PhantomData<T>,
) -> impl Queue<Device = D, Capability = C, Commands = D::Commands> + Captures<&'a D> {
    _Queue::<'a, D, C, T> {
        handle,
        device,
        capability: PhantomData,
        family_tag: tag,
    }
}

struct _Queue<'a, D, C, T> {
    handle: vk::Queue,
    device: &'a D,
    capability: PhantomData<C>,
    family_tag: PhantomData<T>,
}

unsafe impl<'a, D, C, T> Send for _Queue<'a, D, C, T> {}
unsafe impl<'a, D, C, T> Sync for _Queue<'a, D, C, T> {}
impl<'a, D, C, T> ThreadSafeHandle for _Queue<'a, D, C, T> {}

impl<'a, D, C, T> fmt::Debug for _Queue<'a, D, C, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("handle", &self.handle)
            // .field("device", &self.device)
            // .field("capability", &self.capability)
            .finish()
    }
}

impl<'a, D, C, T> Handle for _Queue<'a, D, C, T> {
    type RawHandle = vk::Queue;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<'a, D: Device, C, T> DispatchableHandle for _Queue<'a, D, C, T> {
    type Commands = D::Commands;

    fn commands(&self) -> &Self::Commands {
        todo!()
    }
}

impl<'a, D: Device, C: QueueCapability, T> Queue for _Queue<'a, D, C, T> {
    type Device = D;
    type Capability = C;
}
