use std::fmt::Debug;

pub_export_modules!(
instance;
physical_device;
device;
queue;

device_memory;
command_pool;
command_buffer;
);

/// A handle which can dispatch Vulkan Commands
///
/// This is mostly an implementation detail and you are not
/// intended to implement this yourself.
pub trait DispatchableHandle: Handle {
    type Commands;
    fn commands(&self) -> &Self::Commands;
}

/// A handle which represents a Vulkan object
///
/// This is mostly an implementation detail and you are not
/// intended to implement this yourself.
pub trait Handle: Debug + Sized {
    type RawHandle;
    fn raw_handle(&self) -> Self::RawHandle;
}

/// Handles that are safe to use on different threads
///
/// Most handles in Vulkan are thread safe. The primary
/// exemption are CommandPool and CommandBuffer, which
/// are only Send.
pub trait ThreadSafeHandle: Send + Sync {}
