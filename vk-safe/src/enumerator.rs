use std::marker::PhantomData;

use crate::buffer::Buffer;
use crate::error::Error;

/// Types which enumerate or get items Vulkan
///
/// Vulkan has numerous "enumerate" or "get" commands which
/// allow a caller to query a number of items to be returned,
/// allocate space for the items, and then finally get the items.
/// This is a tedious and error prone set of steps.
///
/// This trait provides a set of methods which is implemented
/// by vk-safe types to allow safely and conveniently enumerating
/// or getting the items.
pub trait Enumerator<I, T: EnumeratorTarget = DefaultTarget> {
    /// Automatically enumerate or get the items
    ///
    /// This will automatically call the underlying Vulkan command
    /// a first time in order to query the number of items, allocate
    /// space with a [`Vec`] for the number of items, and call the
    /// command a second time with the allocated space to enumerate
    /// or get the items.
    fn auto_get_enumerate(&self) -> Result<T::Target<Vec<I>>, Error> {
        let vec = Vec::with_capacity(self.get_len()?);
        self.get_enumerate(vec)
    }

    /// Use the Vulkan command to query the number of items
    ///
    /// Generally, this calls the underlying Vulkan command
    /// with a null pointer for the return buffer, which is a signal
    /// to not enumerate or get any items, and only return the number
    /// of items that are available.
    fn get_len(&self) -> Result<usize, Error>;

    /// Enumerate or get the items using the provided buffer
    ///
    /// Generally, this calls the underlying Vulkan command
    /// with a pointer and capacity of the provided buffer, and
    /// the Vulkan implementation with write the items to the buffer,
    /// and set the length of the buffer.
    ///
    /// The user can choose to pass ownership of the buffer, or provide
    /// a mutable reference to a buffer to allow temporary usage of the
    /// buffer.
    fn get_enumerate<B: Buffer<I>>(&self, buffer: B) -> Result<T::Target<B>, Error>;
}

/// Enumerator produces target type which is generic over a buffer type
///
/// This exists because Enumerator implementations are ugly types that
/// I want to keep entirely opaque (i.e. only ever provide `impl Enumerator`).
/// However, if the Target is part of the Enumerator trait, and the implementation
/// is opaque, then there is no way for the user to know what is the Target type.
/// since it is not really possible to even indicate trait bounds on the generic
/// associated type. It may be possible to use feature(non_lifetime_binders)
/// to indicate the Target, but it is still experimental.
///
/// By separating the Target type into a separate trait, we can implement the
/// Target for a marker type that can be made public.
pub trait EnumeratorTarget {
    /// Target which is generic over buffer type
    type Target<B>;
}

/// Default target for an Enumerator
///
/// By default, the enumerator simply produces the
/// same buffer type that was provided in the first place.
///
/// Some more exotic Enumerators may wrap the buffer into
/// another type to provide more invariants.
pub struct DefaultTarget(PhantomData<()>);

impl EnumeratorTarget for DefaultTarget {
    type Target<B> = B;
}
