use super::{DispatchableHandle, Handle};

use std::fmt;
use std::marker::PhantomData;

use crate::type_conversions::SafeTransmute;
use crate::vk::Device;

use vk_safe_sys as vk;

use vk::enum_traits::CommandBufferLevel;

pub trait CommandBuffer: DispatchableHandle<RawHandle = vk::CommandBuffer> + Send {
    type Device;
    type Level: CommandBufferLevel;
}

/// [`CommandBuffer`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [CommandBuffer]</code>
/// will be returned.
pub struct _CommandBuffer<'a, D, L> {
    handle: vk::CommandBuffer,
    device: &'a D,
    level: PhantomData<L>,
}

unsafe impl<'a, D, L> SafeTransmute<vk::CommandBuffer> for _CommandBuffer<'a, D, L> {}

unsafe impl<D: Sync, L> Send for _CommandBuffer<'_, D, L> {}

impl<'a, D, L> fmt::Debug for _CommandBuffer<'a, D, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandBuffer")
            .field("handle", &self.handle)
            // .field("device", &self.device)
            // .field("level", &self.level)
            .finish()
    }
}

impl<'a, D, L> Handle for _CommandBuffer<'a, D, L> {
    type RawHandle = vk::CommandBuffer;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<'a, D: Device, L> DispatchableHandle for _CommandBuffer<'a, D, L> {
    type Commands = D::Commands;

    fn commands(&self) -> &Self::Commands {
        self.device.commands()
    }
}

impl<'a, D: Device, L: CommandBufferLevel> CommandBuffer for _CommandBuffer<'a, D, L> {
    type Device = D;
    type Level = L;
}

pub trait CommandBuffers: IntoIterator<Item = Self::CommandBuffer> + fmt::Debug + Send {
    type CommandBuffer: CommandBuffer;
    /// Provide an iterator over CommandBuffers without consuming
    /// self.
    fn iter(&self) -> impl Iterator<Item = Self::CommandBuffer>;
}

/// [`CommandBuffers`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [CommandBuffers]</code>
/// will be returned.
pub struct _CommandBuffers<'a, D, L, A> {
    device: &'a D,
    array: A,
    level: PhantomData<L>,
}

pub(crate) fn make_command_buffers<'a, D, L, A>(
    device: &'a D,
    array: A,
) -> _CommandBuffers<'a, D, L, A> {
    _CommandBuffers {
        device,
        array,
        level: PhantomData,
    }
}

impl<'a, D: Sync + Device, L: Send + CommandBufferLevel, A: Send + AsRef<[vk::CommandBuffer]>>
    CommandBuffers for _CommandBuffers<'a, D, L, A>
{
    type CommandBuffer = _CommandBuffer<'a, D, L>;

    fn iter(&self) -> impl Iterator<Item = Self::CommandBuffer> {
        _CommandBufferIterRef {
            device: self.device,
            iter: self.array.as_ref().iter().copied(),
            level: PhantomData,
        }
    }
}

impl<D, L, A: AsRef<[vk::CommandBuffer]>> fmt::Debug for _CommandBuffers<'_, D, L, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommandBuffers")?;
        f.debug_list().entries(self.array.as_ref().iter()).finish()
    }
}

impl<'a, D, L, A: AsRef<[vk::CommandBuffer]>> IntoIterator for _CommandBuffers<'a, D, L, A> {
    type Item = _CommandBuffer<'a, D, L>;

    type IntoIter = _CommandBufferIter<'a, D, L, A>;

    fn into_iter(self) -> Self::IntoIter {
        _CommandBufferIter {
            command_buffers: self,
            next: 0,
        }
    }
}

/// Iterator over command buffers
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT.
pub struct _CommandBufferIter<'a, D, L, A> {
    command_buffers: _CommandBuffers<'a, D, L, A>,
    next: usize,
}

impl<'a, D, L, A: AsRef<[vk::CommandBuffer]>> Iterator for _CommandBufferIter<'a, D, L, A> {
    type Item = _CommandBuffer<'a, D, L>;
    fn next(&mut self) -> Option<Self::Item> {
        let array = self.command_buffers.array.as_ref();
        if self.next >= array.len() {
            None
        } else {
            let handle = unsafe { array.get_unchecked(self.next) };
            self.next += 1;
            Some(_CommandBuffer {
                handle: *handle,
                device: self.command_buffers.device,
                level: PhantomData,
            })
        }
    }
}

struct _CommandBufferIterRef<'a, 's, D, L> {
    device: &'a D,
    iter: std::iter::Copied<std::slice::Iter<'s, vk::CommandBuffer>>,
    level: PhantomData<L>,
}

impl<'a, D, L> Iterator for _CommandBufferIterRef<'a, '_, D, L> {
    type Item = _CommandBuffer<'a, D, L>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|handle| _CommandBuffer {
            handle,
            device: self.device,
            level: PhantomData,
        })
    }
}
