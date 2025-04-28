use std::marker::PhantomData;
use std::ops::Deref;

use crate::type_conversions::ConvertWrapper;

use vk_safe_sys as vk;

use vk::{BaseInStructure, BaseOutStructure};

// #[doc(hidden)]
// pub unsafe trait Pnext<S>: vk::StructExtends<S> {
//     /// link all members in the p_next chain
//     ///
//     /// By taking a reference to the struct, we can ensure that it
//     /// remains in place for the length of the borrow (baring any kind of internal mutability)
//     /// Thus, we can assign all the p_next pointers, and know they will remain valid
//     /// until the [`Linked`] is dropped.
//     fn link<'a>(&'a self) -> Linked<'a, Self>;
// }

/// Represents a collection of structs that can be included in a pNext chain
///
/// This is used to extend functionality in Vulkan by appending
/// structs to a particular struct and forming a linked list.
pub unsafe trait Pnext<S, Dep> {
    type Pnext<Tag>: LinkMut;
    #[doc(hidden)]
    fn p_next_uninit<Tag>() -> std::mem::MaybeUninit<Self::Pnext<Tag>> {
        std::mem::MaybeUninit::uninit()
    }
}

unsafe impl<S, Dep> Pnext<S, Dep> for () {
    type Pnext<Tag> = ();
}

/// Link a collection of extension structs
///
/// ensure the sType is correctly set, and
/// set pNext to link each individual struct together.
#[doc(hidden)]
pub unsafe trait LinkMut {
    fn link_mut(this: *mut Self) -> *mut vk::BaseOutStructure;
}

/// `()` is used to represent an empty collection of extension structs
unsafe impl LinkMut for () {
    fn link_mut(_this: *mut Self) -> *mut vk::BaseOutStructure {
        std::ptr::null_mut()
    }
}

#[repr(transparent)]
#[doc(hidden)]
/// Created as a pointer for a p_next chain that is properly linked
pub struct Linked<'a, T: ?Sized>(*const BaseInStructure, PhantomData<&'a T>);

impl<'a, T: ?Sized> Linked<'a, T> {
    pub unsafe fn new(ptr: *const BaseInStructure) -> Self {
        Self(ptr, PhantomData)
    }
}

unsafe impl<T> ConvertWrapper<*const BaseInStructure> for Linked<'_, T> {}

#[repr(transparent)]
#[doc(hidden)]
/// Created as a pointer for a mutable p_next chain that is properly linked
pub struct LinkedMut<'a, T: ?Sized>(*mut BaseOutStructure, PhantomData<&'a mut T>);

impl<'a, T: ?Sized> LinkedMut<'a, T> {
    pub unsafe fn new(ptr: *mut BaseOutStructure) -> Self {
        Self(ptr, PhantomData)
    }
}

unsafe impl<T> ConvertWrapper<*mut BaseOutStructure> for LinkedMut<'_, T> {}

/// A type that encapsulates a structure, and any extension structures
///
/// This type is returned from Vulkan commands which return a
/// particular struct `T` and possible set of extension structs `P`
pub struct Extended<T, P>(T, P);

pub(crate) fn make_extended<T, P>(base: T, extend: P) -> Extended<T, P> {
    Extended(base, extend)
}

impl<T, P> Deref for Extended<T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, P> Extended<T, P> {
    /// access the collection of extension structs
    pub fn p_next(&self) -> &P {
        &self.1
    }
}

impl<T: std::fmt::Debug, P: std::fmt::Debug> std::fmt::Debug for Extended<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extended")
            .field("base_struct", &self.0)
            .field("p_next", &self.1)
            .finish()
    }
}

#[doc(hidden)]
pub mod p_next_macro_prelude {
    pub use super::LinkMut;
    pub use crate::type_conversions::ConvertWrapper;
    pub use vk_safe_sys::structs as vk_structs;
    pub use vk_safe_sys::StructExtends;
    pub use vk_safe_sys::{BaseStructure, BaseStructureMut};
}

/// Make a list of types to be included in a p_next chain
///
/// When making a p_next chain for the Vulkan implementation to output data to,
/// list the types of the extension structs that you would like to include. The
/// underlying api will create the p_next chain and return the base main structure
/// along with all members in the p_next chain.
#[macro_export]
macro_rules! p_next {
    // When using p_next in the case of retrieving data output by the implementation,
    ( $($name:ident),+ $(,)? ) => {{
        use $crate::struct_extension::p_next_macro_prelude::*;
        $( use vk_structs::{$name}; )*

        // It is assumed that all BaseStructureMut types are only used to return
        // information from the Vulkan implementation, which does not need tyo be
        // generic over anything other than the tag. Thus, we just assume that
        // our Pnext collection only needs to be generic over the tag.

        /// This is a collection of extension structs containing:
        $( #[doc = stringify!($name)] )*
        #[doc = concat!("Defined at ", file!(), line!())]
        #[allow(non_snake_case)]
        pub struct StructExtension<Tag> {
            $(pub $name: $crate::vk::$name<Tag>,)*
        }

        unsafe impl<Tag> LinkMut for StructExtension<Tag> {
            fn link_mut(this: *mut Self) -> *mut vk_structs::BaseOutStructure {
                let mut ptr = std::ptr::null_mut();
                $(
                    let s: *mut $name = unsafe { (&raw mut (*this).$name) }.to_c();
                    BaseStructure::set_s_type(s);
                    BaseStructureMut::p_next_mut(s, ptr);
                    ptr = s.cast();
                )*
                ptr
            }
        }

        impl<Tag> std::fmt::Debug for StructExtension<Tag> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("pNext")
                    $( .field(&self.$name) )*
                    .finish()
            }
        }

        struct Pnext;

        unsafe impl<S, Dep> $crate::struct_extension::Pnext<S, Dep> for Pnext
        where
            $(
                $name: StructExtends<S>,
                Dep: vk_structs::struct_dependencies::$name::HasDependency,
            )*
        {
            type Pnext<Tag> = StructExtension<Tag>;
        }

        Pnext
    }};
}
