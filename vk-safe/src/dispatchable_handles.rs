//! Vulkan dispatchable handles
//!
//! 🚧 docs in progress

pub mod device;
pub mod instance;
pub mod physical_device;
pub mod queue;

pub mod common;

#[cfg(doc)]
/// Example of concrete Device
///
/// Given some <code>D: [Device](device::Device)</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _Device<S, C> = crate::scope::SecretScope<S, device::concrete_type::Device<C>>;

#[cfg(doc)]
/// Example of concrete Instance
///
/// Given some <code>I: [Instance](instance::Instance)</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _Instance<S, C> = crate::scope::SecretScope<S, instance::concrete_type::Instance<C>>;

#[cfg(doc)]
/// Example of concrete PhysicalDevice
///
/// Given some <code>Pd: [PhysicalDevice](physical_device::PhysicalDevice)</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _PhysicalDevice<S, C> =
    crate::scope::SecretScope<S, physical_device::concrete_type::PhysicalDevice<C>>;

#[cfg(doc)]
/// Example of concrete Queue
///
/// Given some <code>Q: [Queue]</code>, you will implicitly have access to a concrete type like this. All
/// the methods shown below will be accessible so long as the appropriate Version or
/// Extension is also enabled.
///
/// 🛑 This is only generated for the documentation and is not usable in your code.
pub type _Queue<S, C> = crate::scope::SecretScope<S, queue::concrete_type::Queue<C>>;
