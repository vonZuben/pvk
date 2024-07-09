pub mod instance;
pub mod physical_device;

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
pub trait Handle {
    type RawHandle;
    fn raw_handle(&self) -> Self::RawHandle;
}
