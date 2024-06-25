//! Vulkan dispatchable handles
//!
//! 🚧 docs in progress

pub mod device;
pub mod instance;
pub mod physical_device;
pub mod queue;

pub mod common;

use crate::scope::{Scope, SecretScope};

use std::fmt::Debug;
use std::ops::DerefMut;

/// Base trait for Handle traits that need to be Scoped
///
/// Handle traits like [`Instance`](crate::vk::Instance) are generally sub traits
/// of this trait.
pub trait ScopedDispatchableHandle<T>:
    DerefMut<Target = SecretScope<Self, T>> + Debug + Send + Sync + Sized
{
}
impl<T: Debug + Send + Sync> ScopedDispatchableHandle<T> for Scope<'_, T> {}

/// Base trait for Handle traits that do not need to be Scoped
///
/// Handle traits like [`Queue`](crate::vk::Queue) are generally sub traits
/// of this trait.
pub trait DispatchableHandle<T>: DerefMut<Target = T> + Debug + Send + Sync + Sized {}
