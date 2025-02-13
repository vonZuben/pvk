use super::device::Device;
use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::scope::{Captures, Tag};

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::flag_traits::QueueFlags;

pub trait Queue:
    DispatchableHandle<RawHandle = vk::Queue, Commands: vk::DeviceLabel> + ThreadSafeHandle
{
    type Device;
    type Capability: QueueFlags;
    type Family;
}

pub(crate) unsafe fn make_queue<'a, 't, D: Device, C: QueueFlags>(
    handle: vk::Queue,
    device: &'a D,
    _family_tag: &Tag<'t>,
) -> impl Queue<Device = D, Capability = C, Commands = D::Commands, Family = Tag<'t>> + Captures<&'a D>
{
    _Queue::<'a, D, C, Tag<'t>> {
        handle,
        device,
        capability: PhantomData,
        family_tag: PhantomData,
    }
}

/// [`Queue`] implementor
pub struct _Queue<'a, D, C, T> {
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

impl<'a, D: Device, C: QueueFlags, T> Queue for _Queue<'a, D, C, T> {
    type Device = D;
    type Capability = C;
    type Family = T;
}

#[derive(Clone, Copy)]
pub struct QueueFamilyMarker<T> {
    queue_family_index: u32,
    tag: PhantomData<T>,
}

impl<T> fmt::Debug for QueueFamilyMarker<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueFamilyMarker")
            .field("queue_family_index", &self.queue_family_index)
            .finish()
    }
}

impl<'t> QueueFamilyMarker<Tag<'t>> {
    /// create a queue family marker
    ///
    /// The caller must ensure that the index is a correct index for a family
    /// of [`Queue`] created with the same tag
    pub(crate) unsafe fn new(queue_family_index: u32, _tag: &Tag<'t>) -> Self {
        Self {
            queue_family_index,
            tag: PhantomData,
        }
    }
}

impl<T> QueueFamilyMarker<T> {
    /// get the index for this family for certain Vulkan Commands
    pub(crate) fn family_index(&self) -> u32 {
        self.queue_family_index
    }
}
