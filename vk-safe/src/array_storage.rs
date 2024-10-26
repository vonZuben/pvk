use crate::error::Error;
use core::ops::{Deref, DerefMut};
use std::mem::MaybeUninit;

use vk_safe_sys as vk;

/// A trait for handling storage space for "Enumerate" and "Get" commands in Vulkan
///
/// An implementor of this trait indicates "if" and "how" to allocate storage space;
/// provides mutable access to uninitialized memory to write to; and finalizes the initialized storage.
///
/// Commands which take an `impl ArrayStorage` type will first call [`ArrayStorage::allocate`], which enables the implementation to query the length of to-be-returned data.
/// Then [`ArrayStorage::uninit_slice`] will be called in order to get a slice of uninitialized data to be written to. Last, [`ArrayStorage::finalize`] will be called to
/// allow the implementation to perform any last work needed to make a safe initialized memory type that can be returned to the user.
///
/// Implementations of this are provided for [`Vec`] and slices / arrays of [`MaybeUninit`]. You may implement this trait yourself for any of your own array like types.
pub trait ArrayStorage<T> {
    /// The final initialized storage type.
    type InitStorage: AsRef<[T]>;

    /// Query len of items to be returned, and allocate space for such.
    ///
    /// `len` is a closure which will call the underling Vulkan commands with a null pointer to query the length
    ///
    /// This method is provided as a no-op by default, which is useful when space is preallocated (e.g. for a slice).
    fn allocate(&mut self, len: impl FnOnce() -> Result<usize, vk::Result>) -> Result<(), Error> {
        let _ = len;
        Ok(())
    }

    /// Provide the uninitialized space to which the Vulkan command will write to.
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>];

    /// Finalize len amount of initialized memory.
    /// # Safety
    /// `len` represents how much memory was written to the slice from [`ArrayStorage::uninit_slice`]. `len` comes from the underlying Vulkan implementation
    /// (i.e. the driver for your hardware), and *should* always be less than or equal to the len of the slice returned from [`ArrayStorage::uninit_slice`].
    /// However, this is not validated and there could be a broken Vulkan implementation, so it is recommended to use `min(len, your_memory_capacity)`.
    fn finalize(self, len: usize) -> Self::InitStorage;
}

impl<T> ArrayStorage<T> for Vec<MaybeUninit<T>> {
    type InitStorage = Vec<T>;
    fn allocate(&mut self, len: impl FnOnce() -> Result<usize, vk::Result>) -> Result<(), Error> {
        self.clear();
        self.reserve_exact(len()?);
        Ok(())
    }
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.capacity()) }
    }
    fn finalize(mut self, len: usize) -> Self::InitStorage {
        let len = std::cmp::min(len, self.capacity());
        unsafe {
            self.set_len(len);
            std::mem::transmute(self)
        }
    }
}

impl<'a, T> ArrayStorage<T> for &'a mut [MaybeUninit<T>] {
    type InitStorage = &'a mut [T];
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>] {
        self
    }
    fn finalize(self, len: usize) -> Self::InitStorage {
        assert!(len <= self.len());
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), len) }
    }
}

/// This is used to implement EnumeratorStorage for [T; LEN] type of storage.
/// This keeps track of how much of the array is actually initialized.
pub struct InitArray<const LEN: usize, T> {
    data: [MaybeUninit<T>; LEN],
    init_len: usize,
}

impl<const LEN: usize, T> InitArray<LEN, T> {
    fn get_initialized(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().cast(), self.init_len) }
    }
    fn get_initialized_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.init_len) }
    }
}

impl<const LEN: usize, T> Drop for InitArray<LEN, T> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() {
            for t in self.get_initialized_mut() {
                unsafe {
                    std::ptr::drop_in_place(t);
                }
            }
        }
    }
}

impl<const LEN: usize, T> Deref for InitArray<LEN, T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.get_initialized()
    }
}

impl<const LEN: usize, T> DerefMut for InitArray<LEN, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_initialized_mut()
    }
}

