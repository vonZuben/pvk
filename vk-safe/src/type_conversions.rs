use std::os::raw::c_char;

use std::mem::{ManuallyDrop, MaybeUninit};

/// Covert a Rust type to a C type equivalent
pub(crate) trait ToC<C, L = Other> {
    fn to_c(self) -> C;
}

impl<T, C, L> ToC<C, L> for T
where
    T: ConvertWrapper<C, L>,
{
    fn to_c(self) -> C {
        self.to_c()
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
// the labels can be inferred when the trait methods are called

/// ConvertWrapper label for converting to same type
pub struct Same();

/// ConvertWrapper label for converting to other type
pub struct Other();

/// ConvertWrapper label for converting to array type
pub struct Array();

/// ConvertWrapper label for converting to MaybeUninit type
pub struct MaybeUninitLabel();

// *******************************************

/// Convert between safe and raw vk-safe types
///
/// vk-safe has many safe wrapper types that simply wrap
/// the raw c-types and add an invariant that it is
/// initialized and safe to use in the vk-safe api.
///
/// This trait is for converting between the vk-safe
/// and raw c-type versions of such a type.
///
/// By default, this trait performs the conversion
/// by manually reinterpreting the bytes of Self to T,
/// while ensuring that the size and alignment match.
pub(crate) unsafe trait ConvertWrapper<T: ?Sized, L = Other> {
    /// Convert to raw c-type
    ///
    /// Convert from the safe api wrapper type to the raw c-type.
    /// This is mainly used internally for calling to raw Vulkan
    /// commands.
    fn to_c(self) -> T
    where
        T: Sized,
        Self: Sized,
    {
        union U<A, B> {
            a: ManuallyDrop<A>,
            b: ManuallyDrop<B>,
        }

        const {
            assert!(std::mem::size_of::<T>() == std::mem::size_of::<Self>());
            assert!(std::mem::align_of::<T>() == std::mem::align_of::<Self>())
        }

        let u = U {
            a: ManuallyDrop::new(self),
        };
        ManuallyDrop::into_inner(unsafe { u.b })
    }

    /// Convert from the raw c-type
    ///
    /// Create a safe api type from the raw c-type. Must ensure
    /// that the raw c-type has been properly initialized.
    ///
    /// In general, a raw-ctype has been properly initialized
    /// if all of the members have been set while adhering to the
    /// VUIDs for the given type. The v-safe wrapper may have
    /// additional invariants that must also be adhered to.
    unsafe fn from_c(c: T) -> Self
    where
        T: Sized,
        Self: Sized,
    {
        union U<A, B> {
            a: ManuallyDrop<A>,
            b: ManuallyDrop<B>,
        }

        const {
            assert!(std::mem::size_of::<T>() == std::mem::size_of::<Self>());
            assert!(std::mem::align_of::<T>() == std::mem::align_of::<Self>())
        }

        let u = U {
            a: ManuallyDrop::new(c),
        };
        ManuallyDrop::into_inner(unsafe { u.b })
    }
}

unsafe impl<T> ConvertWrapper<T, Same> for T {
    fn to_c(self) -> T {
        self
    }

    unsafe fn from_c(c: T) -> Self {
        c
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized, L> ConvertWrapper<&'a T, (Other, L)> for &'a U where
    U: ConvertWrapper<T, L>
{
}

unsafe impl<'a, T, U, L> ConvertWrapper<*const T, (Other, L)> for &'a U
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> *const T {
        unsafe { std::mem::transmute(self) }
    }

    unsafe fn from_c(c: *const T) -> Self {
        unsafe { std::mem::transmute(c) }
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized, L> ConvertWrapper<&'a mut T, (Other, L)> for &'a mut U where
    U: ConvertWrapper<T, L>
{
}

unsafe impl<'a, T, U, L> ConvertWrapper<*mut T, (Other, L)> for &'a mut U
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> *mut T {
        unsafe { std::mem::transmute(self) }
    }

    unsafe fn from_c(c: *mut T) -> Self {
        unsafe { std::mem::transmute(c) }
    }
}

unsafe impl<'a, T, U, L> ConvertWrapper<*mut T, (Other, L)> for *mut U
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> *mut T {
        self.cast()
    }

    unsafe fn from_c(c: *mut T) -> Self {
        c.cast()
    }
}

unsafe impl<'a, T, U, L> ConvertWrapper<&'a [T], (Array, L)> for &'a [U]
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> &'a [T] {
        unsafe { std::mem::transmute(self) }
    }

    unsafe fn from_c(c: &'a [T]) -> Self {
        unsafe { std::mem::transmute(c) }
    }
}

unsafe impl<'a, T, U, L> ConvertWrapper<*const T, (Array, L)> for &'a [U]
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> *const T {
        self.as_ptr().cast()
    }

    unsafe fn from_c(_c: *const T) -> Self {
        const { panic!("Cannot turn a raw pointer into a slice") }
    }
}

unsafe impl<'a, T, U, L> ConvertWrapper<&'a mut [T], (Array, L)> for &'a mut [U]
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> &'a mut [T] {
        unsafe { std::mem::transmute(self) }
    }

    unsafe fn from_c(c: &'a mut [T]) -> Self {
        unsafe { std::mem::transmute(c) }
    }
}

unsafe impl<'a, T, U, L> ConvertWrapper<*mut T, (Array, L)> for &'a mut [U]
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> *mut T {
        self.as_mut_ptr().cast()
    }

    unsafe fn from_c(_c: *mut T) -> Self {
        const { panic!("Cannot turn a raw pointer into a slice") }
    }
}

/// This implementation is used in helper_macros::enumerator_code2!()
unsafe impl<T, U, L> ConvertWrapper<*mut T, (MaybeUninitLabel, L)> for &mut [MaybeUninit<U>]
where
    U: ConvertWrapper<T, L>,
{
    fn to_c(self) -> *mut T {
        self.as_mut_ptr().cast()
    }

    unsafe fn from_c(_c: *mut T) -> Self {
        const { panic!("Cannot turn a raw pointer into a slice") }
    }
}

/// Allow converting from raw c type to wrapper in const context
pub(crate) const unsafe fn convert_wrapper_from_c<T, U>(u: U) -> T
where
    T: ConvertWrapper<U>,
{
    union C<A, B> {
        a: ManuallyDrop<A>,
        b: ManuallyDrop<B>,
    }

    const {
        assert!(std::mem::size_of::<T>() == std::mem::size_of::<U>());
        assert!(std::mem::align_of::<T>() == std::mem::align_of::<U>())
    }

    let u = C {
        a: ManuallyDrop::new(u),
    };
    ManuallyDrop::into_inner(unsafe { u.b })
}
