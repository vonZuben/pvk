use std::os::raw::c_char;

use std::mem::MaybeUninit;

/// Covert a Rust type to a C type equivalent
pub(crate) trait ToC<C> {
    fn to_c(self) -> C;
}

/// If the Rust and C types are the same, then no conversion
impl<C> ToC<C> for C {
    fn to_c(self) -> C {
        self
    }
}

impl ToC<*const c_char> for crate::VkStr<'_> {
    fn to_c(self) -> *const c_char {
        self.as_ptr()
    }
}

impl ToC<*const c_char> for Option<crate::VkStr<'_>> {
    fn to_c(self) -> *const c_char {
        match self {
            Some(s) => s.as_ptr(),
            None => std::ptr::null(),
        }
    }
}

impl<'a, P> ToC<*const P> for Option<&'a P> {
    fn to_c(self) -> *const P {
        // Option<&P> should be same as &P
        unsafe { std::mem::transmute(self) }
    }
}

/// Represent a type that can soundly transmute into another type T
pub(crate) unsafe trait SafeTransmute<T> {}

/// of course all T can transmute to themselves
unsafe impl<T> SafeTransmute<T> for T {}

/// Extension trait intended for slices.
/// Provides the operation of transmuting a slice of U to a slice of T when safe to do so
///
/// This api is pretty experimental and may not all be useful
pub(crate) unsafe trait TransmuteSlice<T> {
    /// transmute a slice of U to a slice of T
    fn safe_transmute_slice<'a>(&'a self) -> &'a [T];
    /// transmute a mut slice of U to a mut slice of T
    fn safe_transmute_slice_mut<'a>(&mut self) -> &'a mut [T];
}

unsafe impl<T, U> TransmuteSlice<T> for [U]
where
    U: SafeTransmute<T>,
{
    fn safe_transmute_slice<'a>(&'a self) -> &'a [T] {
        unsafe { std::mem::transmute::<&[U], &[T]>(self) }
    }

    fn safe_transmute_slice_mut<'a>(&mut self) -> &'a mut [T] {
        unsafe { std::mem::transmute::<&mut [U], &mut [T]>(self) }
    }
}

/// Like the [TransmuteSlice] but intended for a slice of MaybeUninit<U> where U: SafeTransmute<T>
/// only raw_mut is provided since it is the only useful method for uninitialized memory
pub(crate) unsafe trait TransmuteUninitSlice<T> {
    /// transmute a mut slice of uninitialized U to a mut pointer T and length tuple
    fn safe_transmute_uninit_slice(&mut self) -> *mut T;
}

unsafe impl<T, U> TransmuteUninitSlice<T> for [MaybeUninit<U>]
where
    U: SafeTransmute<T>,
{
    fn safe_transmute_uninit_slice(&mut self) -> *mut T {
        self.as_mut_ptr().cast()
    }
}

/// standalone const fn to allow safely transmuting slices in const context, since the trait way cannot be const currently
pub(crate) const fn transmute_slice<A, B>(a: &[A]) -> &[B]
where
    A: SafeTransmute<B>,
{
    unsafe { std::mem::transmute(a) }
}
