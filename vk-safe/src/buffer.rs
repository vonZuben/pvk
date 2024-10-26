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
