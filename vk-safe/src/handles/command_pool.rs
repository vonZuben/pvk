use super::device::Device;
use super::Handle;

use std::fmt;
use std::marker::PhantomData;

use crate::flags::Flags;
use crate::type_conversions::ToC;

use vk_safe_sys as vk;

use vk::has_command::DestroyCommandPool;

pub trait CommandPool: Handle<RawHandle = vk::CommandPool> + Send {
    type Device;

    type Flags: Flags<Type = vk::CommandPoolCreateFlags>;

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

impl<
        D: Sync + Device<Commands: DestroyCommandPool>,
        F: Send + Flags<Type = vk::CommandPoolCreateFlags>,
        Q: Send,
    > CommandPool for _CommandPool<'_, D, F, Q>
{
    type Device = D;

    type Flags = F;

    type QueueFamily = Q;
}

impl<'a, D: Device<Commands: DestroyCommandPool>, F, Q> Drop for _CommandPool<'a, D, F, Q> {
    fn drop(&mut self) {
        unsafe {
            self.device.commands().DestroyCommandPool().get_fptr()(
                self.device.raw_handle(),
                self.handle,
                None.to_c(),
            );
        }
    }
}
