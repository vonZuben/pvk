use core::ops::{Deref, DerefMut};
use std::mem::MaybeUninit;

type Result<T> = std::result::Result<T, vk_safe_sys::Result>;

/// This is used for vulkan commands that enumerate or get multiple items,
/// where the user needs to provide the space to store the items.
///
/// This allows users to control how the want to allocate space.
///
/// Implementations are provided for basic std type that are [T] like.
///
/// Users can implement for custom types.
pub trait ArrayStorage<T> {
    /// The final initialized storage type.
    type InitStorage : AsRef<[T]>;
    /// Allow control of len of items to be returned.
    /// If preallocated space is provided, then there is no reason to get len (e.g. for a slice).
    fn allocate(&mut self, _len: impl FnOnce() -> Result<usize>) -> Result<()> {Ok(())}
    /// Provide the uninitialized space to which the Vulkan command will write to.
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>];
    /// Finalize len amount of initialized memory.
    /// # Safety
    /// len comes from the Vulkan implementation, and *should* always be less than or equal to
    /// the len of the slice returned from uninit_slice. However, there could be risk of faulty or
    /// malicious Vulkan implementations, so it is recommended to assert! that len is not too long
    /// for the capacity of your memory.
    fn finalize(self, len: usize) -> Self::InitStorage;
}

pub(crate) struct UninitArrayInitializer<'a, T> {
    initialized_count: usize,
    array_iter: std::slice::IterMut<'a, MaybeUninit<T>>,
}

#[derive(Debug)]
pub struct ArrayFullError;

pub type InitResult = std::result::Result<(), ArrayFullError>;

impl std::fmt::Display for ArrayFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for ArrayFullError {}

impl<'a, T> UninitArrayInitializer<'a, T> {
    pub(crate) fn new(array_iter: std::slice::IterMut<'a, MaybeUninit<T>>) -> Self {
        Self { initialized_count: 0, array_iter }
    }
    pub(crate) fn push(&mut self, t: T) -> InitResult {
        let to_write = self.array_iter.next().ok_or(ArrayFullError)?;
        to_write.write(t);
        self.initialized_count += 1;
        Ok(())
    }
    pub(crate) fn initialized_count(&self) -> usize {
        self.initialized_count
    }
}

impl<T> ArrayStorage<T> for Vec<MaybeUninit<T>> {
    type InitStorage = Vec<T>;
    fn allocate(&mut self, len: impl FnOnce() -> Result<usize>) -> Result<()> {
        self.clear();
        self.reserve_exact(len()?);
        Ok(())
    }
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.capacity()) }
    }
    fn finalize(mut self, len: usize) -> Self::InitStorage {
        assert!(len <= self.capacity());
        unsafe {
            self.set_len(len);
            std::mem::transmute(self)
        }
    }
}

impl<'a, T> ArrayStorage<T> for &'a mut [MaybeUninit<T>] {
    type InitStorage = &'a mut [T];
    fn uninit_slice(&mut self) -> &mut [MaybeUninit<T>]  {
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
                unsafe { std::ptr::drop_in_place(t); }
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
        }
        else {
            len as _
        }
    }
}