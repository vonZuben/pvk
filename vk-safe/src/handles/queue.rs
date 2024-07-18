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

pub(crate) fn make_queue<'a, D: Device, C: QueueCapability>(
    handle: vk::Queue,
    device: &'a D,
) -> impl Queue<Device = D, Capability = C, Commands = D::Commands> + Captures<&'a D> {
    _Queue {
        handle,
        device,
        capability: PhantomData,
    }
}

struct _Queue<'a, D, C> {
    handle: vk::Queue,
    device: &'a D,
    capability: PhantomData<C>,
}

unsafe impl<'a, D, C> Send for _Queue<'a, D, C> {}
unsafe impl<'a, D, C> Sync for _Queue<'a, D, C> {}
impl<'a, D, C> ThreadSafeHandle for _Queue<'a, D, C> {}

impl<'a, D, C> fmt::Debug for _Queue<'a, D, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("handle", &self.handle)
            // .field("device", &self.device)
            // .field("capability", &self.capability)
            .finish()
    }
}

impl<'a, D, C> Handle for _Queue<'a, D, C> {
    type RawHandle = vk::Queue;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<'a, D: Device, C> DispatchableHandle for _Queue<'a, D, C> {
    type Commands = D::Commands;

    fn commands(&self) -> &Self::Commands {
        todo!()
    }
}

impl<'a, D: Device, C: QueueCapability> Queue for _Queue<'a, D, C> {
    type Device = D;
    type Capability = C;
}