impl<const LEN: usize, T> AsRef<[T]> for InitArray<LEN, T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, const LEN: usize, T> ArrayStorage<T> for [MaybeUninit<T>; LEN] {
    type InitStorage = InitArray<LEN, T>;
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>] {
        self
    }
    fn finalize(self, len: usize) -> Self::InitStorage {
        assert!(len <= self.len());
        InitArray {
            data: self,
            init_len: len,
        }
    }
}

// internal trait for converting between different len types
// most Rust data structures use a 'usize' len type
// most Vulkan arrays use uint32_t
// this is her to make it easy to convert between the two in a natural friction free way
// and to make it easy to deal with other possible array length types

pub trait VulkanLenType {
    fn to_usize(self) -> usize;
    fn from_usize(len: usize) -> Self;
}

impl VulkanLenType for u32 {
    fn to_usize(self) -> usize {
        self as _
    }
    fn from_usize(len: usize) -> Self {
        if len > u32::MAX as usize {
            u32::MAX
        } else {
            len as _
        }
    }
}

/// Represents a buffer that can store a certain number of elements
///
/// Many vulkan APIs require the user to allocate space in advance to
/// return elements into. This is an abstraction to allow different
/// types of buffers to be used.
///
/// This abstraction allows a buffer to have a certain amount of
/// uninitialized space pre-allocated, and the len can be set afterwards.
///
/// It is unsafe to implement this trait because unsafe code relies
/// on the invariants of the capacity, len, and ptr methods. Default
/// methods provided by this trait also rely on those invariants.
pub unsafe trait Buffer<T> {
    /// Returns the number of elements of `T` that the buffer can hold.
    fn capacity(&self) -> usize;

    /// Returns a pointer to the beginning of the buffer
    ///
    /// The returned ptr must point to the beginning of a block of
    /// memory which can store `self.capacity()` amount of `T`.
    fn ptr(&self) -> *const T;

    /// Returns a mut pointer to the beginning of the buffer
    ///
    /// The returned ptr must point to the beginning of a block of
    /// memory which can store `self.capacity()` amount of `T`.
    fn ptr_mut(&mut self) -> *mut T;

    /// Returns the number of elements of `T` that are initialized in the buffer.
    ///
    /// This is the number of initialized elements starting from the beginning
    /// of the buffer.
    fn len(&self) -> usize;

    /// Set the len of the buffer
    ///
    /// After writing `len` number of elements to the ptr returned by `self.ptr_mut()`,
    /// this should be called to set `len` so the Buffer knows how many elements of `T`
    /// are initialized.
    ///
    /// ##SAFETY
    /// The caller of this must ensure `len` == number of elements that
    /// are properly initialized and written to `self.ptr_mut()`
    unsafe fn set_len(&mut self, len: usize);

    /// Get a slice to the initialized elements in the buffer
    fn get_slice(&self) -> &[T] {
        // SAFETY: the trait implementor is responsible for providing the correct pointer and len
        unsafe { std::slice::from_raw_parts(self.ptr(), self.len()) }
    }

    /// Get a mut slice to the initialized elements in the buffer
    fn get_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: the trait implementor is responsible for providing the correct pointer and len
        unsafe { std::slice::from_raw_parts_mut(self.ptr_mut(), self.len()) }
    }
}

unsafe impl<T> Buffer<T> for Vec<T> {
    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn ptr(&self) -> *const T {
        self.as_ptr()
    }

    fn ptr_mut(&mut self) -> *mut T {
        self.as_mut_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }

    unsafe fn set_len(&mut self, len: usize) {
        // SAFETY: the caller has the responsibility to set the correct len
        unsafe {
            self.set_len(len);
        }
    }
}

unsafe impl<T, B: Buffer<T>> Buffer<T> for &mut B {
    fn capacity(&self) -> usize {
        (**self).capacity()
    }

    fn ptr(&self) -> *const T {
        (**self).ptr()
    }

    fn ptr_mut(&mut self) -> *mut T {
        (**self).ptr_mut()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    unsafe fn set_len(&mut self, len: usize) {
        (**self).set_len(len);
    }
}
