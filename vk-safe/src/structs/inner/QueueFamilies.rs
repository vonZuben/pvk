use std::fmt;
use std::marker::PhantomData;

use crate::buffer::Buffer;
use crate::scope::Tag;
use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

/// Properties for queue families by family index
///
/// This is a wrapper for an array of [`QueueFamilyProperties`](vk_safe_sys::QueueFamilyProperties).
/// The index of the properties is the queue family index. The wrapper ensures
/// that the index relationship is maintained.
///
/// # Device configuration
/// A key step in configuring a logical Device is configuring the Queues
/// with [`DeviceQueueCreateInfo`](super::DeviceQueueCreateInfo).
/// The `QueueFamilyProperties` are needed to determine what the Queues
/// of each family can do, and how many Queues can be made for the family.
///
/// Call [`properties_iter`](QueueFamiliesRef::properties_iter) to
/// determine all the `QueueFamilyProperties`.
pub struct QueueFamilies<S, B> {
    buffer: B,
    scope: PhantomData<S>,
}

impl<S, B> QueueFamilies<S, B> {
    pub(crate) fn new(buffer: B) -> Self {
        Self {
            buffer,
            scope: PhantomData,
        }
    }
}

impl<S, B: Buffer<vk::QueueFamilyProperties>> std::ops::Deref for QueueFamilies<S, B> {
    type Target = QueueFamiliesRef<S>;

    fn deref(&self) -> &Self::Target {
        unsafe { <&Self::Target>::from_c(self.buffer.get_slice()) }
    }
}

impl<S, B: Buffer<vk::QueueFamilyProperties>> fmt::Debug for QueueFamilies<S, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.buffer.get_slice().iter())
            .finish()
    }
}

/// Reference to QueueFamilies
///
/// This is mainly for abstracting away the [`Buffer`] generics
/// of [`QueueFamilies`].
#[repr(transparent)]
pub struct QueueFamiliesRef<S> {
    scope: PhantomData<S>,
    families: [vk::QueueFamilyProperties],
}
unsafe impl<S> ConvertWrapper<[vk::QueueFamilyProperties]> for QueueFamiliesRef<S> {}

impl<S> QueueFamiliesRef<S> {
    /// Iterate over [`QueueFamilyProperties`] with a provided [`Tag`]
    ///
    /// The `tag` ensures that all `properties` are related to the same
    /// collection. The `properties` are not Copy/Clone to ensure that each one
    /// can only be used once per tag. This is because you may only configure
    /// each QueueFamily once per logical Device created.
    pub fn properties_iter<'id>(&self, tag: Tag<'id>) -> QueueFamilyIter<(S, Tag<'id>)> {
        let _ = tag;
        QueueFamilyIter {
            iter: self.iter().enumerate(),
            _scope: PhantomData,
        }
    }
}

impl<S> std::ops::Deref for QueueFamiliesRef<S> {
    type Target = [vk::QueueFamilyProperties];

    fn deref(&self) -> &Self::Target {
        &self.families
    }
}

/// An iterator over QueueFamilyProperties
pub struct QueueFamilyIter<'a, Z> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, vk::QueueFamilyProperties>>,
    _scope: PhantomData<Z>,
}

impl<'a, Z> Iterator for QueueFamilyIter<'a, Z> {
    type Item = QueueFamilyProperties<'a, Z>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, properties) = self.iter.next()?;
        let family_index: u32 = index.try_into().ok()?;
        Some(QueueFamilyProperties {
            properties,
            family_index,
            _scope: PhantomData,
        })
    }
}

/// Properties of a Queue Family
///
/// Acquire by iterating over [`QueueFamilies`] via
/// [`properties_iter`](QueueFamiliesRef::properties_iter).
///
/// Each Queue family may only be configured 0 or 1 times.
/// A `tag` is needed to ensure that all `QueueFamilyProperties`
/// are related to the same collection. They are also
/// not Copy/Clone to ensure each one can only be used once.
pub struct QueueFamilyProperties<'a, Z> {
    properties: &'a vk::QueueFamilyProperties,
    pub family_index: u32,
    _scope: PhantomData<Z>,
}

impl<Z> std::ops::Deref for QueueFamilyProperties<'_, Z> {
    type Target = vk::QueueFamilyProperties;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}
