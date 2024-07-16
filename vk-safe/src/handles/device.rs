use super::{DispatchableHandle, Handle, ThreadSafeHandle};

use crate::scope::Tag;
use crate::VkVersion;

use std::fmt;
use std::marker::PhantomData;

use vk_safe_sys as vk;

use vk::has_command::DestroyDevice;
use vk::Version;

pub trait Device: DispatchableHandle<RawHandle = vk::Device> + ThreadSafeHandle {
    const VERSION: VkVersion;
}

#[allow(unused)]
// ⚠️ return impl Device after precise capturing in RPITIT is possible
pub(crate) fn make_device<C: DestroyDevice + Version, Tag>(
    handle: vk::Device,
    commands: C,
    _tag: Tag,
    // ) -> impl Device<Commands = C> + Captures<Tag> {
) -> _Device<C, Tag> {
    _Device::<C, Tag> {
        handle,
        commands,
        tag: PhantomData,
    }
}

/// [`Device`] implementor
///
/// ⚠️ This is **NOT** intended to be public. This is only
/// exposed as a stopgap solution to over capturing in
/// RPITIT. After some kind of precise capturing is possible,
/// this type will be made private and <code>impl [Device]</code>
/// will be returned.
pub struct _Device<C: DestroyDevice, T> {
    handle: vk::Device,
    commands: C,
    tag: PhantomData<T>,
}

impl<'t, C: DestroyDevice> _Device<C, Tag<'t>> {
    pub(crate) fn new(handle: vk::Device, commands: C, _tag: Tag<'t>) -> Self {
        Self {
            handle,
            commands,
            tag: PhantomData,
        }
    }
}

unsafe impl<C: DestroyDevice, T> Send for _Device<C, T> {}
unsafe impl<C: DestroyDevice, T> Sync for _Device<C, T> {}
impl<C: DestroyDevice, T> ThreadSafeHandle for _Device<C, T> {}

impl<C: DestroyDevice, T> fmt::Debug for _Device<C, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("_Device")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<C: DestroyDevice, T> Handle for _Device<C, T> {
    type RawHandle = vk::Device;

    fn raw_handle(&self) -> Self::RawHandle {
        self.handle
    }
}

impl<C: DestroyDevice, T> DispatchableHandle for _Device<C, T> {
    type Commands = C;

    fn commands(&self) -> &Self::Commands {
        &self.commands
    }
}

impl<C: DestroyDevice + Version, T> Device for _Device<C, T> {
    const VERSION: VkVersion = C::VERSION;
}
