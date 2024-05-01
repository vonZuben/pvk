//! Vulkan dispatchable handles
//!
//! 🚧 docs in progress

pub(crate) mod device_type;
pub mod instance;
pub(crate) mod physical_device;
pub(crate) mod queue_type;

pub use device_type::Device;
pub use physical_device::PhysicalDevice;
pub use queue_type::Queue;

#[cfg(doc)]
/// Example of concrete Device
///
/// Given some <code>D: [Device]</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _Device<S, C> = crate::scope::RefScope<S, device_type::DeviceType<C>>;

#[cfg(doc)]
/// Example of concrete Instance
///
/// Given some <code>I: [Instance](instance::Instance)</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _Instance<S, C> = crate::scope::RefScope<S, instance::concrete_type::Instance<C>>;

#[cfg(doc)]
/// Example of concrete PhysicalDevice
///
/// Given some <code>Pd: [PhysicalDevice]</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _PhysicalDevice<S, C> = crate::scope::RefScope<S, physical_device::PhysicalDeviceType<C>>;

#[cfg(doc)]
/// Example of concrete Queue
///
/// Given some <code>Q: [Queue]</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _Queue<S, C> = crate::scope::RefScope<S, queue_type::QueueType<C>>;
