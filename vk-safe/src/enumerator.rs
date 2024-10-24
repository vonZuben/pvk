use crate::array_storage::Buffer;
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
pub trait Enumerator<I> {
    /// Automatically enumerate or get the items
    ///
    /// This will automatically call the underlying Vulkan command
    /// a first time in order to query the number of items, allocate
    /// space with a [`Vec`] for the number of items, and call the
    /// command a second time with the allocated space to enumerate
    /// or get the items.
    fn auto_get_enumerate(&self) -> Result<Vec<I>, Error> {
        let mut vec = Vec::with_capacity(self.get_len()?);
        self.get_enumerate(&mut vec)?;
        Ok(vec)
    }

    /// Use the Vulkan command to query the number of items
    ///
    /// Generally, this calls the underlying Vulkan command
    /// with a null pointer for the return space, which is a signal
    /// to not enumerate or get any items, and only return the number
    /// of items that are available.
    fn get_len(&self) -> Result<usize, Error>;

    /// Enumerate or get the items using the provided buffer space
    ///
    /// Generally, this calls the underlying Vulkan command
    /// with a pointer and capacity of the provided space, and
    /// the Vulkan implementation with write the items to the space,
    /// and return the number of items actually written.
    fn get_enumerate(&self, buffer: &mut impl Buffer<I>) -> Result<(), Error>;
}
