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

// ******* disambiguation labels ************
// for avoiding conflicting trait implementations
// the labels can be when the safe_transmute method is called

/// SafeTransmute label for converting to same type
pub struct Same();

/// SafeTransmute label for converting to other type
pub struct Other();

/// SafeTransmute label for converting to array type
pub struct Array();

/// SafeTransmute label for converting to MaybeUninit type
pub struct MaybeUninitLabel();

// *******************************************

/// Represent a type that can soundly transmute into another type T
///
/// It is intended that specific wrapper types in vk-safe will
/// implement the trait for converting to/from the raw generated
/// types in vk-safe-sys. The default method should not be overwritten.
///
/// Implementations for references and slices are provided
/// where the base types implement SafeTransmute.
pub(crate) unsafe trait SafeTransmute<T: ?Sized, L = Other> {
    fn safe_transmute(self) -> T
    where
        Self: Sized,
        T: Sized,
    {
        let ret = unsafe { std::mem::transmute_copy(&self) };
        std::mem::forget(self);
        ret
    }
}

unsafe impl<T> SafeTransmute<T, Same> for T {
    fn safe_transmute(self) -> T {
        self
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized, L> SafeTransmute<&'a T, (Other, L)> for &'a U
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> &'a T {
        unsafe { std::mem::transmute_copy(&self) }
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized, L> SafeTransmute<*const T, (Other, L)> for &'a U
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> *const T {
        unsafe { std::mem::transmute_copy(&self) }
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized, L> SafeTransmute<&'a mut T, (Other, L)> for &'a mut U
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> &'a mut T {
        unsafe { std::mem::transmute_copy(&self) }
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized, L> SafeTransmute<*mut T, (Other, L)> for &'a mut U
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> *mut T {
        unsafe { std::mem::transmute_copy(&self) }
    }
}

unsafe impl<'a, T, U, L> SafeTransmute<&'a [T], (Array, L)> for &'a [U]
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> &'a [T] {
        unsafe { std::mem::transmute(self) }
    }
}

unsafe impl<'a, T, U, L> SafeTransmute<*const T, (Array, L)> for &'a [U]
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> *const T {
        self.as_ptr().cast()
    }
}

unsafe impl<'a, T, U, L> SafeTransmute<&'a mut [T], (Array, L)> for &'a mut [U]
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> &'a mut [T] {
        unsafe { std::mem::transmute(self) }
    }
}

unsafe impl<'a, T, U, L> SafeTransmute<*mut T, (Array, L)> for &'a mut [U]
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> *mut T {
        self.as_mut_ptr().cast()
    }
}

/// This implementation is used in helper_macros::enumerator_code2!()
unsafe impl<T, U, L> SafeTransmute<*mut T, (MaybeUninitLabel, L)> for &mut [MaybeUninit<U>]
where
    U: SafeTransmute<T, L>,
{
    fn safe_transmute(self) -> *mut T {
        self.as_mut_ptr().cast()
    }
}

// / standalone const fn to allow safely transmuting slices in const context, since the trait way cannot be const currently
// pub(crate) const fn transmute_slice<A, B>(a: &[A]) -> &[B]
// where
//     A: SafeTransmute<B>,
// {
//     unsafe { std::mem::transmute(a) }
// }
